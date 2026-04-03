use crate::async_utils::DroppingOnce;
use crate::traktor_api::model::{
    AppMessage, ConnectionResponse, InitializeRequest, ServerMessage, UpdateRequest,
};
use crate::traktor_api::{StateUpdate, ID};
use bytes::Bytes;
use iced::futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use iced::futures::channel::oneshot;
use iced::futures::{stream, TryFutureExt};
use iced::futures::{SinkExt, Stream, StreamExt};
use libmdns::{Responder, Service};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::Filter;

const MAX_QUEUE_LENGTH: usize = 20;

struct TraktorServer {
    output: UnboundedSender<ServerMessage>,

    debug_logging: bool,
    session_id: String,

    is_initialized: bool,
    queue: Vec<StateUpdate>,

    deck_files: [String; 4],
    loaded_images: Vec<String>,
    pending_images: Vec<String>,

    cover_socket_id: usize,
    cover_sockets: HashMap<usize, UnboundedSender<warp::ws::Message>>,
}

impl TraktorServer {
    pub fn new(output: UnboundedSender<ServerMessage>) -> Self {
        TraktorServer {
            output,

            debug_logging: false,
            session_id: "".to_owned(),

            is_initialized: false,
            queue: Vec::new(),

            deck_files: Default::default(),
            loaded_images: Vec::new(),
            pending_images: Vec::new(),

            cover_socket_id: 0,
            cover_sockets: HashMap::new(),
        }
    }

    async fn send_message(&mut self, message: ServerMessage) {
        let _ = self.output.send(message).await;
    }

    async fn send_messages(&mut self, messages: impl IntoIterator<Item=ServerMessage>) {
        let _ = self
            .output
            .send_all(&mut stream::iter(messages).map(Ok))
            .await;
    }

    pub async fn send_ready(&mut self, app_input_sender: UnboundedSender<AppMessage>) {
        self.send_message(ServerMessage::Ready(app_input_sender))
            .await
    }

    pub fn reconnect(&mut self, debug_logging: bool) {
        self.session_id = Uuid::new_v4().to_string();
        self.debug_logging = debug_logging;

        self.is_initialized = false;
        self.queue.clear();
    }

    fn get_required_images(&self) -> Vec<String> {
        let mut required_images: Vec<String> = self.deck_files
            .iter()
            .filter(|&f| !f.is_empty())
            .map(|f| f.to_owned())
            .collect();
        required_images.dedup();

        required_images
    }

    async fn on_update_deck_files(&mut self) {
        let required_images = self.get_required_images();

        self.loaded_images.retain(|i| required_images.contains(i));
        self.pending_images.retain(|i| required_images.contains(i));

        let new_images = required_images
            .iter()
            .filter(|&i| !self.loaded_images.contains(i) && !self.pending_images.contains(i));

        for img in new_images {
            for socket in self.cover_sockets.values_mut() {
                _ = socket.send(warp::ws::Message::text(img)).await;
            }
        }
    }

    async fn handle_connect(&mut self) -> warp::reply::Json {
        warp::reply::json(&ConnectionResponse {
            session_id: self.session_id.to_owned(),
            debug_logging: self.debug_logging,
        })
    }

    async fn handle_init(&mut self, request: InitializeRequest) -> impl warp::Reply + use<> {
        if request.session_id == self.session_id {
            let time_offset_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| (request.timestamp as i64) - (d.as_millis() as i64))
                .unwrap_or(0);

            for i in 0..4 {
                self.deck_files[i] = request.state.decks[i].content.file_path.clone();
            }
            self.on_update_deck_files().await;

            self.send_message(ServerMessage::Connect {
                time_offset_ms,
                initial_state: Box::new(request.state),
            })
                .await;

            let mut messages = self
                .queue
                .drain(..)
                .map(ServerMessage::Update)
                .collect::<Vec<_>>();
            self.send_messages(messages.drain(..)).await;

            self.is_initialized = true;
        }

