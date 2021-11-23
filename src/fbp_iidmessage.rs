/* =================================================================================
 File:           fbp_iidmessage.rs

 Description:    This file contains message struct and behavior for a Flow Based
                 Programming system

 History:        RustDev 03/31/2021   Code ported from original rustfbp crate

 Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
================================================================================== */

//! # FBP Message Struct
//!
//! A Flow based programming system is at its heart a message passing system.  The
//! IIDMessage struct represents a message that is sent between FBP nodes.
//!
//! The IIDMessage struct has a type and a subtype along with a payload.  The payload
//! can be any String, but it typically is a JSON string of a serialized struct.
//!

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Defines the type of an IIDMessage
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// A 'normal' data message to be processed by an FBP Node.
    Data,
    /// The configuration message is used to modify an FBP Node.
    Config,
    /// The process message is used to control the running of an FBP Node.
    Process,
    /// The invalid message is used to stop message propagation.
    Invalid,
}

/// Defines the sub-types of an IIDMessage
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubMessageType {
    /// A 'normal' message.  One that need to be processed
    Normal,
    /// This sub-message type specifies that this message is a reply to a previously sent messages.
    /// It is mostly used by the TCP system
    Reply,
}

/// Defines the types of Config messages that are the payload of an IIDMessage
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigMessageType {
    /// This specifies that the payload of this Config message is connection information to connect
    /// one nodes output to the input to another node
    Connect,
    /// This specifies that the payload of this Config message is the node to disconnect from the
    /// output of the receiving node
    Disconnect,
    /// This specifies that the payload of this Config message is a JSON string that sets the value
    /// of a field in the FBP node
    Field,
}

/// Defines the types of Process messages for an FBP node
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessMessageType {
    /// The Stop type tells the node to stop processing and quit its input thread
    Stop,
    /// The Suspend type tells the node to suspend processing message and just pass them through to
    /// any nodes that are to receive the output of this node.
    Suspend,
    /// The Restart type tells the node to re-start processing messages.  If the node was not
    /// previously suspended this will no nothing.
    Restart,
}

/// The MessageSerializer trait is used to serialize and deserialize structs so
/// that they may become the payload of an IIDMessage
///
///
/// # Example
/// ```
/// use fbp::fbp_iidmessage::*;
///
/// // config_data is a JSON string to set a path to a log file
/// let config_data = "{\"log_file_path\":\"Log_file.txt\"}".to_string();
///
/// // This creates the new ConfigMessage with the config data
/// let config_msg = ConfigMessage::new(ConfigMessageType::Field, Some(config_data.clone()) );
///
/// // This will create a new IIDMessage with the serialized Config message as its payload
/// let a_msg = config_msg.make_message(MessageType::Config);
///
/// // This trait can also be used to deserialize the payload of an IIDMessage
///
/// // When an config message is received it can use this trait to deserialize the payload
///  if a_msg.msg_type() == MessageType::Config {
///  	let a_config_msg: ConfigMessage = ConfigMessage::make_self_from_string(
/// 			a_msg
/// 				.payload()
/// 				.as_ref()
/// 				.unwrap()
/// 				.as_str());
///
/// 	// Process the config message.
/// }
///
/// ```

pub trait MessageSerializer {
    /// This will deserialize a JSON string that is a serialized Rust struct back into the original struct
    ///
    fn make_self_from_string<'a, T>(json_string: &'a str) -> T
    where
        T: std::marker::Sized + serde::Deserialize<'a>,
    {
        serde_json::from_str(json_string).unwrap()
    }

    /// This will take a struct and serialize that struct into a JSON string that will then become the
    /// payload of an IIDMesssage
    ///
    fn make_message(&self, msg_type: MessageType) -> IIDMessage
    where
        Self: std::marker::Sized + serde::Serialize,
    {
        let struct_string = serde_json::to_string(&self).unwrap();
        IIDMessage::new(msg_type, Some(struct_string))
    }
}

/// Provides the structure of a message that is sent between FBP nodes.
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IIDMessage {
    msg_type: MessageType,
    sub_msg_type: SubMessageType,
    payload: Option<String>,
}

impl IIDMessage {
    /// Creates a new IIDMessage with a specific type and optional payload
    ///
    /// An IIDMesage may or may not have a payload.  If no payload is required then
    /// None can be passed as the payload.
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// let a_msg = IIDMessage::new(MessageType::Data, Some("This is a payload".to_string()));
    ///
    /// ```
    pub fn new(msg_type: MessageType, payload: Option<String>) -> Self {
        IIDMessage {
            msg_type,
            sub_msg_type: SubMessageType::Normal,
            payload,
        }
    }

