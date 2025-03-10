use crate::async_utils::DroppingOnce;
use crate::traktor_api::model::{AppMessage, ServerMessage};
use iced::futures::channel::{mpsc, oneshot};
use iced::futures::stream;
use iced::futures::{SinkExt, Stream, StreamExt};
use std::sync::{Arc, Mutex};
use warp::Filter;

pub fn run_server(port: u16) -> impl Stream<Item = ServerMessage> {
    let (mut output, output_receive) = mpsc::unbounded();
    let (cancel, cancelled) = oneshot::channel();

    let runner = DroppingOnce::new(
        async move {
            let (input_send, mut input) = mpsc::unbounded();
            let _ = output.send(ServerMessage::Ready(input_send)).await;

            let test = Arc::new(Mutex::new(false));
            let test_clone = Arc::clone(&test);

            let hello = warp::path!("hello" / String)
                .map(move |name| format!("Hello, {}! {}", name, test_clone.lock().unwrap()));

            println!("starting traktor server on port {}", port);
            let Ok((_, fut)) =
                warp::serve(hello).try_bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
                    cancelled.await.ok();
                })
            else {
                println!("could not start traktor server on port {}", port);
                return;
            };
            tokio::task::spawn(fut);

            loop {
                match input.select_next_some().await {
                    AppMessage::Reconnect { debug_logging } => {
                        *test.lock().unwrap() = debug_logging;
                    }
                }
            }
        },
        move || {
            println!("stopping traktor server on port {}", port);
            let _ = cancel.send(());
        },
    )
    .filter_map(|_| async { None });

    stream::select(output_receive, runner)
}
