/* =================================================================================
 File:           fbp_node_network.rs

 Description:    This file defines types and macros needed to be able to use and
                 create JSON structs to describe a network of FBP Nodes.


 History:        RustDev 04/26/2021  Initial code
================================================================================== */
#![allow(unused_imports)]

//! # Node Network
//!
//! A Flow Based Programming system typically involves both the creation of FBP nodes
//! and using those nodes to form a network.  The network is a set of nodes that link
//! to other nodes by sending the output of one node to the input of another node.
//! This relationship forms a directed acyclic graph of nodes.
//!
//! This network should be represented as a first class object so that the entire network
//! can be manipulated as a single unit.  That is part of the job of the types in this
//! file, to gather all of the FBP nodes running for an application into a structure
//! that can be used to treat the network as a single unit.
//!
//! There is also a need to define a representation of this network so that a set of nodes
//! may be created from this representation to form a network of nodes that is able
//! to perform the necessary processing.  While using this representation will simplify
//! using a network of nodes, it should be noted that it is **not** required to use the
//! Flow Based Programming system in this crate.  Thus, the usage of the representation
//! (JSON) to create a network of nodes is controlled by a feature (fbp_network) for which the
//! default is that the feature is **not** on.  To use the representation feature one would
//! add the dependency  for this crate using a features parameter:
//!
//! fbp = { version = "0.1.0", features = ["fbp_network"] }
//!
//! This fbp_network feature system uses a JSON representation which is
//! basically a string. This requires that there be a way to create an FBP node from the
//! string name of the node.  That is what the make_nodes macro does.  It creates an enum
//! NodesEnum that will contain all of the all of the node struct names.  The macro also creates an
//! implementation of the NodeCreator trait for the NodesEnum that matches the enum
//! which in turn will create a new node and then returns a clone of the FBPNodeContext
//! struct associated with the Node.  Given that all nodes **MUST** have a
//! Box<FBPNodeContext> as one of their fields, the clone will still point to the
//! same underlying struct in memory.
//!
//! This ability will form the basis of how a network of nodes may be created from a
//! JSON representation.

#[cfg(feature = "fbp_network")]
use crate::fbp_node_network::tests::NodesEnum;
use async_trait::async_trait;
use futures::future::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;
use std::{thread, time};

#[cfg(feature = "fbp_network")]
use strum::IntoEnumIterator;

#[cfg(feature = "fbp_network")]
use strum_macros::EnumIter;

#[cfg(feature = "fbp_network")]
use crate::fbp_node_error::*;

#[cfg(feature = "fbp_network")]
use std::str::FromStr;

#[cfg(feature = "fbp_network")]
use strum_macros::EnumString;

use crate::fbp_asyncstate::*;
use crate::fbp_iidmessage::*;
use crate::fbp_node_context::*;

/// # NodeCreator Trait
///
/// The NodeCreator trait is usen by the make_nodes macro to provide the ability to create
/// an FBP Node from the string name of the struct.
#[cfg(feature = "fbp_network")]
pub trait NodeCreator {
    fn make_node(&self) -> Option<FBPNodeContext>;

    fn find_node(node_name: &str) -> Option<Self>
    where
        Self: Sized;
}
static mut HAS_NODESENUM: AtomicBool = AtomicBool::new(false);

/// set_has_nodesenum
///
/// This is an internal function that is used to signal that the
/// fbp_network feature is in use
#[allow(dead_code)]
fn set_has_nodesenum(flag: bool) {
    unsafe {
        *HAS_NODESENUM.get_mut() = flag;
    }
}

/// get_has_nodesenum
///
/// This function will return true if the fbp_network feature is on.
#[allow(dead_code)]
fn get_has_nodesenum() -> bool {
    unsafe { return HAS_NODESENUM.load(Ordering::SeqCst) }
}

