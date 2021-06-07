/* =================================================================================
 File:           fbp_node_context.rs

 Description:    This file struct defines the basic requirements of an FBP node.
                 An FBP node provides a


 History:        RustDev 03/31/2021   Code ported from original rustfbp crate

================================================================================== */

//! # Required data needed for all FBP Nodes
//!
//! A Flow Based Programming node, provides for a unique set of processing to be done on
//! incoming messages (data) that is run in an asynchronous thread.  This thread has an
//! input queue and a vector or output queues.  The input queue holds the work items
//! that a node will process in a FIFO order. The vector of output queues allows for
//! multiple other nodes to receive the output from a node.  
//!
//! The FBPNodeContext struct provides all of the necessary items for a FBP node to
//! operate.  A specific instance of a FBP node must have a Box\<FBPNodeContext\> as one
//! of its fields.  This ensures that the specific instance can work as an FBP node.
//!
//! # Example
//!
//! ```
//!
//! use crate::fbp::fbp_node_context::*;
//! use fbp::fbp_threadsafe_wrapper::*;
//!
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Serialize, Deserialize)]
//! pub struct ExampleFBPNode {
//!     data: Box<FBPNodeContext>,
//! }
//!
//! ```
//!

use std::sync::mpsc::{channel, RecvError};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

use crate::fbp_asyncstate::*;
use crate::fbp_iidmessage::*;
use crate::fbp_threadsafe_wrapper::*;

/// # SenderWrapper
///
/// The FBP system uses std::sync::mpsc to send IIDMessages between FBP nodes.
/// All of the items in the FBPNodeContext are only needed at runtime and are
/// created when the FBPNodeContext is created.  This means that most of the
/// fields in the FBPNodeContext need to be #[serde(skip)] as the FBPNodeContext
/// does need to be Serialized for at least the name field as it is the name of
/// the "owning" FBPNode.  This is used to allow for creating a network of nodes
/// from a JSON string.  Using #[serde(skip)] requires that at least the Default
/// trait needs to be implemented. The std::sync::mpsc structs do not implement
/// the Default trait.  One cannot implement the Default trait for a type that was
/// not defined in the module.  This issue can be solved by using the
/// [New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
///
/// The SenderWrapper type _wraps_ a std::sync::mpsc::Sender struct so that the
/// Derive and Clone traits can be implemented.
pub struct SenderWrapper(std::sync::mpsc::Sender<IIDMessage>);

impl Deref for SenderWrapper {
    type Target = std::sync::mpsc::Sender<IIDMessage>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for SenderWrapper {
    fn default() -> Self {
        let (sender, _) = channel::<IIDMessage>();
        SenderWrapper(sender)
    }
}

impl Clone for SenderWrapper {
    fn clone(&self) -> Self {
        let sender = self.deref().clone();
        SenderWrapper(sender)
    }
}

/// # FBPNodeSender
///
/// The FBPNodeSender Wraps a SenderWrapper in a ThreadSafeType so that
/// multiple threads will be able to access the underlying
/// std::sync::mpsc::Sender<IIDMessage>
#[derive(Clone, Serialize, Deserialize)]
pub struct FBPNodeSender {
    #[serde(skip)]
    sender: ThreadSafeType<SenderWrapper>,
}

impl FBPNodeSender {
    /// Creates a new FBPNodeSender
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    ///
    /// use std::sync::mpsc::{channel};
    ///
    /// use fbp::fbp_node_context::*;
    /// use fbp::fbp_iidmessage::*;
    ///
    /// let (sender, _) = channel::<IIDMessage>();
    /// let node_sender = FBPNodeSender::new(SenderWrapper(sender));
    ///
    pub fn new(sender: SenderWrapper) -> Self {
        FBPNodeSender {
            sender: ThreadSafeType::new(sender),
        }
    }

