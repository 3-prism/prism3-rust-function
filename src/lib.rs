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
//! - **Transformer types**: Transform values from type T to type R
//! - **UnaryOperator types**: Transform values of type T to the same type T
//! - **BiTransformer types**: Transform two values to produce a result
//! - **BinaryOperator types**: Transform two values of type T to produce a T
//! - **Consumer types**: Functions that consume values without returning
//! - **BiConsumer types**: Functions that consume two values without returning
//! - **Predicate types**: Functions that test values and return boolean
//! - **BiPredicate types**: Functions that test two values and return boolean
//! - **Supplier types**: Functions that produce values without input
//! - **Mapper types**: Stateful transformations from type T to type R
//! - **Tester types**: Functions that test conditions without input
//! - **Comparator types**: Functions that compare values and return ordering
//!
//! # Author
//!
//! Haixing Hu

pub mod bi_consumer;
pub mod bi_consumer_once;
pub mod bi_predicate;
pub mod bi_transformer;
pub mod bi_transformer_once;
pub mod comparator;
pub mod consumer;
pub mod consumer_once;
pub mod mapper;
pub mod mutator;
pub mod mutator_once;
pub mod predicate;
pub mod readonly_bi_consumer;
pub mod readonly_consumer;
pub mod readonly_supplier;
pub mod supplier;
pub mod supplier_once;
pub mod tester;
pub mod transformer;
pub mod transformer_once;

pub use bi_consumer::{ArcBiConsumer, BiConsumer, BoxBiConsumer, FnBiConsumerOps, RcBiConsumer};
pub use bi_consumer_once::{BiConsumerOnce, BoxBiConsumerOnce, FnBiConsumerOnceOps};
pub use bi_predicate::{
    ArcBiPredicate, BiPredicate, BoxBiPredicate, FnBiPredicateOps, RcBiPredicate,
};
pub use bi_transformer::{
    ArcBiTransformer, ArcBinaryOperator, BiTransformer, BinaryOperator, BoxBiTransformer,
    BoxBinaryOperator, FnBiTransformerOps, RcBiTransformer, RcBinaryOperator,
};
pub use bi_transformer_once::{
    BiTransformerOnce, BinaryOperatorOnce, BoxBiTransformerOnce, BoxBinaryOperatorOnce,
    FnBiTransformerOnceOps,
};
pub use comparator::{ArcComparator, BoxComparator, Comparator, FnComparatorOps, RcComparator};
pub use consumer::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};
pub use consumer_once::{BoxConsumerOnce, ConsumerOnce, FnConsumerOnceOps};
pub use mapper::{
    ArcConditionalMapper, ArcMapper, BoxConditionalMapper, BoxMapper, FnMapperOps, Mapper,
    RcConditionalMapper, RcMapper,
};
pub use mutator::{
    ArcConditionalMutator, ArcMutator, BoxConditionalMutator, BoxMutator, FnMutatorOps, Mutator,
    RcConditionalMutator, RcMutator,
};
pub use mutator_once::{BoxConditionalMutatorOnce, BoxMutatorOnce, FnMutatorOnceOps, MutatorOnce};
pub use predicate::{ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate};
pub use readonly_bi_consumer::{
    ArcReadonlyBiConsumer, BoxReadonlyBiConsumer, FnReadonlyBiConsumerOps, RcReadonlyBiConsumer,
    ReadonlyBiConsumer,
};
pub use readonly_consumer::{
    ArcReadonlyConsumer, BoxReadonlyConsumer, FnReadonlyConsumerOps, RcReadonlyConsumer,
    ReadonlyConsumer,
};
pub use readonly_supplier::{
    ArcReadonlySupplier, BoxReadonlySupplier, RcReadonlySupplier, ReadonlySupplier,
};
pub use supplier::{ArcSupplier, BoxSupplier, FnSupplierOps, RcSupplier, Supplier};
pub use supplier_once::{BoxSupplierOnce, SupplierOnce};
pub use tester::{ArcTester, BoxTester, FnTesterOps, RcTester, Tester};
pub use transformer::{
    ArcConditionalTransformer, ArcTransformer, ArcUnaryOperator, BoxConditionalTransformer,
    BoxTransformer, BoxUnaryOperator, FnTransformerOps, RcConditionalTransformer, RcTransformer,
    RcUnaryOperator, Transformer, UnaryOperator,
};
pub use transformer_once::{
    BoxConditionalTransformerOnce, BoxTransformerOnce, BoxUnaryOperatorOnce, FnTransformerOnceOps,
    TransformerOnce, UnaryOperatorOnce,
};
