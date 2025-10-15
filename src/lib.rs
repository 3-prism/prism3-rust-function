/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Prism3 Function
//!
//! Provides functional programming abstractions for Rust, including:
//!
//! - **Function types**: Immutable, mutable, and consuming transformations
//! - **Consumer types**: Functions that consume values without returning
//! - **Predicate types**: Functions that test values and return boolean
//! - **Supplier types**: Functions that produce values without input
//! - **Transformer types**: Specialized transformation utilities
//!
//! # Author
//!
//! Haixing Hu

pub mod consumer;
pub mod function;
pub mod predicate;
pub mod supplier;
pub mod transformer;

pub use consumer::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};
pub use function::{
    ArcFunction, ArcFunctionMut, ArcFunctionOnce, BoxFunction, BoxFunctionMut,
    BoxFunctionOnce, Function, FunctionMut, FunctionOnce, RcFunction,
    RcFunctionMut, RcFunctionOnce,
};
pub use predicate::{ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate};
pub use supplier::{ArcSupplier, BoxSupplier, FnSupplierOps, RcSupplier, Supplier};
pub use transformer::{
    ArcFnTransformer, BoxFnTransformer, BoxTransformer, FnTransformerOps, RcFnTransformer,
    Transformer,
};