    /// Send an IIDMessage to a Node
    ///
    /// Basic usage:
    /// use std::sync::mpsc::{channel};
    ///
    /// use fbp::fbp_node_context::*;
    /// use fbp::fbp_iidmessage::*;
    ///
    ///
    /// let (sender, _) = channel::<IIDMessage>();
    /// let node_sender = FBPNodeSender::new(SenderWrapper(sender));
    /// let msg = IIDMessage::new(MessageType::Data, Some("This is the message payload".to_string()));
    /// node_sender.send(msg);
    ///
    pub fn send(&self, msg: IIDMessage) {
        let send_result = self.sender.get_arc().lock().unwrap().deref().send(msg);
        if send_result.is_err() {
            // TODO Log error
        }
    }
}

impl Default for FBPNodeSender {
    fn default() -> Self {
        let (sender, _) = channel::<IIDMessage>();
        FBPNodeSender::new(SenderWrapper(sender))
    }
}

/// # ReceiverWrapper
///
/// The ReceiverWrapper type _wraps_ a std::sync::mpsc::Receiver struct so that the
/// Derive and Clone traits can be implemented.
///
/// Please see the SenderWrapper documentation on the need for wrapping the
/// std::sync::mpsc::Receiver struct
///
/// While the Clone trait is implemented for this struct, it is **not** a real
/// implementation.  It is required for the typesystem but given that the
/// underlying std::sync::mpsc::Receiver struct does **not** implement Clone
/// the implementation is the best that can be done.
pub struct ReceiverWrapper(std::sync::mpsc::Receiver<IIDMessage>);

impl Deref for ReceiverWrapper {
    type Target = std::sync::mpsc::Receiver<IIDMessage>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ReceiverWrapper {
    fn default() -> Self {
        let (_, receiver) = channel::<IIDMessage>();
        ReceiverWrapper(receiver)
    }
}

impl Clone for ReceiverWrapper {
    fn clone(&self) -> Self {
        ReceiverWrapper::default()
    }
}

/// # FBPNodeReceiver
///
/// The FBPNodeReceiver Wraps a ReceiverWrapper in a ThreadSafeType so that
/// multiple threads will be able to access the underlying
/// std::sync::mpsc::Receiver<IIDMessage>
#[derive(Serialize, Deserialize, Clone)]
pub struct FBPNodeReceiver {
    #[serde(skip)]
    pub receiver: ThreadSafeType<ReceiverWrapper>,
}

impl FBPNodeReceiver {
    /// Create a new FBPNodeReceiver
    ///
    pub fn new(receiver: ReceiverWrapper) -> Self {
        FBPNodeReceiver {
            receiver: ThreadSafeType::new(receiver),
        }
    }

    /// Call the receiver and return the result Result<IIDMessage, RecvError>
    ///
    pub fn recv(&self) -> Result<IIDMessage, RecvError> {
        self.receiver.get_arc().lock().unwrap().deref().recv()
    }
}

impl Default for FBPNodeReceiver {
    fn default() -> Self {
        let (_, receiver) = channel::<IIDMessage>();
        FBPNodeReceiver::new(ReceiverWrapper(receiver))
    }
}

/// Serializer trait for FBP Nodes
///
/// This trait will allow for serializing an FBP node into a JSON string and subsequently
/// take the JSON string from a serialized FBP node and reconstitute the node.
///
/// # Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use async_trait::async_trait;
/// use std::any::Any;
/// use std::ops::{Deref, DerefMut};
///
/// use fbp::fbp_node_context::*;
/// use fbp::fbp_node_error::*;
/// use fbp::fbp_iidmessage::*;
/// use fbp::fbp_node_trait::*;
/// use fbp::fbp_threadsafe_wrapper::*;
///
///
/// #[derive(Clone, Serialize, Deserialize)]
/// pub struct ExampleFBPNode {
///     data: Box<FBPNodeContext>,
/// }
///
/// #[async_trait]
/// impl FBPNodeTrait for ExampleFBPNode {
///
///    fn node_data_clone(&self) -> FBPNodeContext {
///        self.data.deref().clone()
///    }
///
///    fn node_data(&self) -> &FBPNodeContext { &self.data }
///
///    fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
///
///    fn process_message(&mut self,
///         msg: IIDMessage) -> Result<IIDMessage, NodeError> { Ok(msg.clone()) }
///
///    fn node_is_configured(&self) -> bool { self.node_data().node_is_configured() }
/// }
///
/// impl NodeSerializer for ExampleFBPNode {}
///
/// impl ExampleFBPNode {
///     pub fn new() -> Self {
///        let result =  ExampleFBPNode {
///             data: Box::new(FBPNodeContext::new("ExampleFBPNode")),
///         };
///     
///         result.data.set_node_is_configured(true);
///         result.clone().start();
///         result
///     }
/// }
///
/// let ex_node = ExampleFBPNode::new();
/// let serialized_pt_node = ex_node.serialize_node();
///
/// ```
pub trait NodeSerializer {
    /// This will deserialize a JSON string that is a serialized FBP node back into an FBP Node struct
    fn make_self_from_string<'a, T>(json_string: &'a str) -> T
    where
        T: std::marker::Sized + serde::Deserialize<'a>,
    {
        serde_json::from_str(json_string).unwrap()
    }