        self.session_id.to_owned()
    }

    async fn handle_update(
        &mut self,
        session_id: String,
        update: StateUpdate,
    ) -> impl warp::Reply + use < > {
        if session_id == self.session_id {
            if let StateUpdate::DeckContent(id, content) = &update {
                self.deck_files[*id as usize] = content.file_path.clone()
            }
            self.on_update_deck_files().await;

            if self.is_initialized {
                self.send_message(ServerMessage::Update(update)).await;
            } else {
                self.queue.push(update);

                if self.queue.len() > MAX_QUEUE_LENGTH {
                    self.reconnect(self.debug_logging);
                }
            }
        }

        self.session_id.to_owned()
    }

    async fn handle_cover(&mut self, path: String, data: Bytes) -> StatusCode {
        if data.is_empty() {
            return StatusCode::BAD_REQUEST;
        }

        if !self.get_required_images().contains(&path) {
            return StatusCode::OK;
        }

        println!("cover received for \"{}\"", path);

        self.pending_images.retain(|i| i != &path);
        if !self.loaded_images.contains(&path) {
            self.loaded_images.push(path.clone());
        }

        self.send_message(ServerMessage::CoverImage { path, data })
            .await;
        StatusCode::ACCEPTED
    }

    async fn handle_socket_connect(&mut self, mut tx: UnboundedSender<warp::ws::Message>) -> usize {
        while self.cover_sockets.contains_key(&self.cover_socket_id) {
            self.cover_socket_id += 1;
        }

        for img in &self.pending_images {
            _ = tx.send(warp::ws::Message::text(img)).await;
        }

        self.cover_sockets.insert(self.cover_socket_id, tx);
        self.cover_socket_id
    }

    fn handle_socket_disconnect(&mut self, id: usize) {
        self.cover_sockets.remove(&id);
    }

    async fn handle_log(&mut self, msg: String) -> impl warp::Reply + use < > {
        self.send_message(ServerMessage::Log(msg)).await;
        StatusCode::CREATED
    }
}

impl TraktorServer {
    pub fn routes(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone + use < > {
        Self::is_started(state.clone())
            .and(
                Self::route_connect(state.clone())
                    .or(Self::route_init(state.clone()))
                    .or(Self::route_update(state.clone()))
                    .or(Self::route_log(state.clone()))
                    .or(Self::route_cover(state.clone())),
            )
            .map(|_, reply| reply)
    }

    fn with_state(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(Arc<Mutex<Self>>,), Error=Infallible> + Clone {
        warp::any().map(move || state.clone())
    }

    fn is_started(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=((),), Error=warp::Rejection> + Clone {
        warp::any()
            .and(Self::with_state(state))
            .and_then(async |state: Arc<Mutex<Self>>| {
                let state = state.lock().await;

                if state.session_id.is_empty() {
                    Err(warp::reject::not_found())
                } else {
                    Ok(())
                }
            })
    }

    fn json_body<T: DeserializeOwned + Send>()
        -> impl Filter<Extract=(T,), Error=warp::Rejection> + Clone {
        warp::body::json()
    }

    fn route_connect(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        warp::path!("connect")
            .and(Self::with_state(state))
            .then(async |state: Arc<Mutex<Self>>| state.lock().await.handle_connect().await)
    }

    fn route_init(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        warp::path!("init")
            .and(warp::post())
            .and(Self::with_state(state))
            .and(Self::json_body())
            .then(
                async |state: Arc<Mutex<Self>>, request: InitializeRequest| {
                    state.lock().await.handle_init(request).await
                },
            )
    }

    fn route_update(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        warp::path("update")
            .and(warp::post())
            .and(Self::with_state(state))
            .and(Self::route_update_sub_routes())
            .then(async |state: Arc<Mutex<Self>>, (session_id, update)| {
                state.lock().await.handle_update(session_id, update).await
            })
    }

    fn route_update_sub_routes()
        -> impl Filter<Extract=((String, StateUpdate),), Error=warp::Rejection> + Clone {
        warp::path!("mixer")
            .and(Self::json_body())
            .then(async move |req: UpdateRequest<_>| {
                (req.session_id, StateUpdate::Mixer(req.state))
            })
            .or(warp::path!("channel0").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::Channel(ID::A, req.state))
                },
            ))
            .unify()
            .or(warp::path!("channel1").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::Channel(ID::B, req.state))
                },
            ))
            .unify()
            .or(warp::path!("channel2").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::Channel(ID::C, req.state))
                },
            ))
            .unify()
            .or(warp::path!("channel3").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::Channel(ID::D, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck0content").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckContent(ID::A, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck1content").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckContent(ID::B, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck2content").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckContent(ID::C, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck3content").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckContent(ID::D, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck0playstate").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckPlayState(ID::A, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck1playstate").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckPlayState(ID::B, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck2playstate").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckPlayState(ID::C, req.state))
                },
            ))
            .unify()
            .or(warp::path!("deck3playstate").and(Self::json_body()).then(
                async move |req: UpdateRequest<_>| {
                    (req.session_id, StateUpdate::DeckPlayState(ID::D, req.state))
                },
            ))
            .unify()
    }

    fn route_cover(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        warp::path!("cover").and(
            Self::route_cover_upload(state.clone()).or(Self::route_cover_socket(state.clone())),
        )
    }

    fn route_cover_upload(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        warp::post()
            .and(Self::with_state(state))
            .and(warp::body::bytes())
            .and(warp::query::<HashMap<String, String>>())
            .then(
                async |state: Arc<Mutex<Self>>, body: Bytes, query: HashMap<String, String>| {
                    match query.get("path") {
                        Some(path) => state.lock().await.handle_cover(path.to_owned(), body).await,
                        None => StatusCode::BAD_REQUEST,
                    }
                },
            )
    }

    fn route_cover_socket(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        warp::ws()
            .and(Self::with_state(state))
            .map(|ws: warp::ws::Ws, state: Arc<Mutex<Self>>| {
                ws.on_upgrade(move |socket| async move {
                    let (mut ws_tx, mut ws_rx) = socket.split();
                    let (tx, mut rx) = iced::futures::channel::mpsc::unbounded();

                    tokio::task::spawn(async move {
                        while let Some(message) = rx.next().await {
                            ws_tx
                                .send(message)
                                .unwrap_or_else(|e| {
                                    println!("websocket send error: {}", e);
                                })
                                .await;
                        }
                    });

                    let socket_id = state.lock().await.handle_socket_connect(tx).await;
                    println!("websocket connected");

                    while let Some(result) = ws_rx.next().await {
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                println!("websocket error: {}", e);
                                break;
                            }
                        };
                    }

                    state.lock().await.handle_socket_disconnect(socket_id);
                    println!("websocket disconnected");
                })
            })
    }

    fn route_log(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        warp::path!("log")
            .and(warp::post())
            .and(Self::with_state(state))
            .and(warp::body::bytes())
            .then(async |state: Arc<Mutex<Self>>, body: Bytes| {
                state
                    .lock()
                    .await
                    .handle_log(String::from_utf8_lossy(&body).into_owned())
                    .await
            })
    }
}