/// The make_nodes macro, is the key to being able to create a network
/// of nodes from a JSON string.  its usage requires that the fbp_network
/// feature be 'ON'.
///
/// The macro creates an Enum (NodesEnum) with all of the FBP nodes that
/// are listed in the macro
///
/// make_nodes!(NodeA, NodeB, NodeC);
///
/// This needs to be done is a file that uses all of the node files so
/// that the macro can 'see' all of the nodes.  
///
/// The macro also creates an implementation of the NodeCreator trait for
/// the NodesEnum.  This allows for getting an enum from that str for the
/// node i.e. let enum_op: Option<NodesEnum> =
///      NodesEnum::find_node(node_struct_name_as_a_str);
///
/// If the NodesEnum has that enum, then the the Some value will be the enum.
/// The enum can then be used to create an instance of that node
///
///  // let ctx: Option<FBPNodeContext> = enum_op.unwrap().make_node();
///
/// It is this ability to create an instance of an FBPNode from a str that allows
/// for using a JSON string to describe a comnplete FBP network of nodes and
/// subsequently create those nodes.
#[cfg(feature = "fbp_network")]
#[macro_export]
macro_rules! make_nodes {

    ($($x:ident),*) => {
        #[derive(EnumString, EnumIter, strum_macros::ToString)]
        pub enum NodesEnum {
            $(
                $x,
            )*
        }

        impl NodeCreator for NodesEnum {
            fn make_node(&self) -> Option<FBPNodeContext> {
                let result = match self {
                    $(
                        NodesEnum::$x =>  {
                            set_has_nodesenum(true);
                            let mut a_node = $x::new();
                            Some(a_node.node_data_mut().clone())
                        }
                    )*
                };
                result
            }

            fn find_node(node_name:&str) -> Option<Self> {
                set_has_nodesenum(true);
                let enum_result = NodesEnum::from_str(node_name);
                if enum_result.is_err() {
                    return None
                }
                let an_enum = enum_result.unwrap();
                Some(an_enum)
            }

        }
    };
}

/// # NodeNetworkItem
///
/// A NodeNetworkItem define the string representation of an FBP node.
///
/// A JSON string can be created from this class to be used to create
/// a node.
///
/// # Example
///
/// ```
/// // Assuming FBP Nodes NodeA and NodeB and neither of these nodes
/// // requires configurations.
///
/// use serde::{Deserialize, Serialize};
/// use std::collections::HashMap;
/// use uuid::Uuid;
/// use async_trait::async_trait;
/// use std::sync::atomic::{Ordering, AtomicBool};
/// use futures::future::*;
/// use std::ops::{Deref};
///
/// use fbp::fbp_node_network::*;
/// use fbp::fbp_node_context::*;
/// use fbp::fbp_node_trait::*;
/// use fbp::fbp_iidmessage::*;
/// use fbp::fbp_node_error::*;
///
///
///   
/// #[derive(Clone, Serialize, Deserialize)]
/// pub struct NodeA {
///     data: Box<FBPNodeContext>,
/// }
///
/// impl NodeA {
///     #[allow(dead_code)]
///     pub fn new() -> Self {
///         let result = NodeA {
///             data: Box::new(FBPNodeContext::new("NodeA")),
///         };
///
///         result.node_data().set_node_is_configured(true);
///         result.clone().start();
///         result
///     }
/// }
///
/// #[async_trait]
/// impl FBPNodeTrait for NodeA {
///     fn node_data_clone(&self) -> FBPNodeContext {
///         self.data.deref().clone()
///     }
///
///     fn node_data(&self) -> &FBPNodeContext { &self.data }
///
///     fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
///
///     fn process_message(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
///         Ok(msg.clone())
///     }
/// }
///
///   
/// #[derive(Clone, Serialize, Deserialize)]
/// pub struct NodeB {
///     data: Box<FBPNodeContext>,
/// }
///
/// impl NodeB {
///     #[allow(dead_code)]
///     pub fn new() -> Self {
///         let result = NodeB {
///             data: Box::new(FBPNodeContext::new("NodeB")),
///         };
///
///         result.node_data().set_node_is_configured(true);
///         result.clone().start();
///         result
///     }
/// }
///
/// #[async_trait]
/// impl FBPNodeTrait for NodeB {
///     fn node_data_clone(&self) -> FBPNodeContext {
///         self.data.deref().clone()
///     }
///
///     fn node_data(&self) -> &FBPNodeContext { &self.data }
///
///     fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
///
///     fn process_message(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
///         Ok(msg.clone())
///     }
/// }
///
/// let mut nni = NodeNetworkItem::new("NodeA".to_string());
/// nni.add_connection(&"NodeB".to_string(), None);
///
/// let json_string = serde_json::to_string(&nni).unwrap();
///
/// // Subsequently, the NodeNetworkItem can be 'reconstructed' as follows:
///
/// let a_nni: NodeNetworkItem = serde_json::from_str(json_string.as_str()).unwrap();
///
/// // With a NodeNetworkItem, the node can be created as follows:
///
/// let net_config_op: Option<NetworkConfiguration> = a_nni.create_node();
///
/// // A NetworkConfiguration is a struct which contains all of the ndoes that
/// // were constructed.  Please the see the documentation for the NetworkConfiguration
/// // struct
/// ```
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeNetworkItem {
    node_name: String,

    #[serde(skip)]
    uuid: Uuid,

    configurations: Vec<String>,
    pub connections: NodeNetwork,
}