    /// This will take an FBP node and serialize that node into a JSON string
    fn serialize_node(&self) -> String
    where
        Self: std::marker::Sized + serde::Serialize,
    {
        serde_json::to_string(&self).unwrap()
    }
}

/// # FBP Node Context
///
/// The fields in this struct are all of the required data for a Flow Based Programming
/// node.
///
#[derive(Clone, Serialize, Deserialize)]
pub struct FBPNodeContext {
    name: String,
    #[serde(skip)]
    uuid: Uuid,
    #[serde(skip)]
    tx: Box<FBPNodeSender>,
    #[serde(skip)]
    rx: Box<FBPNodeReceiver>,
    #[serde(skip)]
    pub output_vec: ThreadSafeType<HashMap<String, Vec<Box<FBPNodeContext>>>>,
    #[serde(skip)]
    pub is_configured: AsyncState,
    #[serde(skip)]
    pub is_running: AsyncState,
    #[serde(skip)]
    pub node_completion: AsyncState,
    #[serde(skip)]
    node_suspended: Arc<AtomicBool>,
}

impl FBPNodeContext {
    /// Create a new FBPNodeContext
    pub fn new(name: &str) -> Self {
        let (sender, receiver) = channel::<IIDMessage>();

        FBPNodeContext {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            tx: Box::new(FBPNodeSender::new(SenderWrapper(sender))),
            rx: Box::new(FBPNodeReceiver::new(ReceiverWrapper(receiver))),
            output_vec: ThreadSafeType::new(HashMap::new()),
            is_configured: AsyncState::new(),
            is_running: AsyncState::new(),
            node_completion: AsyncState::new(),
            node_suspended: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Return the name of this FBPNodeContext
    ///
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Return the UUID associated with the FBPNodeContext
    ///
    /// All FBPNodeContexts and by extension all FBP Nodes have a unique identifier.  This is used
    /// to add and remove specific instances of an FBP node from various groups.
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Return the Receiver<IIDMessage> for this FBPNodeContext
    ///
    /// This is the input queue for an FBP Node
    pub fn rx(&self) -> Box<FBPNodeReceiver> {
        self.rx.clone()
    }

    /// Return the Sender<IIDMessage> for this FBPNodeContext
    ///
    /// This is the output sender for an FBP Node
    pub fn tx(&self) -> Box<FBPNodeSender> {
        self.tx.clone()
    }
    /// Returns true if the node's thread is running and processing messages
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// // NOTE:  An FBPNodeContext is usally NOT created as a standalone struct.  It is used as
    /// // part of an FBP node as is outlined in the example in the NodeNetworkItem documentation.  
    /// // The following example is just to show how the FBPNodeContext struct works.
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    /// if my_node_context.node_is_running() {
    ///     println!("The node is running");
    /// } else {
    ///     println!("The node is NOT running");
    /// }
    /// ```
    pub fn node_is_running(&self) -> bool {
        self.is_running.is_ready()
    }

    /// Set if an NodeContext is running or not
    ///
    /// This method should only be called by the FBP system and not called directly
    pub fn set_node_is_running(&self, flag: bool) {
        self.is_running.set_is_ready(flag);
    }

    /// Wait for a node to be running
    ///
    /// This will block the caller until the node is running
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    ///
    /// async fn test_wait(a_context: &FBPNodeContext) {
    ///     a_context.wait_for_node_to_be_running().await;
    /// }
    ///
    /// ```
    pub async fn wait_for_node_to_be_running(&self) {
        self.is_running.clone().await;
    }

    /// Check to see if a node has stopped processing
    ///
    /// This will return true if the node has stopped processing
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    /// if my_node_context.node_has_completed() {
    ///     println!("The node has stopped running");
    /// } else {
    ///     println!("The node is still running");
    /// }
    ///
    /// ```
    pub fn node_has_completed(&self) -> bool {
        self.node_completion.is_ready()
    }

    /// Set if the FBPNodeContext has stopped running its thread.
    ///
    /// This method should not be called outside of the FBP control software
    pub fn set_node_has_completed(&self, flag: bool) {
        self.node_completion.set_is_ready(flag);
    }

    /// Wait for a node to stop running
    ///
    /// This will block the caller until the node has stopped running
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    ///
    /// async fn test_wait(a_context: &FBPNodeContext) {
    ///     a_context.wait_for_node_to_complete().await;
    /// }
    /// ```
    pub async fn wait_for_node_to_complete(&self) {
        self.node_completion.clone().await;
    }

    /// Returns whether a node is fully configured
    ///
    /// This will return true if the node is fully configured
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    /// if my_node_context.node_is_configured() {
    ///     println!("The node is fully configured");
    /// } else {
    ///     println!("The node needs to be configured");
    /// }
    ///
    /// ```
    pub fn node_is_configured(&self) -> bool {
        self.is_configured.is_ready()
    }

    /// Set if a node has all of its configuration data in place.
    ///
    /// This should be implemented by an FBP node when it requires configuration.  The easiest way
    /// to do this is to add an accessor for the required field and when it is set, call this
    /// method to signal that the node has all of its configurations in place and can start running
    pub fn set_node_is_configured(&self, flag: bool) {
        self.is_configured.set_is_ready(flag);
    }

    /// Wait for a node to be configured
    ///
    /// This will block the caller until the node is fully configured
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    ///
    /// async fn test_wait(a_context: &FBPNodeContext) {
    ///     a_context.wait_for_node_to_be_configured().await;
    /// }
    ///
    /// ```
    pub async fn wait_for_node_to_be_configured(&self) {
        self.is_configured.clone().await;
    }

    /// Returns whether the node has suspended processing
    ///
    /// This will return true if the node has been suspended
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    /// if my_node_context.node_is_suspended() {
    ///     println!("The node has been suspended");
    /// } else {
    ///     println!("The node is NOT suspended");
    /// }
    ///
    /// ```
    pub fn node_is_suspended(&self) -> bool {
        self.node_suspended.deref().load(Ordering::Relaxed)
    }

    /// Set if this FBPCondeContext is suspended
    ///
    /// This method should only be called by the FBP system and not called directly
    pub fn set_is_suspended(&self, flag: bool) {
        self.node_suspended.store(flag, Ordering::Relaxed)
    }

    /// Add an FBPContext to receive the output of this node
    ///
    /// This will create a Receiver Context and then add that context to the nodes output_vec field
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let mut my_node_context = FBPNodeContext::new("ExampleContext");
    /// let mut my_downstream_context = FBPNodeContext::new("DownStreamContext");
    ///
    /// // The Key parameter allows for grouping receiving nodes into groups.  This could be used
    /// // to send only certain types of output to certain groups.  Typically None is passed which
    /// // specifies that the node should receive ALL output from a node.
    /// my_node_context.add_receiver(&mut my_downstream_context, None);
    ///
    /// ```
    pub fn add_receiver(&mut self, receiver: &mut FBPNodeContext, key: Option<String>) {
        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.clone().unwrap();
        }

        if self.output_vec.get_type().is_empty() {
            let mut vec_for_key: Vec<Box<FBPNodeContext>> = Vec::new();
            vec_for_key.push(Box::new(receiver.clone()));
            self.output_vec
                .get_type()
                .insert(hash_key.clone(), vec_for_key);
        } else {
            if self.output_vec.get_type().get_mut(&hash_key).is_some() {
                self.output_vec
                    .get_type()
                    .get_mut(&hash_key)
                    .unwrap()
                    .push(Box::new(receiver.clone()));
            } else {
                let mut vec_for_key: Vec<Box<FBPNodeContext>> = Vec::new();
                vec_for_key.push(Box::new(receiver.clone()));
                self.output_vec
                    .get_type()
                    .insert(hash_key.clone(), vec_for_key);
            }
        }
    }

    /// Remove an FBPContext from the list of nodes to receive the output from
    ///
    /// This will find the receiver FBPNodeContext in the output_vec field of the node and
    /// will remove it from receiving the output of this node.
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let mut my_node_context = FBPNodeContext::new("ExampleContext");
    /// let mut my_downstream_context = FBPNodeContext::new("DownStreamContext");
    /// // The Key parameter allows for removing a receiving context from the output_vec that was
    /// // previously placed into an output group.  Typically this is set to None which just removes
    /// // the context entirely
    ///  my_node_context.remove_receiver(&mut my_downstream_context, None);
    ///
    /// ```
    pub fn remove_receiver(&mut self, receiver: &mut FBPNodeContext, key: Option<String>) {
        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.clone().unwrap();
        }

        if self.output_vec.get_type().get_mut(&hash_key).is_some() {
            let index = self
                .output_vec
                .get_type()
                .get_mut(&hash_key)
                .unwrap()
                .iter()
                .position(|r| r.deref() == receiver)
                .unwrap();

            self.output_vec
                .get_type()
                .get_mut(&hash_key)
                .unwrap()
                .remove(index);
        }
    }

