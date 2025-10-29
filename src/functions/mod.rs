/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Functions Module
//!
//! This module provides function-related functional programming abstractions
//! for transforming values from one type to another with reference semantics.
//!
//! # Author
//!
//! Haixing Hu

pub mod function;
pub mod function_once;
pub mod stateful_function;

pub use function::{
    ArcConditionalFunction,
    ArcFunction,
    BoxConditionalFunction,
    BoxFunction,
    FnFunctionOps,
    Function,
    RcConditionalFunction,
    RcFunction,
};
pub use function_once::{
    BoxFunctionOnce,
    FnFunctionOnceOps,
    FunctionOnce,
};
pub use stateful_function::{
    ArcConditionalStatefulFunction,
    ArcStatefulFunction,
    BoxConditionalStatefulFunction,
    BoxStatefulFunction,
    FnStatefulFunctionOps,
    RcConditionalStatefulFunction,
    RcStatefulFunction,
    StatefulFunction,
};
