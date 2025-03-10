use crate::traktor_api::model::{AppMessage, ServerMessage};
use iced::futures::channel::{mpsc, oneshot};
use iced::futures::stream::FusedStream;
use iced::futures::{ready, stream};
use iced::futures::{SinkExt, Stream, StreamExt};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use warp::Filter;

pin_project! {
    struct DroppingOnce<Fut, DropFn>
    where
        Fut: Future,
        DropFn: FnOnce() -> (),
    {
        #[pin]
        future: Option<Fut>,
        drop_fn: Option<DropFn>,
    }

    impl<Fut: Future, DropFn: FnOnce() -> ()> PinnedDrop for DroppingOnce<Fut, DropFn> {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();
            if let Some(drop_fn) = this.drop_fn.take() {
                drop_fn();
            }
        }
    }
}

impl<Fut: Future, DropFn: FnOnce() -> ()> DroppingOnce<Fut, DropFn> {
    pub fn new(future: Fut, drop_fn: DropFn) -> Self {
        Self {
            future: Some(future),
            drop_fn: Some(drop_fn),
        }
    }
}

impl<Fut: Future, DropFn: FnOnce() -> ()> Stream for DroppingOnce<Fut, DropFn> {
    type Item = Fut::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let v = match this.future.as_mut().as_pin_mut() {
            Some(fut) => ready!(fut.poll(cx)),
            None => return Poll::Ready(None),
        };

        this.future.set(None);
        Poll::Ready(Some(v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.future.is_some() {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<Fut: Future, DropFn: FnOnce() -> ()> FusedStream for DroppingOnce<Fut, DropFn> {
    fn is_terminated(&self) -> bool {
        self.future.is_none()
    }
}

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

            println!("starting server on port {}", port);
            let Ok((_, fut)) =
                warp::serve(hello).try_bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
                    cancelled.await.ok();
                })
            else {
                println!("starting server on port {} FAILED", port);
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
            println!("ending server on port {}", port);
            let _ = cancel.send(());
        },
    )
    .filter_map(|_| async { None });

    stream::select(output_receive, runner)
}