    /// Returns the number of nodes that have registered to receive the output of this node.
    ///
    /// This will return the number of nodes that have asked to receive the output from this context.
    /// If the key option is set, then it will only count those contexts that have registered with
    /// the group.  If the key option is set to None, then all receivers will be counted
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    ///
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    /// let num_receivers = my_node_context.get_num_items_for_receiver_vec(None);
    ///
    /// ```
    pub fn get_num_items_for_receiver_vec(&self, key: Option<String>) -> usize {
        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.clone().unwrap();
        }

        let mut result: usize = 0;

        if self.output_vec.get_type().get(&hash_key).is_some() {
            result = self.output_vec.get_type().get(&hash_key).unwrap().len();
        }

        result
    }

    /// Post an IIDMessage to the input queue of a context.
    ///
    /// This will post a message to the input queue of this context.  Messages
    /// posted to the input queue are dealt with in a First In, First Out (FIFO)
    /// manner
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_node_context::*;
    /// use fbp::fbp_iidmessage::*;
    ///
    /// let a_msg = IIDMessage::new(MessageType::Data, Some("This is a payload".to_string()));
    /// let my_node_context = FBPNodeContext::new("ExampleContext");
    /// my_node_context.post_msg(a_msg);
    ///
    /// ```
    ///
    pub fn post_msg(&self, msg: IIDMessage) {
        if self.node_is_running() {
            self.tx.send(msg.clone());
        }
    }