impl NodeNetworkItem {
    pub fn new(node_name: String) -> Self {
        NodeNetworkItem {
            node_name: node_name.clone(),
            uuid: Uuid::new_v4(),
            configurations: Vec::new(),
            connections: NodeNetwork::new(),
        }
    }

    pub fn name(&self) -> String {
        self.node_name.clone()
    }

    pub fn add_configuration(&mut self, config_str: String) {
        let configs = &mut self.configurations;
        configs.push(config_str);
    }

    pub fn add_connection(&mut self, node_name: &String, key: Option<String>) {
        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.unwrap();
        }

        self.connections.add_node(node_name.clone(), Some(hash_key));
    }

    pub fn add_node_connection(&mut self, node: NodeNetworkItem, key: Option<String>) {
        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.unwrap();
        }

        self.connections.add_exiting_node(node, Some(hash_key));
    }

    fn base_create_node(&self) -> Option<FBPNodeContext> {
        if get_has_nodesenum() {
            #[cfg(feature = "fbp_network")]
            {
                let an_enum_op = NodesEnum::find_node(self.name().as_str());
                if an_enum_op.is_none() {
                    return None;
                }
                let an_enum = an_enum_op.unwrap();

                let ctx_op = an_enum.make_node();

                if ctx_op.is_none() {
                    println!("Failed to crate a {} node", self.name());
                    return None;
                }

                if self.configurations.len() > 0 {
                    let ctx = ctx_op.clone().unwrap().clone();
                    for a_config_str in &self.configurations {
                        let my_config_message = ConfigMessage::new(
                            ConfigMessageType::Field,
                            Some(a_config_str.to_string()),
                        );
                        let msg = my_config_message.make_message(MessageType::Config);
                        ctx.post_msg(msg);
                    }
                }
                return ctx_op;
            }
        }

        None
    }

    fn make_connection(
        &self,
        key: Option<String>,
        parent_node_op: Option<&mut FBPNodeContext>,
        net_config: &mut NetworkConfiguration,
    ) {
        if get_has_nodesenum() {
            let ctx_op = self.base_create_node();
            if ctx_op.is_none() {
                return;
            }

            let ctx = ctx_op.unwrap().clone();

            if parent_node_op.is_some() {
                let parent_node = parent_node_op.unwrap();
                parent_node.add_receiver(&mut ctx.clone(), key);
            }
            net_config.add_node(ctx.clone());

            if self.connections.node_network.len() > 0 {
                for (key, value) in &self.connections.node_network {
                    for a_nni in value {
                        a_nni.make_connection(
                            Some(key.clone()),
                            Some(&mut ctx.clone()),
                            net_config,
                        );
                    }
                }
            }
        }
    }

    pub fn create_node(&self) -> Option<NetworkConfiguration> {
        if get_has_nodesenum() {
            let ctx_op = self.base_create_node();
            if ctx_op.is_none() {
                return None;
            }

            let mut ctx = ctx_op.unwrap();

            let mut net_config = NetworkConfiguration::new();
            net_config.add_node(ctx.clone());

            if self.connections.node_network.len() > 0 {
                for (key, value) in &self.connections.node_network {
                    for a_nni in value {
                        a_nni.make_connection(Some(key.clone()), Some(&mut ctx), &mut net_config);
                    }
                }
            }

            return Some(net_config);
        }
        None
    }
}

impl PartialEq for NodeNetworkItem {
    fn eq(&self, other: &Self) -> bool {
        self.node_name == other.node_name && self.uuid == other.uuid
    }
}

impl Eq for NodeNetworkItem {}

