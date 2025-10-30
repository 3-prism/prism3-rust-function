/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer Macros Module
//!
//! Provides declarative macros to simplify Consumer implementations and
//! reduce code duplication.
//!
//! # Author
//!
//! Haixing Hu

// Module declarations
mod box_conditional_consumer;
mod box_consumer_methods;
mod conditional_consumer_clone;
mod conditional_consumer_conversions;
mod conditional_consumer_debug_display;
mod consumer_clone;
mod consumer_common_methods;
mod consumer_debug_display;
mod shared_conditional_consumer;
mod shared_consumer_methods;

// Export all macros for use within the crate
pub(crate) use box_conditional_consumer::impl_box_conditional_consumer;
pub(crate) use box_consumer_methods::impl_box_consumer_methods;
pub(crate) use conditional_consumer_clone::impl_conditional_consumer_clone;
pub(crate) use conditional_consumer_conversions::impl_conditional_consumer_conversions;
pub(crate) use conditional_consumer_debug_display::impl_conditional_consumer_debug_display;
pub(crate) use consumer_clone::impl_consumer_clone;
pub(crate) use consumer_common_methods::impl_consumer_common_methods;
pub(crate) use consumer_debug_display::impl_consumer_debug_display;
pub(crate) use shared_conditional_consumer::impl_shared_conditional_consumer;
pub(crate) use shared_consumer_methods::impl_shared_consumer_methods;