async fn server_main(
    addr: SocketAddr,
    output: UnboundedSender<ServerMessage>,
    mut input: UnboundedReceiver<AppMessage>,
    input_send: UnboundedSender<AppMessage>,
    cancelled: oneshot::Receiver<()>,
) {
    let state = Arc::new(Mutex::new(TraktorServer::new(output)));
    let routes = TraktorServer::routes(state.clone());

    println!("starting traktor server on {}", addr);
    let service = advertise_server(addr);

    let Ok(listener) = tokio::net::TcpListener::bind(addr).await else {
        println!("could not start traktor server on {}", addr);

        drop(service);
        return;
    };
    let server = warp::serve(routes).incoming(listener).graceful(async {
        cancelled.await.ok();
    });

    tokio::task::spawn(server.run());

    state.lock().await.send_ready(input_send).await;
    loop {
        match input.select_next_some().await {
            AppMessage::Reconnect { debug_logging } => state.lock().await.reconnect(debug_logging),
        }
    }
}

fn advertise_server(addr: SocketAddr) -> Service {
    let addr_vec = if !addr.ip().is_unspecified() {
        [addr.ip()].to_vec()
    } else {
        Vec::new()
    };
    let responder = Responder::new_with_ip_list(addr_vec).expect("could not create responder");
    let svc = responder.register(
        "_http._tcp",
        "traktor-di-webserver",
        addr.port(),
        &["path=/"],
    );
    println!("advertising traktor server on {}", addr);
    svc
}