/// # NodeNetwork
///
/// The NodeNetwork type represents the network of nodes.  It is just a HashMap that is keyed
/// my a string that represents a grouping of nodes.  This grouping is not required by will
/// allow for having various groups of downstream nodes receive different messages from another
/// group.  While this grouping is optional, it does provide for increased flexibility.
///
/// The way that this would work is that a node's implementation of the FBPNodeTrait::process_message
/// would need to determine if there were a separate grouping of nodes for a specific need and only
/// send specific messages to that group.  A made up example might be that a separate logging
/// group exists and the node would performs its 'normal' processing and then send that result on to
/// the 'normal' outputs, while it could also send logging information to a logging group.
///
/// The value of the HashMap is a Vector of NodeNetworkItems that represent an FBP Node that will be
/// constructed and placed into the same position as the NodeNetworkItem in the NodeNetwork.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NodeNetwork {
    pub node_network: HashMap<String, Vec<NodeNetworkItem>>,
}

impl NodeNetwork {
    pub fn new() -> Self {
        NodeNetwork {
            node_network: HashMap::new(),
        }
    }

    pub fn node_network(&self) -> &HashMap<String, Vec<NodeNetworkItem>> {
        &self.node_network
    }

    fn insert_node(&mut self, a_node: NodeNetworkItem, hash_key: String) {
        if self.node_network.get_mut(&hash_key).is_some() {
            let a_node_vec = self.node_network.get_mut(&hash_key).unwrap();
            a_node_vec.push(a_node);
        } else {
            let mut network: Vec<NodeNetworkItem> = Vec::new();
            network.push(a_node.clone());
            self.node_network.insert(hash_key.clone(), network);
        }
    }

    pub fn find_node_by_name(
        &mut self,
        node_name: String,
        key: Option<String>,
    ) -> Option<&mut NodeNetworkItem> {
        let mut result: Option<&mut NodeNetworkItem> = None;

        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.clone().unwrap();
        }

        let op_my_vec = self.node_network.get_mut(&hash_key);
        if op_my_vec.is_some() {
            let my_vec = op_my_vec.unwrap();
            for nni in my_vec {
                if nni.name() == node_name {
                    result = Some(nni);
                    break;
                }
            }
        }

        result
    }

    pub fn add_node(&mut self, node_name: String, key: Option<String>)
    /*> Option<&mut NodeNetworkItem> */
    {
        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.clone().unwrap();
        }

        self.insert_node(NodeNetworkItem::new(node_name).clone(), hash_key.clone());
        // self.get_node(new_item.clone(), hash_key.clone())
    }

    pub fn add_exiting_node(&mut self, a_node: NodeNetworkItem, key: Option<String>) {
        let mut hash_key = "Any".to_string();

        if key.is_some() {
            hash_key = key.clone().unwrap();
        }

        self.insert_node(a_node.clone(), hash_key.clone());
    }
}

/// # NetworkConfiguration
///
/// The NetworkConfiguration struct is a grouping of all of the nodes in a
/// network of nodes.  It does **not** describe the actual network but does
/// allow for treating the network as a unit instead of a set of disparate
/// nodes.
///
/// This struct can be used in two ways.  The first is to create a network
/// from a JSON representation of the network.  This is done by using the
/// NodeNetworkItem to create all of the nodes in the network.  The call to
/// NodeNetworkItem::create_node returns an Option<NetworkConfiguration>.  This
/// can be used to work with the network as a whole.
///
/// The second way to use the NetworkConfiguration struct is to create an
/// empty NetworkConfiguration struct and calling add_node on the FBPNodeContext
/// of your network.  This will allow for doing the same thing as calling
/// NodeNetworkItem::create_node in that the network of nodes can now be addressed
/// as a unit.
pub struct NetworkConfiguration {
    created_nodes: Vec<FBPNodeContext>,
}

impl NetworkConfiguration {
    pub fn new() -> Self {
        NetworkConfiguration {
            created_nodes: Vec::new(),
        }
    }

    pub fn find_node_name(&self, node_name: String) -> Option<FBPNodeContext> {
        for a_node in &self.created_nodes {
            if a_node.name() == node_name {
                return Some(a_node.clone());
            }
        }
        None
    }

    pub fn add_node(&mut self, a_node: FBPNodeContext) {
        self.created_nodes.push(a_node.clone());
    }

    pub fn remove_node(&mut self, a_node: FBPNodeContext) {
        if let Some(pos) = self.created_nodes.iter().position(|c| *c == a_node) {
            self.created_nodes.remove(pos);
        }
    }

    pub fn get_number_of_nodes(&self) -> usize {
        self.created_nodes.len()
    }

