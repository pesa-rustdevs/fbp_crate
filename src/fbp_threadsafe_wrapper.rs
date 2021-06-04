/* ==========================================================================
 File:           fbp_threadsafe_wrapper.rs

 Description:    This file provides the means for wrapping a type so that
                 it will 'work' when used with threads.  Because of the
                 Rust borrow checker, most of the time, a type needs to
                 be cloned to work within a thread as the thread will take
                 ownership of the type.  This can cause issues when trying
                 to have the same data be at least readable between threads.
                 What is needed is a way to allow for atomic changes to a
                 type and have those changes be viewable in multiple threads.

                 This is what the ThreadSafeWrapper does.  It is a generic
                 type so that this will work with ANY type.


 History:        RustDev 04/05/2021   Ported from the rustfbpf project

 Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
========================================================================== */

//! A thread safe wrapper for a type.
//!
//! This wrapper will ensure that all references point to the same underlying data
//! even between threads.  Access is atomic as a Mutex is used and because the Arc
//! ensures that all references point to the same underlying data.  This is a helpful
//! type for an FBP node as FBP nodes run independent threads for their input queue and
//! message processing.
//!

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, MutexGuard};

/// Generic Thread safe type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadSafeType<T: Clone> {
    data: Arc<Mutex<T>>,
}

impl<T: Clone> ThreadSafeType<T> {
    /// Creates a new ThreadSafeType for a specific type
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    ///
    /// let safe_bool = ThreadSafeType::new(false);
    ///
    /// ```
    ///
    pub fn new(wrapped_type: T) -> Self {
        ThreadSafeType {
            data: Arc::new(Mutex::new(wrapped_type)),
        }
    }

    /// Create a new ThreadSafeType from an Arc<Mutex<T>>
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    ///
    /// let an_arc = Arc::new(Mutex::new(false));
    ///
    /// let safe_bool = ThreadSafeType::create(an_arc);
    ///
    /// ```
    ///
    pub fn create(an_arc: Arc<Mutex<T>>) -> Self {
        ThreadSafeType { data: an_arc }
    }

    /// Return a MutexGuard<T> of the underlying data
    ///
    /// NOTE: MutexGuard<T> will dereference to the underlying type
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::ops::Deref;
    ///
    /// let safe_bool = ThreadSafeType::new(false);
    /// let the_value = safe_bool.get_type();
    /// if *the_value.deref() == true {
    /// 	println!("The value is true");
    /// } else {
    /// 	println!("The value is false");
    /// }
    ///
    /// ```
    ///
    pub fn get_type(&self) -> MutexGuard<T> {
        self.data.lock().unwrap()
    }

    /// Return the underlying arc used by the ThreadSafeType
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    ///
    /// let safe_bool = ThreadSafeType::new(false);
    /// let the_arc = safe_bool.get_arc();
    ///
    /// ```
    ///
    pub fn get_arc(&self) -> Arc<Mutex<T>> {
        self.data.clone()
    }

    /// Change the underlying arc used by the threadSafeType
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    ///
    /// let mut safe_bool = ThreadSafeType::new(false);
    /// let new_arc = Arc::new(Mutex::new(true));
    /// safe_bool.set_arc(new_arc);
    ///
    /// ```
    ///
    pub fn set_arc(&mut self, new_arc: Arc<Mutex<T>>) {
        // typically used as follows self.set_arc(other_threadSafeType.get_arc());
        self.data = Arc::clone(&new_arc);
    }
}

/// Generic thread safe Option
#[derive(Debug)]
pub struct ThreadSafeOptionType<T> {
    data: Arc<Mutex<Option<T>>>,
}

impl<T> Clone for ThreadSafeOptionType<T> {
    fn clone(&self) -> Self {
        let the_arc = self.get_arc();
        ThreadSafeOptionType {
            data: Arc::clone(&the_arc),
        }
    }
}

impl<T> Default for ThreadSafeOptionType<T> {
    fn default() -> Self {
        ThreadSafeOptionType::new(None)
    }
}

impl<T> ThreadSafeOptionType<T> {
    /// Creates a new ThreadSafeOptionType for a specific type
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    ///
    /// let safe_bool: ThreadSafeOptionType<bool> = ThreadSafeOptionType::new(None);
    ///
    /// ```
    ///
    pub fn new(option_to_wrap: Option<T>) -> Self {
        ThreadSafeOptionType {
            data: Arc::new(Mutex::new(option_to_wrap)),
        }
    }

    /// Create a new ThreadSafeOptionType from an Arc<Mutex<Option<T>>>
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    ///
    /// let an_arc = Arc::new(Mutex::new(Some(false)));
    ///
    /// let safe_bool_option = ThreadSafeType::create(an_arc);
    ///
    /// ```
    ///
    pub fn create(op_arc: Arc<Mutex<Option<T>>>) -> Self {
        ThreadSafeOptionType { data: op_arc }
    }

    /// Return the underlying arc used by the ThreadSafeOptionType
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    ///
    /// let safe_bool_option = ThreadSafeOptionType::new(Some(false));
    /// let the_arc = safe_bool_option.get_arc();
    ///
    /// ```
    ///
    pub fn get_arc(&self) -> Arc<Mutex<Option<T>>> {
        self.data.clone()
    }

    /// Change the underlying arc used by the ThreadSafeOptionType
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    ///
    /// let mut safe_bool_option = ThreadSafeOptionType::new(Some(false));
    /// let new_arc = Arc::new(Mutex::new(Some(true)));
    /// safe_bool_option.set_arc(new_arc);
    ///
    /// ```
    ///
    pub fn set_arc(&mut self, new_arc: Arc<Mutex<Option<T>>>) {
        self.data = Arc::clone(&new_arc);
    }

    /// Change the underlying option in a MutexGuard
    ///
    /// NOTE: MutexGuard<Option<T>> will dereference to the underlying Option
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    /// use std::ops::Deref;
    ///
    /// let mut safe_bool_option = ThreadSafeOptionType::new(Some(false));
    /// let the_value = safe_bool_option.get_option();
    /// if the_value.deref().unwrap() == true {
    /// 	println!("The value is true");
    /// } else {
    /// 	println!("The value is false");
    /// }
    ///
    /// ```
    ///
    pub fn get_option(&self) -> MutexGuard<Option<T>> {
        self.data.lock().unwrap()
    }

    /// Replace the underlying option in a ThreadSafeOptionType with a new option
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// use std::sync::{Arc, Mutex, MutexGuard};
    /// use std::ops::{DerefMut, Deref};
    ///
    /// let mut safe_bool_option = ThreadSafeOptionType::new(None);
    /// let new_option = Some(true);
    /// safe_bool_option.set_option(new_option);
    ///
    pub fn set_option(&mut self, new_option: Option<T>) {
        *self.data.lock().as_deref_mut().unwrap() = new_option;
    }

    /// Implement is_none for a ThreadSafeOptionType so that it works like a regular option
    ///
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    ///
    /// let mut safe_bool_option: ThreadSafeOptionType<bool>  = ThreadSafeOptionType::new(None);
    /// assert!(safe_bool_option.is_none());
    ///
    /// ```
    pub fn is_none(&self) -> bool {
        self.get_option().is_none()
    }

    /// Implement is_some for a ThreadSafeOptionType so that it works like a regular option
    ///
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_threadsafe_wrapper::*;
    /// let mut safe_bool_option = ThreadSafeOptionType::new(Some(true));
    /// assert!(safe_bool_option.is_some());
    ///
    /// ```
    pub fn is_some(&self) -> bool {
        self.get_option().is_some()
    }
}
