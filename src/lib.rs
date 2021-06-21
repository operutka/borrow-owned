//! Simple helper for temporary object moving.
//!
//! It is useful if you need to pass your object somewhere and you cannot pass
//! a reference to the object because you wouldn't be able to guarantee that
//! the object lives long enough. For example, you don't want to use
//! `Arc<Mutex<_>>` for some reason but you need to pass your object to a new
//! thread and you need to use the object again in the original thread once the
//! new thread does not need the object anymore.
//!
//! See the examples below.
//!
//! # Features
//! * `async` - asynchronous version of the helper
//!
//! # Example
//! ```
//! use std::{mem, thread, time::Duration};
//! use borrow_owned::BorrowOwned;
//!
//! struct MyStruct;
//!
//! let object = MyStruct;
//! let (borrowed, ret) = object.borrow_owned();
//!
//! thread::spawn(move || {
//!     println!("doing something with the object...");
//!     thread::sleep(Duration::from_millis(100));
//!
//!     mem::drop(borrowed);
//!
//!     println!("doing something else...");
//!     thread::sleep(Duration::from_millis(100));
//! });
//!
//! println!("waiting until the borrow ends...");
//! let objects = ret.wait();
//! println!("the object is back again!");
//! ```
//!
//! # Asynchronous example
//! ```
//! use std::{mem, time::Duration};
//! use borrow_owned::AsyncBorrowOwned;
//!
//! struct MyStruct;
//!
//! #[tokio::main]
//! async fn main() {
//!     let object = MyStruct;
//!     let (borrowed, ret) = object.async_borrow_owned();
//!
//!     tokio::spawn(async move {
//!         println!("doing something with the object...");
//!         tokio::time::sleep(Duration::from_millis(100));
//!
//!         mem::drop(borrowed);
//!
//!         println!("doing something else...");
//!         tokio::time::sleep(Duration::from_millis(100));
//!     });
//!
//!     println!("waiting until the borrow ends...");
//!     let object = ret.await;
//!     println!("the object is back again!");
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async")]
mod asynchronous;

use std::{
    ops::{Deref, DerefMut},
    sync::mpsc::{self, Receiver, Sender},
};

#[cfg(feature = "async")]
pub use crate::asynchronous::{AsyncBorrowOwned, AsyncBorrowed, AsyncReturn};

/// Trait allowing objects to be temporarily moved elsewhere.
pub trait BorrowOwned: Sized {
    /// Borrow this object.
    ///
    /// The method returns a [`Borrowed`](Borrowed) wrapper which can be easily
    /// moved elsewhere (e.g. a separate thread) and a [`Return`](Return)
    /// handle which can be used to wait until the borrow ends.
    fn borrow_owned(self) -> (Borrowed<Self>, Return<Self>);
}

impl<T> BorrowOwned for T
where
    T: Sized,
{
    fn borrow_owned(self) -> (Borrowed<Self>, Return<Self>) {
        let (tx, rx) = mpsc::channel();

        let borrowed = Borrowed {
            tx,
            val: Some(self),
        };
        let ret = Return { rx };

        (borrowed, ret)
    }
}

/// Return handle.
pub struct Return<T> {
    rx: Receiver<T>,
}

impl<T> Return<T> {
    /// Wait until the corresponding borrow ends and return the borrowed
    /// object.
    pub fn wait(self) -> T {
        self.rx.recv().unwrap()
    }
}

/// Borrowed object.
pub struct Borrowed<T> {
    tx: Sender<T>,
    val: Option<T>,
}

impl<T> Drop for Borrowed<T> {
    fn drop(&mut self) {
        if let Some(val) = self.val.take() {
            self.tx.send(val).unwrap_or_default();
        }
    }
}

impl<T> Deref for Borrowed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val.as_ref().unwrap()
    }
}

impl<T> DerefMut for Borrowed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.val.as_mut().unwrap()
    }
}