    pub fn stop_all_created_nodes(&self) {
        let stopper =
            ProcessMessage::new(ProcessMessageType::Stop, false).make_message(MessageType::Process);

        for a_node in &self.created_nodes {
            a_node.post_msg(stopper.clone());
        }
    }

    pub async fn wait_for_nodes_to_be_configured(&mut self) {
        let contents: Vec<&mut AsyncState> = self
            .created_nodes
            .iter_mut()
            .map(|node| &mut node.is_configured)
            .collect();

        join_all(contents).await;
    }

    pub async fn wait_for_nodes_to_all_be_running(&mut self) {
        let contents: Vec<&mut AsyncState> = self
            .created_nodes
            .iter_mut()
            .map(|node| &mut node.is_running)
            .collect();

        join_all(contents).await;
    }

    pub async fn wait_for_nodes_to_all_complete(&mut self) {
        let contents: Vec<&mut AsyncState> = self
            .created_nodes
            .iter_mut()
            .map(|node| &mut node.node_completion)
            .collect();

        let mut loop_value = true;
        while loop_value {
            let mut completed = true;
            for x in &contents {
                if !x.is_ready() {
                    completed = false;
                }
            }

            if !completed {
                thread::sleep(time::Duration::from_millis(100));
            } else {
                loop_value = false;
            }
        } 
        // join_all(contents).await;
    }
}

/* --------------------------------------------------------------------------
 Unit Tests
------------------------------------------------------------------------- */

mod tests {
    use super::*;
    use serde_json::json;
    use serde_json::value::Value;
    use std::fs::{File, OpenOptions};
    use std::io::{Error, ErrorKind, Read, Write};
    use std::ops::Deref;
    use std::path::Path;
    use std::{thread, time};

    use std::convert::AsRef;
    use strum::IntoEnumIterator;
    use strum_macros::AsRefStr;
    use strum_macros::EnumIter;

    use crate::fbp_iidmessage::*;
    use crate::fbp_node_context::*;
    use crate::fbp_node_error::*;
    use crate::fbp_node_trait::*;
    use crate::fbp_threadsafe_wrapper::*;
    use crate::fbp_wait_for_payload::*;

    /* --------------------------------------------------------------------------
     Define some FBP nodes that can be used for testing.
    -------------------------------------------------------------------------- */

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

