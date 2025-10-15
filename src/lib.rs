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
//! Provides alias definitions for common function types, similar to functional interfaces in Java.
//! These type aliases simplify type declarations in functional programming, providing better readability and maintainability.
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
    ArcFnFunction, BoxFnFunction, BoxFunction, FnFunctionOps, Function, RcFnFunction,
};
pub use predicate::{ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate};
pub use supplier::{ArcSupplier, BoxSupplier, FnSupplierOps, RcSupplier, Supplier};
pub use transformer::{
    ArcFnTransformer, BoxFnTransformer, BoxTransformer, FnTransformerOps, RcFnTransformer,
    Transformer,
};
