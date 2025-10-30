/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumers Module
//!
//! This module provides consumer-related functional programming abstractions,
//! including single-parameter consumers, bi-consumers, and their stateful
//! variants.
//!
//! # Author
//!
//! Haixing Hu

// do NOT use obsleted `#[macro_use]` syntax
pub mod macros;

pub mod bi_consumer;
pub mod bi_consumer_once;
pub mod consumer;
pub mod consumer_once;
pub mod stateful_bi_consumer;
pub mod stateful_consumer;

// Re-export macros from the macros module
pub(crate) use macros::*;

pub use bi_consumer::{
    ArcBiConsumer,
    BiConsumer,
    BoxBiConsumer,
    FnBiConsumerOps,
    RcBiConsumer,
};
pub use bi_consumer_once::{
    BiConsumerOnce,
    BoxBiConsumerOnce,
    FnBiConsumerOnceOps,
};
pub use consumer::{
    ArcConsumer,
    BoxConsumer,
    Consumer,
    FnConsumerOps,
    RcConsumer,
};
pub use consumer_once::{
    BoxConsumerOnce,
    ConsumerOnce,
    FnConsumerOnceOps,
};
pub use stateful_bi_consumer::{
    ArcStatefulBiConsumer,
    BoxStatefulBiConsumer,
    FnStatefulBiConsumerOps,
    RcStatefulBiConsumer,
    StatefulBiConsumer,
};
pub use stateful_consumer::{
    ArcStatefulConsumer,
    BoxStatefulConsumer,
    FnStatefulConsumerOps,
    RcStatefulConsumer,
    StatefulConsumer,
};
