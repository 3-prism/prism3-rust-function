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
//! - **BiConsumer types**: Functions that consume two values without returning
//! - **Predicate types**: Functions that test values and return boolean
//! - **BiPredicate types**: Functions that test two values and return boolean
//! - **Supplier types**: Functions that produce values without input
//! - **Transformer types**: Specialized transformation utilities
//! - **Comparator types**: Functions that compare values and return ordering
//!
//! # Author
//!
//! Haixing Hu

pub mod bi_consumer;
pub mod bi_predicate;
pub mod comparator;
pub mod consumer;
pub mod consumer_once;
pub mod function;
pub mod function_mut;
pub mod function_once;
pub mod mutator;
pub mod predicate;
pub mod readonly_consumer;
pub mod supplier;
pub mod transformer;
pub mod transformer_mut;
pub mod transformer_once;

pub use bi_consumer::{ArcBiConsumer, BiConsumer, BoxBiConsumer, FnBiConsumerOps, RcBiConsumer};
pub use bi_predicate::{
    ArcBiPredicate, BiPredicate, BoxBiPredicate, FnBiPredicateOps, RcBiPredicate,
};
pub use comparator::{ArcComparator, BoxComparator, Comparator, FnComparatorOps, RcComparator};
pub use consumer::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};
pub use consumer_once::{BoxConsumerOnce, ConsumerOnce, FnConsumerOnceOps};
pub use function::{ArcFunction, BoxFunction, Function, RcFunction};
pub use function_mut::{ArcFunctionMut, BoxFunctionMut, FunctionMut, RcFunctionMut};
pub use function_once::{ArcFunctionOnce, BoxFunctionOnce, FunctionOnce, RcFunctionOnce};
pub use mutator::{ArcMutator, BoxMutator, FnMutatorOps, Mutator, RcMutator};
pub use predicate::{ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate};
pub use readonly_consumer::{
    ArcReadonlyConsumer, BoxReadonlyConsumer, FnReadonlyConsumerOps, RcReadonlyConsumer,
    ReadonlyConsumer,
};
pub use supplier::{ArcSupplier, BoxSupplier, FnSupplierOps, RcSupplier, Supplier};
pub use transformer::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};
pub use transformer_mut::{ArcTransformerMut, BoxTransformerMut, RcTransformerMut, TransformerMut};
pub use transformer_once::{
    ArcTransformerOnce, BoxTransformerOnce, RcTransformerOnce, TransformerOnce,
};
