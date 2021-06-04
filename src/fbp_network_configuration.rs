/* =================================================================================
    File:           fbp_network_configuration.rs

    Description:    This file provides the types and methods needed to define an FBP
                    network.

    History:        RustDev 04/06/2021   Code ported from original rustfbp crate

    Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
   ================================================================================== */

//! # FBP Network Configuration
//!
//! A Flow based programming system is created when a series of nodes are linked 
//! together to form a network of nodes.  The basis of this network is a node, along
//! with all of the configurations for that node, and the vector of nodes that wish
//! to receive the output of the node.  This construction forms the basis of the 
//! network

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Description of an FBP node in an FBP network
///  
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfigNode {
    pub node_name: String,

    #[serde(skip)]
    pub uuid: Uuid,

    pub configurations: Vec<String>,
    pub connections: ConfigNodeNetwork,
}

impl ConfigNode {
    /// Creates a new ConfigNode for a specific FBP Node
	///
	/// This will create a new ConfigNode.  All that is required is the name
    /// of the FBP struct
    /// 
	/// # Example
	///
	/// Basic usage:
	/// ```
	/// use fbp::fbp_confignode::*;
	///
	/// let a_confignode = ConfigNode::new("ExampleNode".to_string());
	///
	/// ```
    /// 
    pub fn new(node_name: String) -> Self {
        ConfigNode {
            node_name,
            uuid: Uuid::new_v4(),
            configurations: Vec::new(),
            connections: ConfigNodeNetwork {
                node_network: HashMap::new(),
            },
        }
    }

    /// Add Configuration data to an ConfigNode
    /// 
    /// While not necessary, if known before hand, it is nice to be able to 
    /// set the configuration information for a node.  An example might be
    /// the file path to a logging file or a TCP address.  The data should 
    /// be a JSON string of the following form: 
    ///  "{\"field_name_in_struct\":\"Value_to assigned_to_field\"}"
    /// 
    /// For example:  To set a log file path in a node that has a field name
    /// of log_file_path to be "Log_File.txt", one would have the following 
    /// configuration string: "{\"log_file_path\":\"Log_file.txt\"}"
    ///
    /// /// # Example
	///
	/// Basic usage:
	/// ```
	/// use fbp::fbp_confignode::*;
	///
	/// let a_confignode = ConfigNode::new("ExampleNode".to_string());
    /// let a_config_str =  "{\"log_file_path\":\"Log_file.txt\"}".to_string();
    /// a_confignode.add_configuration(a_config_str);
    /// 
	/// ```
    pub fn add_configuration(&mut self, config_str:String) {
        let configs = &mut self.configurations;
        configs.push(config_str);
    }

    pub fn add_connection(&mut self, node_name: String, key: Option<String>) ->  Option<&mut ConfigNode> {
        let mut hash_key = "Any".to_string();
        
        if key.is_some() {
            hash_key = key.unwrap();
        }
    
        let config_node = self.connections.add_node(node_name, Some(hash_key));
        config_node
   }

}
