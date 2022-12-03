use std::{
    cmp::Ordering,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::{ready, Stream};
use futures_util::{stream::Peekable, StreamExt};
use pin_project::pin_project;

#[pin_project]
struct Side<S>
where
    S: Stream,
{
    #[pin]
    stream: S,
    head: Option<S::Item>,
    terminated: bool,
}

struct Head<'a, T> {
    inner: &'a mut Option<T>,
}

impl<'a, T> Head<'a, T> {
    fn new(inner: &'a mut Option<T>) -> Self {
        debug_assert!(inner.is_some());
        Self { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner.take().unwrap()
    }
}

impl<T> AsRef<T> for Head<'_, T> {
    fn as_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }
}

impl<S: Stream> Side<S> {
    fn new(stream: S) -> Self {
        Self {
            stream,
            head: None,
            terminated: false,
        }
    }

    fn poll_peek(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Head<'_, S::Item>>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if this.head.is_some() {
                break Some(Head::new(this.head));
            } else if *this.terminated {
                break None;
            } else if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                *this.head = Some(item);
            } else {
                break None;
            }
        })
    }
}

#[pin_project]
pub struct MergeBy<L, R, F>
where
    L: Stream,
    R: Stream<Item = L::Item>,
    F: Fn(&L::Item, &R::Item) -> bool,
{
    #[pin]
    left: Side<L>,
    #[pin]
    right: Side<R>,
    compare: F,
}

impl<L, R, F> Stream for MergeBy<L, R, F>
where
    L: Stream,
    R: Stream<Item = L::Item>,
    F: Fn(&L::Item, &R::Item) -> bool,
{
    type Item = L::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        Poll::Ready(
            match (
                ready!(this.left.poll_peek(cx)),
                ready!(this.right.poll_peek(cx)),
            ) {
                (Some(l), Some(r)) => {
                    if (this.compare)(l.as_ref(), r.as_ref()) {
                        Some(l.into_inner())
                    } else {
                        Some(r.into_inner())
                    }
                }
                (Some(l), None) => Some(l.into_inner()),
                (None, Some(r)) => Some(r.into_inner()),
                (None, None) => return Poll::Ready(None),
            },
        )
    }
}

pub trait MergeStreamExt {
    /// Merge two streams.
    fn merge_by<R, F>(self, right: R, is_first: F) -> MergeBy<Self, R, F>
    where
        Self: Stream + Sized,
        R: Stream<Item = Self::Item>,
        F: Fn(&Self::Item, &R::Item) -> bool,
    {
        MergeBy {
            left: Side::new(self),
            right: Side::new(right),
            compare: is_first,
        }
    }
}

impl<T: ?Sized> MergeStreamExt for T where T: Stream {}

#[cfg(test)]
mod tests {
    use futures_util::{stream, StreamExt};

    use super::MergeStreamExt;

    #[tokio::test]
    async fn merge_by() {
        let fizz = stream::iter((0..10).step_by(3).map(|n| (n, 'f')));
        let buzz = stream::iter((0..10).step_by(5).map(|n| (n, 'b')));
        let merged = fizz.merge_by(buzz, |f, b| f.0 <= b.0);

        assert_eq!(
            merged.collect::<Vec<_>>().await,
            [(0, 'f'), (0, 'b'), (3, 'f'), (5, 'b'), (6, 'f'), (9, 'f'),]
        );
    }
}
