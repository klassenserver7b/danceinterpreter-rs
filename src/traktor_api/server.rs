use crate::async_utils::DroppingOnce;
use crate::traktor_api::model::{
    AppMessage, ConnectionResponse, InitializeRequest, ServerMessage, UpdateRequest,
};
use crate::traktor_api::{StateUpdate, ID};
use bytes::Bytes;
use iced::futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use iced::futures::channel::{mpsc, oneshot};
use iced::futures::stream;
use iced::futures::{SinkExt, Stream, StreamExt};
use serde::de::DeserializeOwned;
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
}

impl TraktorServer {
    pub fn new(output: UnboundedSender<ServerMessage>) -> Self {
        TraktorServer {
            output,

            debug_logging: false,
            session_id: "".to_owned(),

            is_initialized: false,
            queue: Vec::new(),
        }
    }

    async fn send_message(&mut self, message: ServerMessage) {
        let _ = self.output.send(message).await;
    }

    async fn send_messages(&mut self, messages: impl IntoIterator<Item = ServerMessage>) {
        let _ = self
            .output
            .send_all(&mut stream::iter(messages).map(|msg| Ok(msg)))
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

    async fn handle_connect(&mut self) -> impl warp::Reply {
        warp::reply::json(&ConnectionResponse {
            session_id: self.session_id.to_owned(),
            debug_logging: self.debug_logging,
        })
    }

    async fn handle_init(&mut self, request: InitializeRequest) -> impl warp::Reply {
        if request.session_id == self.session_id {
            let time_offset_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| (request.timestamp as i64) - (d.as_millis() as i64))
                .unwrap_or(0);

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

    async fn handle_update(&mut self, session_id: String, update: StateUpdate) -> impl warp::Reply {
        if session_id == self.session_id {
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

    async fn handle_log(&mut self, msg: String) -> impl warp::Reply {
        self.send_message(ServerMessage::Log(msg)).await;
        StatusCode::CREATED
    }
}

impl TraktorServer {
    pub fn routes(
        state: &Arc<Mutex<Self>>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        Self::is_started(state.clone())
            .and(
                Self::route_connect(state.clone())
                    .or(Self::route_init(state.clone()))
                    .or(Self::route_update(state.clone()))
                    .or(Self::route_log(state.clone())),
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
        (warp::path!("mixer")
            .and(Self::json_body())
            .then(async move |req: UpdateRequest<_>| {
                (req.session_id, StateUpdate::Mixer(req.state))
            }))
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
                    .handle_log(String::from_utf8_lossy(&*body).into_owned())
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
    let Ok((_, fut)) = warp::serve(routes).try_bind_with_graceful_shutdown(addr, async {
        cancelled.await.ok();
    }) else {
        println!("could not start traktor server on {}", addr);
        return;
    };
    tokio::task::spawn(fut);

    state.lock().await.send_ready(input_send).await;
    loop {
        match input.select_next_some().await {
            AppMessage::Reconnect { debug_logging } => state.lock().await.reconnect(debug_logging),
        }
    }
}

pub fn run_server(addr: SocketAddr) -> impl Stream<Item = ServerMessage> {
    let (output, output_receive) = mpsc::unbounded();
    let (input_send, input) = mpsc::unbounded();
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
