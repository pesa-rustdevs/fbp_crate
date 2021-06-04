/* =================================================================================
 File:           lib.rs

 Description:    This file contains the modules that are used by the FBP crate

 History:        RustDev 03/31/2021   Code ported from original rustfbp crate

 Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
================================================================================== */

//! Flow Based Programming Crate For Rust
//!
//! Provides a basic Flow Based Programming (FBP) system for Rust
//!
//! # Introduction To Flow Based Programming
//!
//! In the early days of microprocessor development, Moore’s law was
//! achieved by increasing the number of transistors and the
//! frequency of the chips.  The chips while becoming more powerful
//! with each release, were still focused on a single CPU or core.  
//! In modern systems, the increase of performance is mostly dictated
//! by the number of CPUs or cores on a microprocessor.  
//! This change has lead to the need for a different type of programming model.  
//! This model must be factored into many threads of execution to use
//! all the processing power of these new microprocessors.
//!
//! This need for splitting a system into multiple threads has been fraught with problems.  
//! Originally, the model was to use locking of shared resources so that thread contention
//! would not cause one thread to corrupt another thread.  This model led to many problems
//! with locks and given that most developers are not trained in disparate threads of
//! execution, many ‘bugs’ were introduced.
//!
//! Flow Based programming was designed to address the issues of multiple cores and
//! to do so without using or needing locks.  The basic concept is that an application
//! is broken down to discreet blocks of work, much like an assembly line in a
//! manufacturing plant.  Work comes to the block when an item is placed onto an input queue.   
//! The block waits (is quiescent) until a work item is enqueued onto the input queue.  
//! At that time, the work item is processed and then placed onto one or more blocks
//! that have registered interest in the output of the block. This arrangement does not
//! use locks and each block in the system only knows what other nodes are interested
//! in its output.  It has no idea what the other blocks do or how big the network of
//! blocks are that makes up the application.  As suggested previously, this mimics the
//! layout of a manufacturing assembly line.  It also allows for breaking down the
//! functionality of a system to small understandable blocks of processing.
//!
//! This crate is designed as a Flow Based Programming system.  All functionality
//! is done through a message passing system that enqueues work onto various ‘nodes’ or blocks.
//!
//! This model also makes it much easier to scale a system as message passing can be done over TCP
//! and thus distribute the application across multiple systems.
//!
//! To read more about Flow Based Programming please see
//! [Wikipedia Flow Based Programming](https://en.wikipedia.org/wiki/Flow-based_programming)
//!
//! # Rust Flow Based Programming
//!
//! This implementation of Flow Based Programming (FBP) is not 'traditional'.  It uses some of the
//! built in Rust features to implement the system.

pub mod fbp_asyncstate;
pub mod fbp_iidmessage;
pub mod fbp_node_context;
pub mod fbp_node_error;
pub mod fbp_node_network;
pub mod fbp_node_trait;
pub mod fbp_threadsafe_wrapper;
