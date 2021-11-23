/* =================================================================================
 File:           fbp_asyncstate.rs

 Description:    This file struct that can be used to wait on the outcome of
                 an asynchronous operation.

 History:        RustDev 03/31/2021   Code ported from original rustfbp crate

 Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
================================================================================== */

//! # An asynchronous state change monitor
//!
//! Given the asynchronous nature of FBP nodes there are times when code may
//! need to wait until an asynchronous state has changed.  The AsyncState state provides
//! for that need
//!
//! # Example
//!
//! ```
//! use futures::*;
//! use std::pin::Pin;
//! use std::task::{Context, Poll};
//! use std::sync::Arc;
//! use std::sync::atomic::{Ordering, AtomicBool};
//! use std::ops::{Deref};
//! use fbp::fbp_asyncstate::*;
//! use std::{thread, time};
//!
//! async fn asyncstate_ex() {
//! 	let my_asyncstate = AsyncState::new();
//!
//! 	assert_eq!(my_asyncstate.is_ready(), false);
//!
//! 	// Clone the async_state for use inside the thread
//! 	let clone_asyncsatate = my_asyncstate.clone();
//!
//! 	let jh =  thread::spawn(move || {
//! 		thread::sleep(time::Duration::from_secs(1));
//!
//! 		clone_asyncsatate.clone().set_is_ready(true);
//! 	});
//!
//! 	// A clone is needed because the await actually runs a
//! 	// thread that polls the state
//!
//! 	// This will block until the state is changed inside the thread
//! 	my_asyncstate.clone().await;
//!
//! 	assert_eq!(my_asyncstate.is_ready(), true);
//!
//! 	let _ = jh.join();
//! }
//!  
//! ```
//!
use futures::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

/// Provides the structure of an AsyncState
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncState {
    #[serde(skip)]
    value_is_ready: Arc<AtomicBool>,
}

impl Future for AsyncState {
    type Output = bool;

    // Wait until the state is ready.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_ready() {
            Poll::Ready(true)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl Default for AsyncState {
    fn default() -> Self {
        AsyncState::new()
    }
}

impl AsyncState {
    /// Creates a new AsyncState setting the default is_ready to false
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_asyncstate::*;
    /// use std::sync::atomic::{Ordering, AtomicBool};
    ///
    /// let an_asyncstate = AsyncState::new();
    ///
    /// ```
    ///
    pub fn new() -> Self {
        AsyncState {
            value_is_ready: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns if this asyncstate is ready.  Ready means that the item that this state is
    /// monitoring has completed
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_asyncstate::*;
    ///
    /// let an_asyncstate = AsyncState::new();
    /// if an_asyncstate.is_ready() {
    /// 	// The item has completed
    /// }
    ///
    /// ```
    ///
    pub fn is_ready(&self) -> bool {
        self.value_is_ready.deref().load(Ordering::Relaxed)
    }

    /// Set the value of an asyncstate.  This must be done in another thread from the thread that is
    /// awaiting for the state to change.
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_asyncstate::*;
    ///
    /// let an_asyncstate = AsyncState::new();
    /// an_asyncstate.set_is_ready(true);
    ///
    /// ```
    ///
    pub fn set_is_ready(&self, flag: bool) {
        self.value_is_ready.store(flag, Ordering::Relaxed)
    }
}

/* --------------------------------------------------------------------------
Unit Tests
------------------------------------------------------------------------- */
#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[actix_rt::test]

    async fn asyncstate() {
        let my_asyncstate = AsyncState::new();

        assert_eq!(my_asyncstate.is_ready(), false);

        // Clone the async_state for use inside the thread
        let clone_asyncsatate = my_asyncstate.clone();

        let jh = thread::spawn(move || {
            thread::sleep(time::Duration::from_secs(1));

            clone_asyncsatate.clone().set_is_ready(true);
        });

        // A clone is needed because the await actually runs a thread that polls the
        // state.
        my_asyncstate.clone().await;

        assert_eq!(my_asyncstate.is_ready(), true);

        let _ = jh.join();
    }
}
