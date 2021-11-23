/* ==========================================================================
 File:          fbp_wait_for_payload.rs

 Description:   This file provides the means for receiving an input message
                an allowing code to wait until a payload has been received.
                This can come in handle when sending a message to another 
                node and the result of that message is needed in order to
                continue processing.


 History:        RustDev 08/23/2021   New functionality

 Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
========================================================================== */

//! A FBP node that will allow for waiting on the reception of a message
//!
//! Somethimes especially when a TCP message is sent to another node, the 
//! sending node may need to get the results of that send in order to be
//! able to continue processing.  The WaitForPayloadNode allows for this.
//! 
//! 

use async_trait::async_trait;
use std::ops::{Deref};


use crate::fbp_iidmessage::*;
use crate::fbp_node_context::*;
use crate::fbp_node_error::*;
use crate::fbp_node_trait::*;
use crate::fbp_threadsafe_wrapper::*;
use crate::fbp_asyncstate::*;


// Define a type that will wait for a payload to be updated.
/// This can be useful when wanting to get the output of a node
/// within that node itself.  One make this node a receiver of 
/// the node whose output is desired and then have another node
/// call the get_payload async method which will wait for the 
/// payload to arrive.
///
/// /// # Example
///
/// Basic usage:
/// ```
/// use async_trait::async_trait;
/// use std::ops::{Deref};
/// use serde::{Deserialize, Serialize};
///
/// use fbp::fbp_iidmessage::*;
/// use fbp::fbp_node_context::*;
/// use fbp::fbp_node_trait::*;
/// use fbp::fbp_wait_for_payload::*;
/// use fbp::fbp_node_error::*;
///
/// #[derive(Clone, Serialize, Deserialize)]
///     pub struct PassthroughNode {
///         data: Box<FBPNodeContext>,
///     }
///
///     impl PassthroughNode {
///         #[allow(dead_code)]
///         pub fn new() -> Self {
///             let result = PassthroughNode {
///                 data: Box::new(FBPNodeContext::new("PassthroughNode")),
///             };
///
///             result.node_data().set_node_is_configured(true);
///             result.clone().start();
///             result
///         }
///     }
///
///     #[async_trait]
///     impl FBPNodeTrait for PassthroughNode {
///         fn node_data_clone(&self) -> FBPNodeContext {
///             self.data.deref().clone()
///         }
///
///         fn node_data(&self) -> &FBPNodeContext {
///             &self.data
///         }
///
///         fn node_data_mut(&mut self) -> &mut FBPNodeContext {
///             &mut self.data
///         }
///
///         fn process_message(
///             &mut self,
///             msg: IIDMessage,
///         ) -> std::result::Result<IIDMessage, NodeError> {
///
///             Ok(msg.clone())
///         }
///     }
///
///     let mut pt_node = PassthroughNode::new();
///     let mut wait_node = WaitForPayloadNode::new();
///
///     pt_node.node_data_mut().add_receiver(wait_node.node_data_mut(), None);
///
///     let a_msg = IIDMessage::new(MessageType::Data, Some("This is a payload".to_string()));
///     pt_node.node_data().post_msg(a_msg);
///     let mut rt = tokio::runtime::Builder::new()
///             .enable_all()
///             .build()
///             .unwrap();
///
///     let mut payload: String = String::new();
///
///      rt.block_on(async {
///         payload = wait_node.get_payload().await;
///     });
///
/// ```
#[derive(Clone)]
pub struct WaitForPayloadNode {
    data: Box<FBPNodeContext>,
    pub wait_for_payload: AsyncState,
    pub payload: ThreadSafeOptionType<String>,
}

impl WaitForPayloadNode {
    /// Create a new WaitForPayloadNode.
    /// 
    /// This node wil wait on a payload that is sent to it as an
    /// IIDMessage.  
    pub fn new() -> Self {
        let result = WaitForPayloadNode {
            data: Box::new(FBPNodeContext::new("WaitForPayloadNode")),
            wait_for_payload: AsyncState::new(),
            payload: ThreadSafeOptionType::new(None),
        };

        result.node_data().set_node_is_configured(true);
        result.clone().start();
        result
    }

    ///  Get the payload sent to this node.  This is an async method
    /// that will wait for an IIDMesssage to be sent to this node and
    /// will return the the payload of that message when it arrives.
    #[allow(dead_code)]
    pub async fn get_payload(&self) -> String {
        if self.payload.is_some() {
            return self.payload.get_option().as_ref().unwrap().clone();
        }
        // Wait for data
        let wait_ref = self.wait_for_payload.clone();
        wait_ref.await;

        if self.payload.is_some() {
            return self.payload.get_option().as_ref().unwrap().clone();
        }

        // Just in case :-)
        String::new()
    }
}

#[async_trait]
impl FBPNodeTrait for WaitForPayloadNode {
    fn node_data_clone(&self) -> FBPNodeContext {
        self.data.deref().clone()
    }

    fn node_data(&self) -> &FBPNodeContext { &self.data }

    fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }

    // When an IIDMessage is received, it will be set into the payload field and
    // the AsyncState will be signaled.
    fn process_message(&mut self, msg: IIDMessage) -> Result<IIDMessage, NodeError> {
            
        if msg.payload().is_some() {
            let the_payload = msg.payload().as_ref().unwrap().clone();
            self.payload.set_option(Some(the_payload));
            self.wait_for_payload.set_is_ready(true);
        }
        Ok(msg.clone())
    }
}