pub fn run_server(addr: SocketAddr) -> impl Stream<Item=ServerMessage> {
    let (output, output_receive) = iced::futures::channel::mpsc::unbounded();
    let (input_send, input) = iced::futures::channel::mpsc::unbounded();
    let (cancel, cancelled) = oneshot::channel();

    let runner = DroppingOnce::new(
        server_main(addr, output, input, input_send, cancelled),
        move || {
            println!("stopping traktor server on {}", addr);
            let _ = cancel.send(());
        },
    )
        .filter_map(|_| async { None });

    stream::select(output_receive, runner)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use iced::futures::channel::mpsc as iced_mpsc;
    use serde_json::json;
    use tokio::time::{timeout, Duration};

    // ── Shared test helpers ───────────────────────────────────────────────────

    /// Creates a fresh server state and returns the message receiver alongside it.
    fn new_state() -> (
        Arc<Mutex<TraktorServer>>,
        iced_mpsc::UnboundedReceiver<ServerMessage>,
    ) {
        let (tx, rx) = iced_mpsc::unbounded();
        (Arc::new(Mutex::new(TraktorServer::new(tx))), rx)
    }

    /// JSON for a complete valid Traktor state.
    /// `deck0` has `file_path` set; decks 1-3 are empty.
    fn full_state_json(file_path: &str) -> serde_json::Value {
        json!({
            "mixer": {
                "xFader": 0.5, "masterVolume": 1.0,
                "cueVolume": 0.5, "cueMix": 0.5, "micVolume": 0.0
            },
            "channel0": {"cue": false, "volume": 1.0, "xFaderLeft": true, "xFaderRight": false},
            "channel1": {"cue": false, "volume": 0.5, "xFaderLeft": false, "xFaderRight": true},
            "channel2": {"cue": false, "volume": 0.0, "xFaderLeft": false, "xFaderRight": false},
            "channel3": {"cue": false, "volume": 0.0, "xFaderLeft": false, "xFaderRight": false},
            "deck0content": {
                "isLoaded": true, "number": 1, "title": "Test Track",
                "artist": "Test Artist", "album": "", "genre": "Waltz",
                "comment": "", "comment2": "", "label": "", "key": "",
                "filePath": file_path, "trackLength": 180.0, "bpm": 90.0
            },
            "deck1content": {
                "isLoaded": false, "number": 0, "title": "", "artist": "",
                "album": "", "genre": "", "comment": "", "comment2": "", "label": "",
                "key": "", "filePath": "", "trackLength": 0.0, "bpm": 0.0
            },
            "deck2content": {
                "isLoaded": false, "number": 0, "title": "", "artist": "",
                "album": "", "genre": "", "comment": "", "comment2": "", "label": "",
                "key": "", "filePath": "", "trackLength": 0.0, "bpm": 0.0
            },
            "deck3content": {
                "isLoaded": false, "number": 0, "title": "", "artist": "",
                "album": "", "genre": "", "comment": "", "comment2": "", "label": "",
                "key": "", "filePath": "", "trackLength": 0.0, "bpm": 0.0
            },
            "deck0playstate": {"timestamp": 0, "position": 0.0, "speed": 1.0},
            "deck1playstate": {"timestamp": 0, "position": 0.0, "speed": 0.0},
            "deck2playstate": {"timestamp": 0, "position": 0.0, "speed": 0.0},
            "deck3playstate": {"timestamp": 0, "position": 0.0, "speed": 0.0}
        })
    }

    fn init_body(session_id: &str, file_path: &str) -> serde_json::Value {
        json!({
            "sessionId": session_id,
            "timestamp": 1_700_000_000_000_u64,
            "state": full_state_json(file_path)
        })
    }

    fn mixer_update_body(session_id: &str) -> serde_json::Value {
        json!({
            "sessionId": session_id,
            "state": {
                "xFader": 0.3, "masterVolume": 0.8,
                "cueVolume": 0.5, "cueMix": 0.0, "micVolume": 0.0
            }
        })
    }

    /// Awaits the next message with a 500 ms timeout, panicking on timeout.
    async fn recv_msg(rx: &mut iced_mpsc::UnboundedReceiver<ServerMessage>) -> ServerMessage {
        timeout(Duration::from_millis(500), rx.next())
            .await
            .expect("timeout waiting for ServerMessage")
            .expect("channel closed unexpectedly")
    }

    /// Asserts that no message arrives within 50 ms.
    async fn assert_no_msg(rx: &mut iced_mpsc::UnboundedReceiver<ServerMessage>) {
        let result = timeout(Duration::from_millis(50), rx.next()).await;
        assert!(result.is_err(), "expected no message but one arrived");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Unit tests — exercised through warp::test (no real TCP socket)
    // ─────────────────────────────────────────────────────────────────────────

    // ── Session guard ─────────────────────────────────────────────────────────

    /// Every route returns 404 while session_id is empty (server not yet
    /// started via reconnect).
    #[tokio::test]
    async fn connect_before_reconnect_returns_404() {
        let (state, _rx) = new_state();
        let filter = TraktorServer::routes(state);

        let res = warp::test::request().path("/connect").reply(&filter).await;

        assert_eq!(res.status(), 404);
    }

    /// After reconnect the session_id is non-empty and /connect returns it.
    #[tokio::test]
    async fn connect_after_reconnect_returns_session_id_and_debug_flag() {
        let (state, _rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();

        let filter = TraktorServer::routes(state);
        let res = warp::test::request().path("/connect").reply(&filter).await;

        assert_eq!(res.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(body["sessionId"], session_id);
        assert_eq!(body["debugLogging"], false);
    }

    /// Debug logging flag is reflected in the /connect response.
    #[tokio::test]
    async fn connect_returns_debug_logging_true_when_set() {
        let (state, _rx) = new_state();
        state.lock().await.reconnect(true);

        let filter = TraktorServer::routes(state);
        let res = warp::test::request().path("/connect").reply(&filter).await;

        let body: serde_json::Value = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(body["debugLogging"], true);
    }

    /// Calling reconnect() again generates a new, different session ID.
    #[tokio::test]
    async fn reconnect_generates_new_session_id_each_time() {
        let (state, _rx) = new_state();

        state.lock().await.reconnect(false);
        let filter = TraktorServer::routes(state.clone());
        let res1 = warp::test::request().path("/connect").reply(&filter).await;
        let body1: serde_json::Value = serde_json::from_slice(res1.body()).unwrap();

        state.lock().await.reconnect(false);
        let res2 = warp::test::request().path("/connect").reply(&filter).await;
        let body2: serde_json::Value = serde_json::from_slice(res2.body()).unwrap();

        assert_ne!(body1["sessionId"], body2["sessionId"]);
    }

    // ── /init endpoint ────────────────────────────────────────────────────────

    /// A POST /init with the correct session ID emits ServerMessage::Connect
    /// and returns the session ID as the response body.
    #[tokio::test]
    async fn init_correct_session_emits_connect_and_returns_session_id() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();

        let filter = TraktorServer::routes(state);
        let body = init_body(&session_id, "/music/track.mp3");

        let res = warp::test::request()
            .method("POST")
            .path("/init")
            .json(&body)
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);
        assert_eq!(res.body().as_ref(), session_id.as_bytes());

        let msg = recv_msg(&mut rx).await;
        assert!(matches!(msg, ServerMessage::Connect { .. }));
    }

    /// A POST /init with a wrong session ID is silently ignored.
    #[tokio::test]
    async fn init_wrong_session_id_is_ignored() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);

        let filter = TraktorServer::routes(state);
        let body = init_body("00000000-0000-0000-0000-000000000000", "/music/track.mp3");

        let res = warp::test::request()
            .method("POST")
            .path("/init")
            .json(&body)
            .reply(&filter)
            .await;

        // Server still returns its own session ID, but no Connect message.
        assert_eq!(res.status(), 200);
        assert_no_msg(&mut rx).await;
    }

    /// Updates queued before init are flushed (as Update messages) immediately
    /// after a successful /init.
    #[tokio::test]
    async fn queued_updates_are_flushed_on_init() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();

        let filter = TraktorServer::routes(state);

        // Queue two mixer updates before initialising
        for _ in 0..2 {
            warp::test::request()
                .method("POST")
                .path("/update/mixer")
                .json(&mixer_update_body(&session_id))
                .reply(&filter)
                .await;
        }

        // Now init — Connect arrives, then the two queued Updates
        warp::test::request()
            .method("POST")
            .path("/init")
            .json(&init_body(&session_id, ""))
            .reply(&filter)
            .await;

        let connect = recv_msg(&mut rx).await;
        assert!(matches!(connect, ServerMessage::Connect { .. }));

        let upd1 = recv_msg(&mut rx).await;
        assert!(matches!(upd1, ServerMessage::Update(_)));

        let upd2 = recv_msg(&mut rx).await;
        assert!(matches!(upd2, ServerMessage::Update(_)));
    }

    // ── /update/* endpoints ───────────────────────────────────────────────────

    /// A POST /update/mixer with the correct session and after init emits
    /// ServerMessage::Update(Mixer(…)).
    #[tokio::test]
    async fn update_mixer_correct_session_emits_update() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();

        let filter = TraktorServer::routes(state);

        // Init first so is_initialized = true
        warp::test::request()
            .method("POST")
            .path("/init")
            .json(&init_body(&session_id, ""))
            .reply(&filter)
            .await;
        let _ = recv_msg(&mut rx).await; // discard Connect

        // Now send mixer update
        let res = warp::test::request()
            .method("POST")
            .path("/update/mixer")
            .json(&mixer_update_body(&session_id))
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);
        let msg = recv_msg(&mut rx).await;
        assert!(matches!(msg, ServerMessage::Update(StateUpdate::Mixer(_))));
    }

    /// An update with the wrong session ID is silently dropped.
    #[tokio::test]
    async fn update_wrong_session_id_is_dropped() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();
        let filter = TraktorServer::routes(state);

        // Init to mark as initialized
        warp::test::request()
            .method("POST")
            .path("/init")
            .json(&init_body(&session_id, ""))
            .reply(&filter)
            .await;
        let _ = recv_msg(&mut rx).await; // discard Connect

        warp::test::request()
            .method("POST")
            .path("/update/mixer")
            .json(&mixer_update_body("wrong-session-id"))
            .reply(&filter)
            .await;

        assert_no_msg(&mut rx).await;
    }

    /// An update arriving before /init is placed in the queue and NOT forwarded.
    #[tokio::test]
    async fn update_before_init_goes_to_queue_not_forwarded() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();
        let filter = TraktorServer::routes(state.clone());

        warp::test::request()
            .method("POST")
            .path("/update/mixer")
            .json(&mixer_update_body(&session_id))
            .reply(&filter)
            .await;

        assert_no_msg(&mut rx).await;
        assert_eq!(state.lock().await.queue.len(), 1);
    }

    /// When the update queue exceeds MAX_QUEUE_LENGTH the server reconnects
    /// (new session ID).
    #[tokio::test]
    async fn queue_overflow_triggers_reconnect() {
        let (state, _rx) = new_state();
        state.lock().await.reconnect(false);
        let session_before = state.lock().await.session_id.clone();

        let filter = TraktorServer::routes(state.clone());

        // Send MAX_QUEUE_LENGTH + 1 updates without calling /init
        for _ in 0..=MAX_QUEUE_LENGTH {
            warp::test::request()
                .method("POST")
                .path("/update/mixer")
                .json(&mixer_update_body(&session_before))
                .reply(&filter)
                .await;
        }

        let session_after = state.lock().await.session_id.clone();
        assert_ne!(
            session_before, session_after,
            "session ID should change after queue overflow"
        );
    }

    // ── /cover endpoint ───────────────────────────────────────────────────────

    /// A non-empty POST /cover for a required file path emits CoverImage and
    /// returns 202 Accepted.
    #[tokio::test]
    async fn cover_upload_for_required_path_emits_cover_image() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();
        let filter = TraktorServer::routes(state.clone());

        // Init with a deck that has a known file path
        warp::test::request()
            .method("POST")
            .path("/init")
            .json(&init_body(&session_id, "/music/track.mp3"))
            .reply(&filter)
            .await;
        let _ = recv_msg(&mut rx).await; // discard Connect

        let res = warp::test::request()
            .method("POST")
            .path("/cover?path=/music/track.mp3")
            .body(b"\xff\xd8\xff\xe0fake_jpeg".as_slice()) // non-empty body
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 202);
        let msg = recv_msg(&mut rx).await;
        assert!(
            matches!(msg, ServerMessage::CoverImage { path, .. } if path == "/music/track.mp3")
        );
    }

    /// An empty body returns 400 Bad Request and no message is emitted.
    #[tokio::test]
    async fn cover_upload_empty_body_returns_400() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();
        let filter = TraktorServer::routes(state.clone());

        warp::test::request()
            .method("POST")
            .path("/init")
            .json(&init_body(&session_id, "/music/track.mp3"))
            .reply(&filter)
            .await;
        let _ = recv_msg(&mut rx).await;

        let res = warp::test::request()
            .method("POST")
            .path("/cover?path=/music/track.mp3")
            .body(b"".as_slice())
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 400);
        assert_no_msg(&mut rx).await;
    }

    /// A cover upload for a path that is not loaded on any deck returns 200 OK
    /// and no message is emitted.
    #[tokio::test]
    async fn cover_upload_for_unknown_path_returns_200_no_message() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false);
        let session_id = state.lock().await.session_id.clone();
        let filter = TraktorServer::routes(state.clone());

        // Init with empty file paths
        warp::test::request()
            .method("POST")
            .path("/init")
            .json(&init_body(&session_id, ""))
            .reply(&filter)
            .await;
        let _ = recv_msg(&mut rx).await;

        let res = warp::test::request()
            .method("POST")
            .path("/cover?path=/music/not-loaded.mp3")
            .body(b"\xff\xd8\xff".as_slice())
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);
        assert_no_msg(&mut rx).await;
    }

    /// Missing `path` query parameter returns 400.
    #[tokio::test]
    async fn cover_upload_missing_path_query_param_returns_400() {
        let (state, _rx) = new_state();
        state.lock().await.reconnect(false);
        let filter = TraktorServer::routes(state);

        let res = warp::test::request()
            .method("POST")
            .path("/cover") // no ?path=...
            .body(b"data".as_slice())
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 400);
    }

    // ── /log endpoint ─────────────────────────────────────────────────────────

    /// With debug_logging enabled, POST /log emits a Log message and
    /// returns 201.
    #[tokio::test]
    async fn log_with_debug_logging_enabled_emits_log_message() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(true); // debug_logging = true
        let filter = TraktorServer::routes(state);

        let res = warp::test::request()
            .method("POST")
            .path("/log")
            .body(b"deck A loaded")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 201);
        let msg = recv_msg(&mut rx).await;
        assert!(matches!(msg, ServerMessage::Log(s) if s == "deck A loaded"));
    }

    /// With debug_logging disabled, POST /log still returns 201 but no
    /// Log message is forwarded.
    #[tokio::test]
    async fn log_without_debug_logging_no_message_emitted() {
        let (state, mut rx) = new_state();
        state.lock().await.reconnect(false); // debug_logging = false
        let filter = TraktorServer::routes(state);

        let res = warp::test::request()
            .method("POST")
            .path("/log")
            .body(b"some log")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 201);
        assert_no_msg(&mut rx).await;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Integration tests — real TCP server driven via run_server()
    // ─────────────────────────────────────────────────────────────────────────

    mod integration {
        use super::*;
        use iced::futures::channel::mpsc::UnboundedSender as IcedSender;
        use iced::futures::StreamExt as IcedStreamExt;
        use tokio::sync::mpsc as tokio_mpsc;

        // ── Test server harness ───────────────────────────────────────────────

        struct TestServer {
            pub addr: SocketAddr,
            pub session_id: String,
            messages: tokio_mpsc::UnboundedReceiver<ServerMessage>,
            app_tx: IcedSender<AppMessage>,
            _task: tokio::task::JoinHandle<()>,
        }

        impl TestServer {
            /// Binds to a random loopback port, starts the server, waits for
            /// Ready, sends Reconnect, and fetches the resulting session ID.
            async fn start() -> Self {
                // Grab a free port then immediately release it.
                let port = {
                    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                    l.local_addr().unwrap().port()
                };
                // Tiny sleep to let the OS reclaim the port before we bind again.
                tokio::time::sleep(Duration::from_millis(10)).await;

                let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
                let (col_tx, col_rx) = tokio_mpsc::unbounded_channel::<ServerMessage>();

                let task = tokio::spawn(async move {
                    let mut stream = Box::pin(run_server(addr));
                    while let Some(msg) = stream.next().await {
                        if col_tx.send(msg).is_err() {
                            break;
                        }
                    }
                });

                let mut server = Self {
                    addr,
                    session_id: String::new(),
                    messages: col_rx,
                    app_tx: Self::dummy_sender(), // replaced in wait_ready
                    _task: task,
                };
                server.wait_ready().await;
                server.session_id = server.fetch_session_id().await;
                server
            }

            fn dummy_sender() -> IcedSender<AppMessage> {
                let (tx, _) = iced::futures::channel::mpsc::unbounded();
                tx
            }

            /// Waits for ServerMessage::Ready, then sends AppMessage::Reconnect
            /// so the server assigns a session ID.
            async fn wait_ready(&mut self) {
                timeout(Duration::from_secs(5), async {
                    loop {
                        match self.messages.recv().await {
                            Some(ServerMessage::Ready(tx)) => {
                                tx.unbounded_send(AppMessage::Reconnect {
                                    debug_logging: false,
                                })
                                .ok();
                                self.app_tx = tx;
                                // Give the server event loop time to process Reconnect.
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                break;
                            }
                            Some(_) => {}
                            None => panic!("server stream ended before Ready"),
                        }
                    }
                })
                .await
                .expect("timed out waiting for server Ready");
            }

            async fn fetch_session_id(&self) -> String {
                let url = format!("http://{}/connect", self.addr);
                let resp = reqwest::get(&url).await.expect("GET /connect failed");
                let json: serde_json::Value = resp.json().await.unwrap();
                json["sessionId"].as_str().unwrap().to_owned()
            }

            /// Convenience: POST JSON to a path and return the response.
            async fn post_json(&self, path: &str, body: &serde_json::Value) -> reqwest::Response {
                let req = reqwest::Client::new()
                    .post(format!("http://{}{}", self.addr, path))
                    .json(body);
                req.send().await.expect("HTTP POST failed")
            }

            /// Awaits the next ServerMessage with a 2-second timeout.
            async fn recv(&mut self) -> ServerMessage {
                timeout(Duration::from_secs(2), self.messages.recv())
                    .await
                    .expect("timeout waiting for ServerMessage")
                    .expect("message channel closed")
            }
        }

        impl Drop for TestServer {
            fn drop(&mut self) {
                self._task.abort();
            }
        }

        // ── Tests ──────────────────────────────────────────────────────────────

        /// The server starts, binds its port, and responds to GET /connect with
        /// a valid UUID session ID.
        #[tokio::test]
        async fn server_starts_and_connect_returns_session_id() {
            let server = TestServer::start().await;

            assert!(
                !server.session_id.is_empty(),
                "session ID should be non-empty"
            );
            // Verify UUID format (8-4-4-4-12)
            assert!(
                server.session_id.len() == 36
                    && server.session_id.chars().filter(|&c| c == '-').count() == 4,
                "session ID should look like a UUID, got: {}",
                server.session_id
            );
        }

        /// Full handshake: POST /init → Connect message; POST /update/mixer →
        /// Update(Mixer) message.
        #[tokio::test]
        async fn full_handshake_and_mixer_update_round_trip() {
            let mut server = TestServer::start().await;

            // Init
            let init = init_body(&server.session_id, "/music/track.mp3");
            let res = server.post_json("/init", &init).await;
            assert_eq!(res.status(), 200);

            let msg = server.recv().await;
            assert!(matches!(msg, ServerMessage::Connect { .. }));

            // Mixer update
            let mixer = mixer_update_body(&server.session_id);
            server.post_json("/update/mixer", &mixer).await;

            let msg = server.recv().await;
            assert!(matches!(msg, ServerMessage::Update(StateUpdate::Mixer(_))));
        }

        /// A WebSocket client connected to /cover receives the file path of a
        /// newly loaded deck when a deck content update arrives.
        #[tokio::test]
        async fn cover_websocket_receives_file_path_on_deck_update() {
            use futures_util::StreamExt as FutStreamExt;

            let mut server = TestServer::start().await;

            // Connect WebSocket BEFORE sending the deck update
            let ws_url = format!("ws://{}/cover", server.addr);
            let (mut ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
                .await
                .expect("WebSocket connection failed");

            // Init with an empty state first (no file paths yet)
            let init = init_body(&server.session_id, "");
            server.post_json("/init", &init).await;
            let _ = server.recv().await; // discard Connect

            // Now update deck0 content with a real file path
            let deck_update = json!({
                "sessionId": server.session_id,
                "state": {
                    "isLoaded": true, "number": 2, "title": "New Track",
                    "artist": "DJ Test", "album": "", "genre": "Tango",
                    "comment": "", "comment2": "", "label": "", "key": "",
                    "filePath": "/music/new_track.mp3",
                    "trackLength": 240.0, "bpm": 120.0
                }
            });
            server.post_json("/update/deck0content", &deck_update).await;

            // The server should push the file path to all connected WebSocket clients
            let ws_msg = timeout(Duration::from_secs(2), ws_stream.next())
                .await
                .expect("WS timeout")
                .expect("WS stream ended")
                .expect("WS error");

            assert_eq!(ws_msg.into_text().unwrap(), "/music/new_track.mp3");
        }

        /// Dropping TestServer aborts the task; a subsequent bind to the same
        /// address should succeed (server released the port).
        #[tokio::test]
        async fn server_releases_port_after_drop() {
            let server = TestServer::start().await;
            let addr = server.addr;
            drop(server);

            // Give the OS a moment to reclaim the port
            tokio::time::sleep(Duration::from_millis(200)).await;

            let rebind = tokio::net::TcpListener::bind(addr).await;
            assert!(rebind.is_ok(), "port should be available after server drop");
        }
    }
}
