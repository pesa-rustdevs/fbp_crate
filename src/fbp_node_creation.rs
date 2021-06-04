/* =================================================================================
    File:           fbp_node_creation.rs

    Description:    This file struct defines a way to create FBP nodes from a 
                    JSON string.  This is how an FBP network and be both saved and
                    reconsituted.


    History:        RustDev 04/14/2021   Code ported from original rustfbp crate

    Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
   ================================================================================== */


use serde::{Deserialize, Serialize};
use serde_json::*;
use std::io::{Error, ErrorKind, Read, Write};
use std::fs::{File, OpenOptions};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::result::Result;
use async_trait::async_trait;
use std::any::Any;
use std::str::FromStr;
use strum_macros::EnumString;
use traitcast::{TraitcastFrom, Traitcast};

use crate::fbp_iidmessage::*;
use crate::fbp_node_context::*;
use crate::fbp_node_trait::*;
use crate::fbp_node_error::*;
use crate::fbp_threadsafe_wrapper::*;

/*
pub trait ConvertToNode {

    fn convert(ctx: ThreadSafeType<FBPNodeContext>) -> Option<ThreadSafeType<Self>>
        where Self: std::marker::Sized + Clone + 'static {

        let mut result: Option<ThreadSafeType<&Self>> = None;


        let dyn_obj_op =  ctx.get_type().deref().get_trait_object();
        if dyn_obj_op.is_some() {
            let dyn_obj = *dyn_obj_op.unwrap().deref();
            let fu = dyn_obj.get_as_any();
            if let Some(obj) = fu.downcast_ref::<Self>() {
                result = Some(ThreadSafeType::new(obj));
            }
        }

        result
    }

}
*/


#[derive(Clone, Serialize, Deserialize)]
pub struct PassthroughNode<'a> {
    data: ThreadSafeType<FBPNodeContext<'a>>,
}

// impl ConvertToNode for PassthroughNode<'_> {}

impl PassthroughNode<'_> {

    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut result: PassthroughNode<'_> = PassthroughNode {
            data: ThreadSafeType::new(FBPNodeContext::new("PassthroughNode")),
        };

        let yatz: &PassthroughNode = &result;
        let dyn_obj: &(dyn FBPNodeTrait + 'static) = yatz;
        let box_obj = Box::new(dyn_obj);

        result.data.get_type().deref_mut().set_trait_object(box_obj);

        // result.data.get_type().deref_mut().set_trait_object(box_obj);
        // result.node_data_mut().set_trait_object(box_obj);
        result.clone().start();
        result
    }
}

#[async_trait]
impl FBPNodeTrait for PassthroughNode<'static> {

    fn node_data_clone(self) -> FBPNodeContext<'static> {
        self.data.get_type().deref().clone()
    }

    fn node_data(&self) -> &FBPNodeContext<'static> {
        &self.data.get_type().deref()
    }

    fn node_data_mut(&mut self) -> &mut FBPNodeContext<'static> {
        self.data.get_type().deref();
    }

    fn process_message(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
        Ok(msg.clone())
    }
}

// This FBP Node will take an incoming IIDMessage and append data to the
// payload of the message and then send it on.
#[derive(Clone, Serialize, Deserialize)] // PartialEq)]
pub struct AppendNode<'a> {
    data: ThreadSafeType<FBPNodeContext<'a>>,

    #[serde(skip)]
    append_data: ThreadSafeOptionType<String>, // Arc<Mutex<Option<String>>>,
}

// impl ConvertToNode for AppendNode<'_> {}

impl<'a> AppendNode<'a> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut result = AppendNode {
            data: ThreadSafeType::new(FBPNodeContext::new("AppendNode")), // Box::new(FBPNodeContext::new("AppendNode")),
            append_data: ThreadSafeOptionType::new(None), //Arc::new(Mutex::new(None)),
        };

        result.clone().start();
        result
    }

    pub fn set_append_data(&mut self, data: String) {
        self.append_data.set_option(Some(data));
        self.data.get_type().set_node_is_configured(true);
    }
}

