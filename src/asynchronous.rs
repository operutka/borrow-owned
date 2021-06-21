use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

use futures_channel::oneshot::{self, Receiver, Sender};

/// Trait allowing objects to be temporarily moved elsewhere.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub trait AsyncBorrowOwned: Sized {
    /// Borrow this object.
    ///
    /// The method returns an [`AsyncBorrowed`](AsyncBorrowed) wrapper which
    /// can be easily moved elsewhere (e.g. a separate thread) and an
    /// [`AsyncReturn`](AsyncReturn) handle which can be used to wait until the
    /// borrow ends.
    fn async_borrow_owned(self) -> (AsyncBorrowed<Self>, AsyncReturn<Self>);
}

impl<T> AsyncBorrowOwned for T
where
    T: Sized,
{
    fn async_borrow_owned(self) -> (AsyncBorrowed<Self>, AsyncReturn<Self>) {
        let (tx, rx) = oneshot::channel();

        let borrowed = AsyncBorrowed {
            inner: Some((self, tx)),
        };
        let ret = AsyncReturn { rx };

        (borrowed, ret)
    }
}

/// Return handle.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub struct AsyncReturn<T> {
    rx: Receiver<T>,
}

impl<T> Future for AsyncReturn<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rx = Pin::new(&mut self.rx);

        if let Poll::Ready(res) = rx.poll(cx) {
            Poll::Ready(res.unwrap())
        } else {
            Poll::Pending
        }
    }
}

/// Borrowed object.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub struct AsyncBorrowed<T> {
    inner: Option<(T, Sender<T>)>,
}

impl<T> Drop for AsyncBorrowed<T> {
    fn drop(&mut self) {
        if let Some((val, tx)) = self.inner.take() {
            tx.send(val).unwrap_or_default();
        }
    }
}

impl<T> Deref for AsyncBorrowed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().map(|(val, _)| val).unwrap()
    }
}

impl<T> DerefMut for AsyncBorrowed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().map(|(val, _)| val).unwrap()
    }
}
