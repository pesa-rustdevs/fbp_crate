/* =================================================================================
 File:           fbp_node_error.rs

 Description:    This file contains an Error type for FBP nodes


 History:        RustDev 03/31/2021   Code ported from original rustfbp crate

 Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
================================================================================== */

//! # Specific Error Type for FBP nodes
//!
//! Many of the methods for FBP nodes will return a Result.  Some of those Results will
//! contain an error.  The Error type defined here will provide that Error type.
//!

use std::error::Error;
use std::fmt;

/// # FBP Error Type
///
/// An FBP specific Error type
///
#[derive(Debug)]
pub struct NodeError {
    details: String,
}

impl NodeError {
    /// Creates a new NodeError
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_error::*;
    ///
    /// let a_fbp_error= NodeError::new("Some error message");
    ///
    /// ```
    ///
    pub fn new(msg: &str) -> NodeError {
        NodeError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for NodeError {
    fn description(&self) -> &str {
        &self.details
    }
}