    /// Retrieves a message's msg_type
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// let a_msg = IIDMessage::new(MessageType::Data, Some("This is a payload".to_string()));
    ///
    /// match a_msg.msg_type() {
    /// 	MessageType::Data => { /* Deal with a Data Message */ },
    /// 	MessageType::Config => { /* Deal with a Config Message */ },
    /// 	MessageType::Process => { /* Deal with a Process Message */ },
    /// 	_ => { /* Invalid or unknown message type */ },
    /// };
    ///
    /// ```
    ///
    pub fn msg_type(&self) -> MessageType {
        self.msg_type
    }

    /// Retrieves a messages's subtype
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// pub fn deal_with_msg(a_msg: IIDMessage) {
    /// 	if a_msg.sub_msg_type() == SubMessageType::Reply {
    /// 		// ignore the message
    /// 	}
    /// }
    ///
    /// ```
    ///
    pub fn sub_msg_type(&self) -> SubMessageType {
        self.sub_msg_type
    }

    /// Retrieves a messages's payload
    ///
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// pub fn process_msg(a_msg: IIDMessage) {
    /// 	let msg_payload = a_msg.payload();
    /// 	if msg_payload.is_some() {
    /// 		// do something with the payload
    /// 	}
    /// }
    ///
    ///
    /// ```
    ///
    pub fn payload(&self) -> &Option<String> {
        &self.payload
    }

    /// Returns a mutable reference to the messages's submesssage type so that it maybe changed.
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// pub fn set_msg_reply(a_msg: &mut IIDMessage) {
    /// 	*a_msg.sub_msg_type_mut() =  SubMessageType::Reply;
    /// }
    ///
    /// ```
    ///
    pub fn sub_msg_type_mut(&mut self) -> &mut SubMessageType {
        &mut self.sub_msg_type
    }
}

/// Supports the ConfigMessageType::Field ConfigMessage for more complex
/// configurations.
///
/// Suppose that a field for a node needs to be generated from *base data*.
/// For example, suppose that a TCP FBP node that uses TLS needs a PKCS12
/// so that the certificate and private key can be used to secure the TCP
/// connection.  A PKCS12 file also will need a password.  Beyond that, one
/// might want to specify if the connection will use mutual authentification
/// or not and what specific cipher list should be used. A struct with this
/// information could be used to create the neccessary security information
/// for a TLS connection, but is NOT the data that is directly used for the
/// necessary fields of a TCP FBP node that uses TLS.  This is an example
/// of what the FieldConfiguration struct could be used for.
///
/// ## Example
/// ```
/// use serde::{Deserialize, Serialize};
///
/// use fbp::fbp_iidmessage::*;
/// // This struct holds the information for creating  the necessary data
/// // to use a secure TLS connection with TCP.
/// // 		file_path: 			This is a file path to a PKCS12 file
/// //		password: 			This is the password for the PKCS12 file
/// // 							if there is no password then this will be None
/// //		require_mutual_authentication:
/// //							This signifies if the TLS connect needs to be
/// //							mutually authenticated (Has both a server and
/// //							a client certificate)
/// //		cipher_list			The specific cipher list to use.  None means to
/// //							use the default cipher list.
/// //
/// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// pub struct SecurityData {
///     pub file_path: String,
///     pub password: Option<String>,
///     pub require_mutual_authentication: bool,
///     pub cipher_list: Option<String>,
/// }
///
/// impl SecurityData {
///     pub fn new(file_path: String,
///                require_mutual_authentication: bool,
///                password: Option<String>,
///                cipher_list: Option<String>) -> Self {
///         SecurityData {
///             file_path,
///             password,
///             require_mutual_authentication,
///             cipher_list
///         }
///     }
/// }
///
/// // Typically configuration JSON data will be stored either in a file or
/// // sent from another node.  This function shows how the FieldConfiguration
/// // struct can be used to create complex configuration data.
/// pub fn make_config_msg() -> IIDMessage {
/// 	let sd =  SecurityData::new("secureData.p12".to_string(),
/// 		true, Some("password".to_string()), None);
///
/// 	let sd_string = serde_json::to_string(&sd).unwrap();
///
/// 	let fc = FieldConfiguration::new("tcp_stream_handler".to_string(),
/// 		sd_string);
///
/// 	let fc_string = serde_json::to_string(&fc).unwrap();
///
/// 	let cm = ConfigMessage::new(ConfigMessageType::Field, Some(fc_string));
///
/// 	cm.make_message(MessageType::Config)
/// }
///
/// // The make_config_msg will return a IIDMessage that can be sent to a FBP
/// // node to configure that node.  The deal_with_config_message assumes that
/// // it will be called from the  fn process_config(self: &mut Self, msg: IIDMessage)
/// // trait message when a FBP node is being configured
/// pub fn deal_with_config_message(msg: IIDMessage) {
///
/// 	if msg.msg_type() != MessageType::Config {
/// 		return;
/// 	}
///
/// 	let payload = msg.payload().as_ref().unwrap();
///		let config_message: ConfigMessage = serde_json::from_str(&payload)
///			.expect("Failed to deserialize the config message");
///
/// 	let tcp_stream_handler_string = "tcp_stream_handler".to_string();
///
/// 	if config_message.data().as_ref().is_some() {
/// 		let fc: FieldConfiguration = serde_json::from_str(config_message.data().as_ref().unwrap()).unwrap();
/// 		match fc.get_field_name() {
/// 			tcp_stream_handler_string => {
/// 				let _sd: SecurityData = serde_json::from_str(fc.get_config_string().as_str()).unwrap();
/// 				// Now the SecurityData can be used to configure the TCP FBP Node.
/// 			},
/// 		_ => {
/// 			// Invalid field
/// 			}
/// 		};
/// 	}
/// }
///
/// ```
///
#[derive(Clone, Serialize, Deserialize)]
pub struct FieldConfiguration {
    field_name: String,
    config_string: String,
}