    /// Post an IIDMessage to a specific set of receiver nodes
    ///
    /// This will post a message to the group of receivers that were added with a specific key
    /// when calling add_receiver.  The message will only be sent to those receivers that were
    /// added with the key
    ///
    /// /// Basic usage:
    /// ```
    ///
    /// use fbp::fbp_node_context::*;
    /// use fbp::fbp_iidmessage::*;
    ///
    /// let mut my_node_context = FBPNodeContext::new("ExampleContext");
    /// let mut group_a =  FBPNodeContext::new("GroupA");
    /// let mut group_b =  FBPNodeContext::new("GroupB");
    /// my_node_context.add_receiver(&mut group_a, Some("GroupA".to_string()));
    /// my_node_context.add_receiver(&mut group_b, Some("GroupB".to_string()));
    ///
    /// let group_a_msg = IIDMessage::new(MessageType::Data, Some("A GroupA msg".to_string()));
    /// let group_b_msg = IIDMessage::new(MessageType::Data, Some("A GroupB msg".to_string()));
    ///
    /// // This code would most likely be in the trait implementation  of
    /// // process_message(self: &mut Self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError>;
    /// // method.  This assumes that a node will create two different types of IIDMessages.  One
    /// // for GroupA and one for GroupB.
    ///
    /// my_node_context.post_msg_to_group(group_a_msg, Some("GroupA".to_string()));
    /// my_node_context.post_msg_to_group(group_b_msg, Some("GroupB".to_string()));
    /// ```
    ///
    pub fn post_msg_to_group(&self, msg: IIDMessage, key: Option<String>) {
        if key.is_none() {
            return;
        }

        let hash_key = key.unwrap();

        if self.output_vec.get_type().get(&hash_key).is_none() {
            return;
        }

        for ctx in self.output_vec.get_type().get(&hash_key).unwrap().iter() {
            ctx.post_msg(msg.clone());
        }
    }
}

impl PartialEq for FBPNodeContext {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

/* --------------------------------------------------------------------------
 Unit Tests
------------------------------------------------------------------------- */

mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use serde_json::value::Value;
    use std::fs::{File, OpenOptions};
    use std::io::{Error, ErrorKind, Read, Write};
    use std::ops::Deref;
    use std::path::Path;

    #[allow(unused_imports)]
    use std::{thread, time};

    use crate::fbp_node_error::*;
    use crate::fbp_node_trait::*;

    /* --------------------------------------------------------------------------
     Define some FBP nodes that can be used for testing.
    -------------------------------------------------------------------------- */

