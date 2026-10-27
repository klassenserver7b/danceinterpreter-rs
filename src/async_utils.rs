use iced::futures::stream::FusedStream;
use iced::futures::{Stream, ready};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    pub struct DroppingOnce<Fut, DropFn>
    where
        Fut: Future,
        DropFn: FnOnce(),
    {
        #[pin]
        future: Option<Fut>,
        drop_fn: Option<DropFn>,
    }

    impl<Fut: Future, DropFn: FnOnce()> PinnedDrop for DroppingOnce<Fut, DropFn> {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();
            if let Some(drop_fn) = this.drop_fn.take() {
                drop_fn();
            }
        }
    }
}

impl<Fut: Future, DropFn: FnOnce()> DroppingOnce<Fut, DropFn> {
    pub fn new(future: Fut, drop_fn: DropFn) -> Self {
        Self {
            future: Some(future),
            drop_fn: Some(drop_fn),
        }
    }
}

impl<Fut: Future, DropFn: FnOnce()> Stream for DroppingOnce<Fut, DropFn> {
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

impl<Fut: Future, DropFn: FnOnce()> FusedStream for DroppingOnce<Fut, DropFn> {
    fn is_terminated(&self) -> bool {
        self.future.is_none()
    }
}