impl FieldConfiguration {
    pub fn new(field_name: String, config_string: String) -> Self {
        FieldConfiguration {
            field_name,
            config_string,
        }
    }

    pub fn get_field_name(&self) -> String {
        self.field_name.clone()
    }

    pub fn set_field_name(&mut self, new_field: String) {
        self.field_name = new_field.clone();
    }

    pub fn get_config_string(&self) -> String {
        self.config_string.clone()
    }

    pub fn set_config_string(&mut self, new_config: String) {
        self.config_string = new_config.clone();
    }
}

/// Provides the structure of the payload of an IIDMessage when the IIDMessage has a type
/// of MessageType::Config
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigMessage {
    msg_type: ConfigMessageType,
    data: Option<String>,
}

impl ConfigMessage {
    /// Creates a new ConfigMessage with a specific type and optional payload
    ///
    /// A ConfigMessage may or may not have a payload.  If no payload is required then
    /// None can be passed as the payload.
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// // Assuming that an FBP node has a field called log_file_path, set its value to be Log_file.txt.
    /// // This is done with a JSON string.
    /// let json_config_string = "{\"log_file_path\":\"Log_file.txt\"}".to_string();
    /// let a_msg = ConfigMessage::new(ConfigMessageType::Field, Some(json_config_string));
    ///
    /// ```
    pub fn new(msg_type: ConfigMessageType, data: Option<String>) -> Self {
        ConfigMessage { msg_type, data }
    }

    /// Retrieves an ConfigMessage's msg_type
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// let json_config_string = "{\"log_file_path\":\"Log_file.txt\"}".to_string();
    /// let c_msg = ConfigMessage::new(ConfigMessageType::Field, Some(json_config_string));
    /// let a_msg = c_msg.make_message(MessageType::Config);
    ///
    /// match a_msg.msg_type() {
    /// 	MessageType::Data => { /* Deal with a Data Message */ },
    /// 	MessageType::Config => {
    /// 		if a_msg.payload().is_some() {
    /// 			let a_config_msg: ConfigMessage =
    /// 				ConfigMessage::make_self_from_string(a_msg.payload().clone().unwrap().as_str());
    /// 			match a_config_msg.msg_type() {
    /// 				ConfigMessageType::Connect => {
    /// 					// Deal with a Connect
    /// 				},
    /// 				ConfigMessageType::Disconnect => {
    /// 					// Deal with a Disconnect
    /// 				},
    /// 				ConfigMessageType::Field => {
    /// 					// Deal with setting a field in an FBP node
    /// 				}
    /// 			};
    /// 		}
    /// 	},
    /// 	MessageType::Process => {
    /// 		// Deal with a process message
    /// 	},
    /// 	_ => {
    /// 		// Deal with an invalid or unknown message
    ///		},
    /// };
    ///
    ///
    /// ```
    ///
    pub fn msg_type(&self) -> ConfigMessageType {
        self.msg_type
    }