    const LOGGER_GROUP: &str = "Logger_Group";

    // The LoggerNode will write out the payload of all messages to a file.
    #[derive(Clone, Serialize, Deserialize)]
    pub struct LoggerNode {
        data: Box<FBPNodeContext>,

        #[serde(skip)]
        log_file_path: ThreadSafeOptionType<String>,
    }

    impl LoggerNode {
        #[allow(dead_code)]
        pub fn new() -> Self {
            let result = LoggerNode {
                data: Box::new(FBPNodeContext::new("LoggerNode")),
                log_file_path: ThreadSafeOptionType::new(None),
            };

            result.clone().start();
            result
        }

        pub fn set_log_file_path(&mut self, log_file_path: String) {
            self.log_file_path.set_option(Some(log_file_path));

            // Ensure the File
            let string_ref = self.log_file_path.get_option().as_ref().unwrap().clone();
            let file_path = Path::new(string_ref.as_str());
            let _file = File::create(file_path).expect("Unable to create file");
            // drop(file);

            self.data.set_node_is_configured(true);
        }

        #[allow(dead_code)]
        pub fn get_log_string(&self) -> Result<String, Error> {
            if self.log_file_path.is_none() {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Cannot get log string until the node is setup",
                ));
            }

            let mut contents = String::new();
            let string_ref = self.log_file_path.get_option().as_ref().unwrap().clone();

            let file_path = Path::new(string_ref.as_str());
            let mut file = OpenOptions::new()
                .read(true)
                .open(file_path)
                .expect("Failed to open file {} for reading");

            file.read_to_string(&mut contents)
                .expect("Failed to write contents to string");