#[async_trait]
impl FBPNodeTrait for AppendNode<'static > {


    fn node_data_clone(self) -> FBPNodeContext<'static> {
        self.data.get_type().clone()
    }

    fn node_data(&self) -> &FBPNodeContext<'static> {
            &self.data.get_type().deref()
    }

    fn node_data_mut(&mut self) -> &mut FBPNodeContext<'static> {
        &mut self.data.get_type().deref()
    }

    // Here is an example of a node needing additional data before it can start processing
    // incoming IIDMessages.  The AppendNode FBP Node needs to be configured with the
    // string that will be appended to incoming messages.  That is why the process_config
    // method is implemented.  It will parse the incoming Config message and will then call
    // the set_append_data method after the string has been extracted from the payload.
    fn process_config(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
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
                    },
                    ConfigMessageType::Connect => {
                        // Deal with a Connect
                        // This is not implemented for this example
                    },
                    ConfigMessageType::Disconnect => {
                        // Deai with a Disconnect
                        // This is not implemented for this example
                    },
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
        let string_ref = self.append_data.get_option().unwrap().clone();

        if msg.payload().is_some() {
            let mut payload = msg.payload().as_ref().unwrap().clone();
            if self.append_data.is_some() {
                payload.push_str(string_ref.as_str());
            }

            let new_msg = IIDMessage::new(MessageType::Data, Some(payload));
            return Ok(new_msg);
        } else {
            if self.append_data.is_some() {
                let new_msg = IIDMessage::new(MessageType::Data, Some(string_ref));
                return Ok(new_msg);
            }
        }

        Ok(msg.clone())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoggerNode<'a> {
    data: ThreadSafeType<FBPNodeContext<'a>>,

    #[serde(skip)]
    log_file_path: ThreadSafeOptionType<String>, // Arc<Mutex<Option<String>>>,
}

// impl ConvertToNode for LoggerNode<'_> {}

impl<'a> LoggerNode<'a> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut result = LoggerNode {
            data: ThreadSafeType::new(FBPNodeContext::new("LoggerNode")), // Box::new(FBPNodeContext::new("LoggerNode")),
            log_file_path: ThreadSafeOptionType::new(None), // Arc::new(Mutex::new(None)),
        };

        result.clone().start();
        result
    }


    pub fn set_log_file_path(&mut self, log_file_path: String) {
        self.log_file_path.set_option(Some(log_file_path));

        // Ensure the File
        let string_ref = self.log_file_path.get_option().unwrap().clone();
        let file_path = Path::new(string_ref.as_str());
        let file = File::create(file_path).expect("Unable to create file");
        drop(file);

        self.data.get_type().set_node_is_configured(true);
    }

    #[allow(dead_code)]
    pub fn get_log_string(&self) -> Result<String, Error> {
        if self.log_file_path.is_none() {
            return Err(Error::new(ErrorKind::Other, "Cannot get log string until the node is setup"));
        }

        let mut contents = String::new();
        let string_ref = self.log_file_path.get_option().unwrap().clone();

        let file_path = Path::new(string_ref.as_str());
        let mut file = OpenOptions::new().read(true)
            .open(file_path)
            .expect("Failed to open file {} for reading");

        file.read_to_string(&mut contents)
            .expect("Failed to write contents to string");

        drop(file);
        Ok(contents)
    }

    pub fn log_string_to_file(&self, data: &String) -> Result<(), Error> {
        if self.log_file_path.is_none() {
            return Err(Error::new(ErrorKind::Other, "Cannot get log to file until the node is setup"));
        }

        let string_ref = self.log_file_path.get_option().as_ref().unwrap().clone(); 
        let file_path = Path::new(string_ref.as_str());

        let mut file = OpenOptions::new().append(true)
            .open(file_path)
            .expect("Failed to open file for append");

        let string_to_write = data.clone();
        let string_to_write = string_to_write.replace("\0", "");

        let _write_result = file.write(string_to_write.as_bytes());
        drop(file);
        Ok(())
    }
}

