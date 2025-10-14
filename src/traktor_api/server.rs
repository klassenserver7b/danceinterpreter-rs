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

    deck_files: (String, String, String, String),
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

    async fn send_messages(&mut self, messages: impl IntoIterator<Item = ServerMessage>) {
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
        let mut required_images: Vec<String> = vec![
            &self.deck_files.0,
            &self.deck_files.1,
            &self.deck_files.2,
            &self.deck_files.3,
        ]
        .into_iter()
        .filter(|&f| !f.is_empty()).map(|f| f.to_owned())
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

    async fn handle_connect(&mut self) -> impl warp::Reply + use<> {
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

            self.deck_files.0 = request.state.decks.0.content.file_path.clone();
            self.deck_files.1 = request.state.decks.1.content.file_path.clone();
            self.deck_files.2 = request.state.decks.2.content.file_path.clone();
            self.deck_files.3 = request.state.decks.3.content.file_path.clone();
            self.on_update_deck_files().await;

            self.send_message(ServerMessage::Connect {
                time_offset_ms,
                initial_state: request.state,
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

    async fn handle_update(&mut self, session_id: String, update: StateUpdate) -> impl warp::Reply + use<> {
        if session_id == self.session_id {
            match &update {
                StateUpdate::DeckContent(ID::A, content) => {
                    self.deck_files.0 = content.file_path.clone()
                }
                StateUpdate::DeckContent(ID::B, content) => {
                    self.deck_files.1 = content.file_path.clone()
                }
                StateUpdate::DeckContent(ID::C, content) => {
                    self.deck_files.2 = content.file_path.clone()
                }
                StateUpdate::DeckContent(ID::D, content) => {
                    self.deck_files.3 = content.file_path.clone()
                }
                _ => {}
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

    async fn handle_log(&mut self, msg: String) -> impl warp::Reply + use<> {
        self.send_message(ServerMessage::Log(msg)).await;
        StatusCode::CREATED
    }
}

impl TraktorServer {
    pub fn routes(
        state: &Arc<Mutex<Self>>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + use<> {
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
    ) -> impl Filter<Extract = (Arc<Mutex<Self>>,), Error = Infallible> + Clone {
        warp::any().map(move || state.clone())
    }

    fn is_started(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
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

    fn json_body<T: DeserializeOwned + Send>(
    ) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(64 * 1024).and(warp::body::json())
    }

    fn route_connect(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("connect")
            .and(Self::with_state(state))
            .then(async |state: Arc<Mutex<Self>>| state.lock().await.handle_connect().await)
    }

    fn route_init(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
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
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("update")
            .and(warp::post())
            .and(Self::with_state(state))
            .and(Self::route_update_sub_routes())
            .then(async |state: Arc<Mutex<Self>>, (session_id, update)| {
                state.lock().await.handle_update(session_id, update).await
            })
    }

    fn route_update_sub_routes(
    ) -> impl Filter<Extract = ((String, StateUpdate),), Error = warp::Rejection> + Clone {
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
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("cover").and(
            Self::route_cover_upload(state.clone()).or(Self::route_cover_socket(state.clone())),
        )
    }

    fn route_cover_upload(
        state: Arc<Mutex<Self>>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::post()
            .and(Self::with_state(state))
            .and(warp::body::content_length_limit(16 * 1024 * 1024).and(warp::body::bytes()))
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
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
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
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("log")
            .and(warp::post())
            .and(Self::with_state(state))
            .and(warp::body::content_length_limit(4 * 1024).and(warp::body::bytes()))
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
    let routes = TraktorServer::routes(&state);

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
    let addr_vec = if !addr.ip().is_unspecified() {[addr.ip()].to_vec()} else {Vec::new()};
    let responder = Responder::new_with_ip_list(addr_vec).expect("could not create responder");
    let svc = responder.register(
        "_http._tcp".to_owned(),
        "traktor-di-webserver".to_owned(),
        addr.port(),
        &["path=/"],
    );
    println!("advertising traktor server on {}", addr);
    svc
}

pub fn run_server(addr: SocketAddr) -> impl Stream<Item = ServerMessage> {
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