            Ok(msg.clone())
        }
    }

    // The AppendNode will append a preset value to the end of any message it receives and then
    // will forward that updated message on to all of its receivers.
    #[derive(Clone, Serialize, Deserialize)]
    pub struct AppendNode {
        data: Box<FBPNodeContext>,

        #[serde(skip)]
        append_data: ThreadSafeOptionType<String>,
    }

    impl AppendNode {
        #[allow(dead_code)]
        pub fn new() -> Self {
            let result = AppendNode {
                data: Box::new(FBPNodeContext::new("AppendNode")),
                append_data: ThreadSafeOptionType::new(None),
            };

            result.clone().start();
            result
        }

        pub fn set_append_data(&mut self, data: String) {
            self.append_data.set_option(Some(data));
            // This is the only outstanding field that needed to be configured
            // once set, the node is configured.
            self.data.set_node_is_configured(true);
        }
    }

    #[async_trait]
    impl FBPNodeTrait for AppendNode {
        fn node_data_clone(&self) -> FBPNodeContext {
            self.data.deref().clone()
        }

        fn node_data(&self) -> &FBPNodeContext {
            &self.data
        }

        fn node_data_mut(&mut self) -> &mut FBPNodeContext {
            &mut self.data
        }

        // Here is an example of a node needing additional data before it can start processing
        // incoming IIDMessages.  The AppendNode FBP Node needs to be configured with the
        // string that will be appended to incoming messages.  That is why the process_config
        // method is implemented.  It will parse the incoming Config message and will then call
        // the set_append_data method after the string has been extracted from the payload.
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
                                let key_str = "append_data";
                                if config_str.to_string().contains(key_str) {
                                    let json_str = config_str.as_str().unwrap();

                                    let convert_result = serde_json::from_str(json_str);
                                    if convert_result.is_ok() {
                                        let json_value: Value = convert_result.unwrap();
                                        let the_value = &json_value[key_str];
                                        let append_str = String::from(the_value.as_str().unwrap());

                                        self.set_append_data(append_str);
                                    }
                                }
                            }
                        }
                        ConfigMessageType::Connect => {
                            // Deal with a Connect
                            // This is not implemented for this example
                        }
                        ConfigMessageType::Disconnect => {
                            // Deai with a Disconnect
                            // This is not implemented for this example
                        }
                    };
                } //  if msg.payload.is_some()
            } // if msg.msg_type == MessageType::Config

            // Configuration messages should almost never be propagated as they relate to a specific
            // FBP node.  Sending an Invalid message will stop message propagation.
            Ok(IIDMessage::new(MessageType::Invalid, None))
        }

        // Given that the AppendNode does some work, it needs to implement the process_message
        // method to do that work
        fn process_message(&mut self, msg: IIDMessage) -> Result<IIDMessage, NodeError> {
            if self.append_data.is_none() {
                return Ok(msg.clone());
            }

            let the_string = self.append_data.get_option().as_ref().unwrap().clone();
            let mut payload = msg.payload().as_ref().unwrap().clone();
            payload.push_str(the_string.as_str());
            let new_msg = IIDMessage::new(MessageType::Data, Some(payload));
            Ok(new_msg)
        }
    }

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

        // Implement the process_config to use the log file path
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
                            // Deai with a Disconnect
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
                if self
                    .log_string_to_file(&msg.clone().payload().as_ref().unwrap())
                    .is_err()
                {
                    return Err(NodeError::new("Failed to write message to log file"));
                }
            }

            Ok(msg.clone())
        }
    }
    /* --------------------------------------------------------------------------
    End of Test FBP nodes
    -------------------------------------------------------------------------- */

    // Helper function to remove log file
    #[allow(dead_code)]
    pub fn remove_logger_file(path_str: &str) {
        if Path::new(path_str).exists() {
            let _ = std::fs::remove_file(path_str);
        }
    }

    // Helper function to get the contents of the log file
    #[allow(dead_code)]
    pub fn get_logger_file_string(path_str: &str) -> String {
        let file_path = Path::new(path_str);
        let mut file = OpenOptions::new()
            .read(true)
            .open(file_path)
            .expect("Failed to open file {} for reading");

        let mut result: String = "".to_string();
        file.read_to_string(&mut result)
            .expect("Failed to write contents to string");

        result
    }

    // Typically, an FBP application will define nodes in multiple files.
    // There will then need to be a single file where all of the Node files are included
    // (with a use statement).  Once that is done the the make_nodes! macro will be called with the
    // list of the nodes.  The placement of this call outside of any function mimics the 'normal'
    // way that the make_nodes! macro will be used.
    #[cfg(feature = "fbp_network")]
    make_nodes!(PassthroughNode, AppendNode, LoggerNode);

    
    #[test]
    #[cfg(feature = "fbp_network")]
    fn feature_test() {
        set_has_nodesenum(true);
        for node_enum in NodesEnum::iter() {
            let node_name = node_enum.to_string();
            let ctx_op = node_enum.make_node();
            assert!(ctx_op.is_some());
            assert_eq!(node_name, ctx_op.unwrap().name());
        }
    }

    #[test]
    fn node_creation_test() {
        let pt_node = PassthroughNode::new();
        let mut node_name = pt_node.node_data().name();
        assert_eq!("PassthroughNode".to_string(), node_name);

        let ap_node = AppendNode::new();
        node_name = ap_node.node_data().name();
        assert_eq!("AppendNode".to_string(), node_name);

        let lg_node = LoggerNode::new();
        node_name = lg_node.node_data().name();
        assert_eq!("LoggerNode".to_string(), node_name);
    }

    // These next two tests do the same thing.
    //
    // The network_serialization test, creates a text
    // representation of a three node network.  This is done by setting up the NodeNetworkItem
    // with all of the nodes, configurations, and connections needed for the FBP network. This
    // network is then serialized using the serde_json::to_string(&pt_node) method.  Once
    // serialized into a JSON string, that string can be used to create a 'real' FBP network of
    // nodes.

    #[actix_rt::test]
    async fn by_hand_node_network() {
        remove_logger_file("Log1_file.txt");

        let mut pt_node = PassthroughNode::new();
        let mut ap_node = AppendNode::new();
        let mut lg_node = LoggerNode::new();

        ap_node.set_append_data(" World".to_string());
        lg_node.set_log_file_path("Log1_file.txt".to_string());

        pt_node
            .node_data_mut()
            .add_receiver(ap_node.node_data_mut(), None);
        ap_node
            .node_data_mut()
            .add_receiver(lg_node.node_data_mut(), None);

        let data_msg = IIDMessage::new(MessageType::Data, Some("Hello".to_string()));
        pt_node.node_data().post_msg(data_msg);

        // Most FBP systems are ongoing systems.  In this case, a bit of time is allotted to allow
        // message passing to occur.
        thread::sleep(time::Duration::from_secs(2));

        let log_string_result = lg_node.get_log_string();
        assert!(log_string_result.is_ok());

        let log_string = log_string_result.unwrap();
        assert_eq!(log_string, "Hello World".to_string());
    }


    // Do the same thing as the by_hand_node_network test does except
    // wait for the payload instead of using a sleep.
    #[actix_rt::test]
    async fn by_hand_node_network_with_wait() {
        let mut pt_node = PassthroughNode::new();
        let mut ap_node = AppendNode::new();
        let mut wait_node = WaitForPayloadNode::new();

        ap_node.set_append_data(" World".to_string());

        pt_node
            .node_data_mut()
            .add_receiver(ap_node.node_data_mut(), None);

        ap_node
            .node_data_mut()
            .add_receiver(wait_node.node_data_mut(), None);

        let data_msg = IIDMessage::new(MessageType::Data, Some("Hello".to_string()));
        pt_node.node_data().post_msg(data_msg);


        let the_payload = wait_node.get_payload().await;

        assert_ne!(the_payload.is_empty(), true);
        assert_eq!(the_payload, "Hello World".to_string());
    }

    // The network_json test, takes a JSON string that could be part of a "setup" that had
    // preconfigured the necessary JSON string to construct the necessary FBP network needed for
    // a specific application.  As noted, this network is excactly the same as the previous test
    // it just uses the JSON string directly instead of creating the string using the NodeNetworkItem
    // and NodeNetwork structs.  This example shows how compact the code can be when using pre-created
    // JSON strings to create an FBP network.

    #[actix_rt::test]
    #[cfg(feature = "fbp_network")]
    async fn network_from_json() {
        remove_logger_file("Log_file.txt");

        // Before using the NodeNetwork classes, one MUST call make_nodes! with the list of nodes
        // that can be created.  After the make_nodes! call, there needs to be a call to
        // set_has_nodesenum(true) or the NodeNetwork code will not work.  Typically this would be
        // done in the lib.rs or main.rs files.  Given this is a test, the call is being made here
        set_has_nodesenum(true);

        // This very long string would in 'normal' cases, be read from a file.  This is here for
        // a quick test.  It has the same organization as is seen in the by_hand_node_network test.
        let config_string = "{\"node_name\":\"PassthroughNode\",\"configurations\":[],\"connections\":{\"node_network\":{\"Any\":[{\"node_name\":\"AppendNode\",\"configurations\":[\"{\\\"append_data\\\": \\\" World\\\"}\"],\"connections\":{\"node_network\":{\"Any\":[{\"node_name\":\"LoggerNode\",\"configurations\":[\"{\\\"log_file_path\\\":\\\"Log_file.txt\\\"}\"],\"connections\":{\"node_network\":{}}}]}}}]}}}".to_string();

        let nni_result: Result<NodeNetworkItem, serde_json::Error> =
            serde_json::from_str(config_string.as_str());
        assert!(nni_result.is_ok());
        let nni = nni_result.unwrap();

        let config_op = nni.create_node();
        assert!(config_op.is_some());

        let mut config = config_op.unwrap();
        assert_eq!(config.get_number_of_nodes(), 3);

        config.wait_for_nodes_to_all_be_running().await;
        config.wait_for_nodes_to_be_configured().await;

        let pt_ctx_op = config.find_node_name("PassthroughNode".to_string());
        assert!(pt_ctx_op.is_some());

        let pt_ctx = pt_ctx_op.unwrap();

        let data_msg = IIDMessage::new(MessageType::Data, Some("Hello".to_string()));
        pt_ctx.post_msg(data_msg);

        // Most FBP systems are ongoing systems.  In this case, a bit of time is allotted to allow
        // message passing to occur.
        thread::sleep(time::Duration::from_secs(2));

        config.stop_all_created_nodes();
        config.wait_for_nodes_to_all_complete().await;

        let log_string = get_logger_file_string("Log_file.txt");
        assert_eq!(log_string, "Hello World".to_string());
    }
}