#[async_trait]
impl FBPNodeTrait for LoggerNode<'static> {

    fn node_data_clone(self) -> FBPNodeContext<'static> {
        self.data.get_type().deref().clone()
    }

    fn node_data(&self) -> &FBPNodeContext<'static> {
        &self.data.get_type().deref() 
    }

    fn node_data_mut(&mut self) -> &mut FBPNodeContext<'static> {
        &mut self.data.get_type().deref()
    }

    // Implement the process_config to se the log file path
    fn process_config(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
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
                                    let log_file_path = String::from(the_value.as_str().unwrap());
                                    self.set_log_file_path(log_file_path);
                                }
                            }
                        }
                    },
                    ConfigMessageType::Connect => {
                        // Deal with a Connect
                        // This is not implemented for this example
                    },
                    ConfigMessageType::Disconnect => {
                        // Deai with a Disconnect
                        // This is not implemented for this example
                    },
                };
            } //  if msg.payload.is_some()
        } // if msg.msg_type == MessageType::Config

        Ok(IIDMessage::new(MessageType::Invalid, None))
    }

    // Implement the process_message to do the work of this node by writing the log to a file
    fn process_message(&mut self, msg: IIDMessage) -> Result<IIDMessage, NodeError> {
        if msg.payload().is_some() {
            if self.log_string_to_file(&msg.clone().payload().as_ref().unwrap()).is_err() {
                return Err(NodeError::new("Failed to write message to log file"));
            }
        }

        Ok(msg.clone())
    }
}

pub trait NodeMake {
    fn make_node(&self) -> ThreadSafeType<FBPNodeContext>;
}

#[derive(Debug, PartialEq, EnumString)]
pub enum FBPNodes {
    PassthroughNode,
    AppendNode,
    LoggerNode,
}

impl NodeMake for FBPNodes {
    fn make_node(&self) -> ThreadSafeType<FBPNodeContext> {
        let result = match self {
            FBPNodes::PassthroughNode => {
                let a_node = PassthroughNode::new();
                let ctx: ThreadSafeType<FBPNodeContext> = a_node.data.clone();
                ctx
            },
            FBPNodes::AppendNode => {
                let node_box = ThreadSafeType::new(AppendNode::new());
                node_box.get_type().deref().data.clone()
            },
            FBPNodes::LoggerNode => {
                let node_box = ThreadSafeType::new(LoggerNode::new());
                node_box.get_type().deref().data.clone()
            }
        };
        result
    }
}

pub fn create_node_from_name(node_name: &String) -> Option<ThreadSafeType<FBPNodeContext>> {

    let mut result: Option<ThreadSafeType<FBPNodeContext>> = None;

    let nodes_enum_result = FBPNodes::from_str(node_name.as_str());

    if nodes_enum_result.is_ok() {
        let nodes_enum = nodes_enum_result.unwrap();

        let ctx = nodes_enum.make_node();
        result = Some(ctx);

    }

    result

}

/*--------------------------------------------------------------------------
    Unit Tests
  ------------------------------------------------------------------------- */


#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_creation() {
        let ctx_op = create_node_from_name(&"PassthroughNode".to_string());
        assert!(ctx_op.is_some());

        // the assumption is that the JSON was read and we know the type of node to make by name

        let pt_node_op = PassthroughNode::convert(ctx_op.unwrap());
        assert!(pt_node_op.is_some());

        let a_pt_node = pt_node_op.unwrap();

        let node_name = a_pt_node.node_data().node_name();

        println!("a_pt_node is a {} node", node_name)


    }

}

/*
#[allow(unused_macros)]
macro_rules! make_nodes {
    ($($x:ident),*) => {
        enum FBPNodes<'a> {
            $(
                $x($x),
            )*
        }

        impl NodeMake for FBPNodes<'_> {
            fn make_node(&self) -> Self {
                match self {
                    $(
                        FBPNodes::$x($x) => $x::new(),
                    )*
                }
            }
        }
    };
}

make_nodes!(PassthroughNode<'a>, AppendNode<'a>, LoggerNode<'a>);

 */



/*
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

lazy_static! {
    static ref FBP_NODES: Mutex<Hashmap<String, 
}
   

pub trait CreateNodeFromString {
    fn create_node(self: Self, json_string: &String) -> Self;
}



#[allow(unused_macros)]
macro_rules! make_nodes {
    ($($x:ident),*) => {
        enum FBPNodes {
            $(
                $x($x),
            )*
        }

        impl NodeCreator for FBPNodes {
            fn create_node(&self, json_string: &String) -> Self {
                match self {
                    $(
                        FBPNodes::$x(a_node) => a_node.create_node(json_string),
                    )*
                }
            }
        }
    };
}
*/