    /// Retrieves a Config messages's data
    ///
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// pub fn process_config_msg(a_msg: ConfigMessage) {
    /// 	let data = a_msg.data();
    /// 	if data.is_some() {
    /// 		// do something with the data
    /// 	}
    /// }
    ///
    ///
    /// ```
    ///
    pub fn data(&self) -> &Option<String> {
        &self.data
    }
}

impl MessageSerializer for ConfigMessage {}

/// Provides the structure of payload of an IIDMessage when the IIDMessage has a type
/// of MessageType::Process
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessMessage {
    msg: ProcessMessageType,
    propagate: bool,
    pub message_node: Option<Uuid>,
}

impl ProcessMessage {
    /// Creates a new ProcessMessage with a specific type and a flag to signal if the message
    /// shold be propagated
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// let p_msg = ProcessMessage::new(ProcessMessageType::Stop, true);
    ///
    /// ```
    pub fn new(msg: ProcessMessageType, propagate: bool) -> Self {
        ProcessMessage {
            msg,
            propagate,
            message_node: None,
        }
    }

    /// Retrieves a Process message's type
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// pub fn deal_with_process_msg(p_msg: ProcessMessage) {
    /// 	if p_msg.msg() == ProcessMessageType::Stop {
    /// 		//  Time to stop!
    /// 	}
    /// }
    ///
    /// ```
    ///
    pub fn msg(&self) -> ProcessMessageType {
        self.msg
    }

    /// Retrieves a Process message propagate flag
    ///
    /// # Example
    ///
    /// Basic usage:
    /// ```
    /// use fbp::fbp_iidmessage::*;
    ///
    /// pub fn should_propagate_process_msg(p_msg: ProcessMessage) -> bool {
    /// 	p_msg.propagate()
    /// }
    ///
    /// ```
    ///
    pub fn propagate(&self) -> bool {
        self.propagate
    }

    pub fn message_node(&self) -> &Option<Uuid> {
        &self.message_node
    }
}

impl MessageSerializer for ProcessMessage {}

/* --------------------------------------------------------------------------
Unit Tests
------------------------------------------------------------------------- */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iidmessage() {
        let msg_string = "foo".to_string();

        let mut msg = IIDMessage::new(MessageType::Data, Some(msg_string.clone()));

        assert_eq!(msg.msg_type(), MessageType::Data);
        assert_eq!(msg.payload().is_none(), false);
        assert_eq!(*msg.payload().as_ref().unwrap(), msg_string);
        assert_eq!(msg.sub_msg_type(), SubMessageType::Normal);

        *msg.sub_msg_type_mut() = SubMessageType::Reply;

        assert_eq!(msg.sub_msg_type(), SubMessageType::Reply);
    }

    #[test]
    fn test_config_message() {
        let config_data = "{\"log_file_path\":\"Log_file.txt\"}".to_string();
        let config_msg = ConfigMessage::new(ConfigMessageType::Field, Some(config_data.clone()));

        assert_eq!(config_msg.msg_type(), ConfigMessageType::Field);
        assert!(config_msg.data().is_some());
        assert_eq!(*config_msg.data().as_ref().unwrap(), config_data);

        // serialize the config message into a IIDMessage
        let a_msg = config_msg.make_message(MessageType::Config);

        assert_eq!(a_msg.msg_type(), MessageType::Config);
        assert!(a_msg.payload().is_some());

        let a_config_msg: ConfigMessage =
            ConfigMessage::make_self_from_string(a_msg.payload().as_ref().unwrap().as_str());

        assert_eq!(a_config_msg.msg_type(), ConfigMessageType::Field);
        assert!(a_config_msg.data().is_some());
        assert_eq!(*a_config_msg.data().as_ref().unwrap(), config_data);
    }

    #[test]
    fn test_process_message() {
        let prop_msg = ProcessMessage::new(ProcessMessageType::Stop, true);

        assert_eq!(prop_msg.msg(), ProcessMessageType::Stop);
        assert!(prop_msg.propagate());

        // serialize the process message into a IIDMessage
        let a_msg = prop_msg.make_message(MessageType::Process);

        assert_eq!(a_msg.msg_type(), MessageType::Process);
        assert!(a_msg.payload().is_some());

        let a_prop_msg: ProcessMessage =
            ProcessMessage::make_self_from_string(a_msg.payload().as_ref().unwrap().as_str());

        assert_eq!(a_prop_msg.msg(), ProcessMessageType::Stop);
        assert!(a_prop_msg.propagate());
    }
}
