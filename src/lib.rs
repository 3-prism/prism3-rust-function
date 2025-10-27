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
pub mod function;
pub mod function_once;
pub mod mutator;
pub mod mutator_once;
pub mod predicate;
pub mod stateful_bi_consumer;
pub mod stateful_consumer;
pub mod stateful_function;
pub mod stateful_supplier;
pub mod stateful_transformer;
pub mod supplier;
pub mod supplier_once;
pub mod tester;
pub mod transformer;
pub mod transformer_once;

// BiConsumer - Fn(&T, &U)
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

// BiPredicate - Fn(&T, &U) -> bool
pub use bi_predicate::{
    ArcBiPredicate,
    BiPredicate,
    BoxBiPredicate,
    FnBiPredicateOps,
    RcBiPredicate,
};

// BiTransformer - Fn(T, U) -> R
pub use bi_transformer::{
    ArcBiTransformer,
    ArcBinaryOperator,
    BiTransformer,
    BinaryOperator,
    BoxBiTransformer,
    BoxBinaryOperator,
    FnBiTransformerOps,
    RcBiTransformer,
    RcBinaryOperator,
};
pub use bi_transformer_once::{
    BiTransformerOnce,
    BinaryOperatorOnce,
    BoxBiTransformerOnce,
    BoxBinaryOperatorOnce,
    FnBiTransformerOnceOps,
};

// Comparator - Fn(&T, &T) -> Ordering
pub use comparator::{
    ArcComparator,
    BoxComparator,
    Comparator,
    FnComparatorOps,
    RcComparator,
};

// Consumer - Fn(&T)
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

// Function - Fn(&T) -> R
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

// Mutator - Fn(&mut T)
pub use mutator::{
    ArcConditionalMutator,
    ArcMutator,
    BoxConditionalMutator,
    BoxMutator,
    FnMutatorOps,
    Mutator,
    RcConditionalMutator,
    RcMutator,
};
pub use mutator_once::{
    BoxConditionalMutatorOnce,
    BoxMutatorOnce,
    FnMutatorOnceOps,
    MutatorOnce,
};

// Predicate - Fn(&T) -> bool
pub use predicate::{
    ArcPredicate,
    BoxPredicate,
    FnPredicateOps,
    Predicate,
    RcPredicate,
};

// StatefulBiConsumer - FnMut(&T, &U)
pub use stateful_bi_consumer::{
    ArcStatefulBiConsumer,
    BoxStatefulBiConsumer,
    FnStatefulBiConsumerOps,
    RcStatefulBiConsumer,
    StatefulBiConsumer,
};

// StatefulConsumer - FnMut(&T)
pub use stateful_consumer::{
    ArcStatefulConsumer,
    BoxStatefulConsumer,
    FnStatefulConsumerOps,
    RcStatefulConsumer,
    StatefulConsumer,
};

// StatefulFunction - FnMut(&T) -> R
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

// StatefulSupplier - FnMut() -> R
pub use stateful_supplier::{
    ArcStatefulSupplier,
    BoxStatefulSupplier,
    FnStatefulSupplierOps,
    RcStatefulSupplier,
    StatefulSupplier,
};

// StatefulTransformer - FnMut(T) -> R
pub use stateful_transformer::{
    ArcConditionalStatefulTransformer,
    ArcStatefulTransformer,
    BoxConditionalStatefulTransformer,
    BoxStatefulTransformer,
    FnStatefulTransformerOps,
    RcConditionalStatefulTransformer,
    RcStatefulTransformer,
    StatefulTransformer,
};

// Supplier - Fn() -> R
pub use supplier::{
    ArcSupplier,
    BoxSupplier,
    RcSupplier,
    Supplier,
};
pub use supplier_once::{
    BoxSupplierOnce,
    SupplierOnce,
};

// Tester - FnMut() -> bool
pub use tester::{
    ArcTester,
    BoxTester,
    FnTesterOps,
    RcTester,
    Tester,
};

// Transformer - Fn(T) -> R
pub use transformer::{
    ArcConditionalTransformer,
    ArcTransformer,
    ArcUnaryOperator,
    BoxConditionalTransformer,
    BoxTransformer,
    BoxUnaryOperator,
    FnTransformerOps,
    RcConditionalTransformer,
    RcTransformer,
    RcUnaryOperator,
    Transformer,
    UnaryOperator,
};
pub use transformer_once::{
    BoxConditionalTransformerOnce,
    BoxTransformerOnce,
    BoxUnaryOperatorOnce,
    FnTransformerOnceOps,
    TransformerOnce,
    UnaryOperatorOnce,
};