            Ok(contents)
        }

        pub fn log_string_to_file(&self, data: &String) -> Result<(), Error> {
            if self.log_file_path.is_none() {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Cannot get log to file until the node is setup",
                ));
            }

            let string_ref = self.log_file_path.get_option().as_ref().unwrap().clone();
            let file_path = Path::new(string_ref.as_str());

            let mut file = OpenOptions::new()
                .append(true)
                .open(file_path)
                .expect("Failed to open file for append");

            let string_to_write = data.clone();
            let string_to_write = string_to_write.replace("\0", "");

            let _write_result = file.write(string_to_write.as_bytes());
            Ok(())
        }
    }

    #[async_trait]
    impl FBPNodeTrait for LoggerNode {
        fn node_data_clone(&self) -> FBPNodeContext {
            self.data.deref().clone()
        }

        fn node_data(&self) -> &FBPNodeContext {
            &self.data
        }

        fn node_data_mut(&mut self) -> &mut FBPNodeContext {
            &mut self.data
        }

        // Implement the process_config to se the log file path
        fn process_config(
            &mut self,
            msg: IIDMessage,
        ) -> std::result::Result<IIDMessage, NodeError> {
            if msg.msg_type() == MessageType::Config {
                if msg.payload().is_some() {
                    let payload = msg.payload().as_ref().unwrap();
                    let config_message: ConfigMessage = serde_json::from_str(&payload)
                        .expect("Failed to deserialize the config message");

                    match config_message.msg_type() {
                        ConfigMessageType::Field => {
                            if config_message.data().as_ref().is_some() {
                                let config_str = json!(config_message.data().as_ref().unwrap());
                                let key_str = "log_file_path";
                                if config_str.to_string().contains(key_str) {
                                    let json_str = config_str.as_str().unwrap();
                                    let convert_result = serde_json::from_str(json_str);
                                    if convert_result.is_ok() {
                                        let json_value: Value = convert_result.unwrap();
                                        let the_value = &json_value[key_str];
                                        let log_file_path =
                                            String::from(the_value.as_str().unwrap());
                                        self.set_log_file_path(log_file_path);
                                    }
                                }
                            }
                        }
                        ConfigMessageType::Connect => {
                            // Deal with a Connect
                            // This is not implemented for this example
                        }
                        ConfigMessageType::Disconnect => {
                            // Deal with a Disconnect
                            // This is not implemented for this example
                        }
                    };
                } //  if msg.payload.is_some()
            } // if msg.msg_type == MessageType::Config

            Ok(IIDMessage::new(MessageType::Invalid, None))
        }

        // Implement the process_message to do the work of this node by writing the log to a file
        fn process_message(&mut self, msg: IIDMessage) -> Result<IIDMessage, NodeError> {
            if msg.payload().is_some() {
                let log_string = msg.clone().payload().as_ref().clone().unwrap().clone();
                if self.log_string_to_file(&log_string).is_err() {
                    return Err(NodeError::new("Failed to write message to log file"));
                }
            }

            Ok(msg.clone())
        }
    }

    // The PassthroughNode just passes messages from its input to all of its
    // receivers.
    #[derive(Clone, Serialize, Deserialize)]
    pub struct PassthroughNode {
        data: Box<FBPNodeContext>,
    }

    impl PassthroughNode {
        #[allow(dead_code)]
        pub fn new() -> Self {
            let result = PassthroughNode {
                data: Box::new(FBPNodeContext::new("PassthroughNode")),
            };

            result.node_data().set_node_is_configured(true);
            result.clone().start();
            result
        }
    }

    #[async_trait]
    impl FBPNodeTrait for PassthroughNode {
        fn node_data_clone(&self) -> FBPNodeContext {
            self.data.deref().clone()
        }

        fn node_data(&self) -> &FBPNodeContext {
            &self.data
        }

        fn node_data_mut(&mut self) -> &mut FBPNodeContext {
            &mut self.data
        }

        fn process_message(
            &mut self,
            msg: IIDMessage,
        ) -> std::result::Result<IIDMessage, NodeError> {
            // Check to see if there is a LOGGER_GROUP.  If there is create and send a log message
            if self
                .node_data()
                .get_num_items_for_receiver_vec(Some(LOGGER_GROUP.to_string()))
                > 0
            {
                // Create a Log message
                if msg.payload().is_some() {
                    let orig_payload = msg.payload().as_ref().unwrap().clone();
                    let mut new_payload =
                        "The PassthroughNode received a data message with this payload: "
                            .to_string();
                    new_payload.push_str(orig_payload.as_str());
                    let logger_msg = IIDMessage::new(MessageType::Data, Some(new_payload.clone()));

                    self.node_data()
                        .post_msg_to_group(logger_msg, Some(LOGGER_GROUP.to_string()));
                }
            }
            Ok(msg.clone())
        }
    }

    // This test the ability of having multiple output groups on a node.  A PassthroughNode will
    // have two LoggerNodes as receivers.  One will be in the LOGGER_GROUP while the other will
    // be in the 'Normal (Any)' group.  When the Passthrough node gets a IIDMessage, it will
    // see if it has any nodes in the LOGGER_GROUP.  If it does then it will create a Log IIDMessage
    // and send it ONLY to those nodes in the LOGGER_GROUP.  IIDMessages that are returned from
    // the PassthroughNode are propagated tro any nodes that are in the 'Normal (Any)' group.
    // This is just one way to use the grouping of outputs.
    #[test]
    fn multiple_outputs() {
        // Create a Logger Node to Log messages sent to the PassthroughNode
        let mut lg_node = LoggerNode::new();
        lg_node.set_log_file_path("PassthroughNode_Log.txt".to_string());

        // Create the PassthroughNode and add the Logger node the the LOGGER_GROUP.
        let mut pt_node = PassthroughNode::new();
        pt_node
            .node_data_mut()
            .add_receiver(lg_node.node_data_mut(), Some(LOGGER_GROUP.to_string()));

        // Now add another instance of the Logger Node but add it to the Any group (No group)

        let mut lg_normal_node = LoggerNode::new();
        lg_normal_node.set_log_file_path("Normal_Log.txt".to_string());
        pt_node
            .node_data_mut()
            .add_receiver(lg_normal_node.node_data_mut(), None);

        let msg_str = "It was the best of times, it was the worst of times".to_string();

        let a_msg = IIDMessage::new(MessageType::Data, Some(msg_str.clone()));
        pt_node.node_data().post_msg(a_msg);

        thread::sleep(time::Duration::from_secs(2));

        let log_str_result = lg_node.get_log_string();
        assert!(log_str_result.is_ok());

        let log_string = log_str_result.unwrap();
        let good_log_string = "The PassthroughNode received a data message with this payload: It was the best of times, it was the worst of times".to_string();

        assert_eq!(log_string, good_log_string);

        let normal_log_str_result = lg_normal_node.get_log_string();
        assert!(normal_log_str_result.is_ok());

        let normal_log_string = normal_log_str_result.unwrap();
        let good_normal_log_string =
            "It was the best of times, it was the worst of times".to_string();
        assert_eq!(normal_log_string, good_normal_log_string);
    }
}
