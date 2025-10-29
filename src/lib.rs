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

// Module declarations
pub mod comparator;
pub mod consumers;
pub mod functions;
pub mod mutators;
pub mod predicates;
pub mod suppliers;
pub mod tester;
pub mod transformers;

// Re-export all types from submodules for backward compatibility

// BiConsumer - Fn(&T, &U)
pub use consumers::{
    ArcBiConsumer,
    ArcStatefulBiConsumer,
    BiConsumer,
    BiConsumerOnce,
    BoxBiConsumer,
    BoxBiConsumerOnce,
    BoxStatefulBiConsumer,
    FnBiConsumerOnceOps,
    FnBiConsumerOps,
    FnStatefulBiConsumerOps,
    RcBiConsumer,
    RcStatefulBiConsumer,
    StatefulBiConsumer,
};

// Consumer - Fn(&T)
pub use consumers::{
    ArcConsumer,
    ArcStatefulConsumer,
    BoxConsumer,
    BoxConsumerOnce,
    BoxStatefulConsumer,
    Consumer,
    ConsumerOnce,
    FnConsumerOnceOps,
    FnConsumerOps,
    FnStatefulConsumerOps,
    RcConsumer,
    RcStatefulConsumer,
    StatefulConsumer,
};

// Comparator - Fn(&T, &T) -> Ordering
pub use comparator::{
    ArcComparator,
    BoxComparator,
    Comparator,
    FnComparatorOps,
    RcComparator,
};

// Function - Fn(&T) -> R
pub use functions::{
    ArcConditionalFunction,
    ArcConditionalStatefulFunction,
    ArcFunction,
    ArcMutatingFunction,
    ArcStatefulFunction,
    ArcStatefulMutatingFunction,
    BoxConditionalFunction,
    BoxConditionalStatefulFunction,
    BoxFunction,
    BoxFunctionOnce,
    BoxMutatingFunction,
    BoxMutatingFunctionOnce,
    BoxStatefulFunction,
    BoxStatefulMutatingFunction,
    FnFunctionOnceOps,
    FnFunctionOps,
    FnMutStatefulMutatingFunctionOps,
    FnMutatingFunctionOps,
    FnOnceMutatingFunctionOps,
    FnStatefulFunctionOps,
    Function,
    FunctionOnce,
    MutatingFunction,
    MutatingFunctionOnce,
    RcConditionalFunction,
    RcConditionalStatefulFunction,
    RcFunction,
    RcMutatingFunction,
    RcStatefulFunction,
    RcStatefulMutatingFunction,
    StatefulFunction,
    StatefulMutatingFunction,
};

// Mutator - Fn(&mut T)
pub use mutators::{
    ArcConditionalMutator,
    ArcConditionalStatefulMutator,
    ArcMutator,
    ArcStatefulMutator,
    BoxConditionalMutator,
    BoxConditionalMutatorOnce,
    BoxConditionalStatefulMutator,
    BoxMutator,
    BoxMutatorOnce,
    BoxStatefulMutator,
    FnMutStatefulMutatorOps,
    FnMutatorOnceOps,
    FnMutatorOps,
    Mutator,
    MutatorOnce,
    RcConditionalMutator,
    RcConditionalStatefulMutator,
    RcMutator,
    RcStatefulMutator,
    StatefulMutator,
};

// Predicate - Fn(&T) -> bool
pub use predicates::{
    ArcBiPredicate,
    ArcPredicate,
    BiPredicate,
    BoxBiPredicate,
    BoxPredicate,
    FnBiPredicateOps,
    FnPredicateOps,
    Predicate,
    RcBiPredicate,
    RcPredicate,
};

// Supplier - Fn() -> R
pub use suppliers::{
    ArcStatefulSupplier,
    ArcSupplier,
    BoxStatefulSupplier,
    BoxSupplier,
    BoxSupplierOnce,
    FnStatefulSupplierOps,
    RcStatefulSupplier,
    RcSupplier,
    StatefulSupplier,
    Supplier,
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
pub use transformers::{
    ArcBiTransformer,
    ArcBinaryOperator,
    ArcConditionalStatefulBiTransformer,
    ArcConditionalStatefulTransformer,
    ArcConditionalTransformer,
    ArcStatefulBiTransformer,
    ArcStatefulTransformer,
    ArcTransformer,
    ArcUnaryOperator,
    BiTransformer,
    BiTransformerOnce,
    BinaryOperator,
    BinaryOperatorOnce,
    BoxBiTransformer,
    BoxBiTransformerOnce,
    BoxBinaryOperator,
    BoxBinaryOperatorOnce,
    BoxConditionalStatefulBiTransformer,
    BoxConditionalStatefulTransformer,
    BoxConditionalTransformer,
    BoxConditionalTransformerOnce,
    BoxStatefulBiTransformer,
    BoxStatefulTransformer,
    BoxTransformer,
    BoxTransformerOnce,
    BoxUnaryOperator,
    BoxUnaryOperatorOnce,
    FnBiTransformerOnceOps,
    FnBiTransformerOps,
    FnStatefulBiTransformerOps,
    FnStatefulTransformerOps,
    FnTransformerOnceOps,
    FnTransformerOps,
    RcBiTransformer,
    RcBinaryOperator,
    RcConditionalStatefulBiTransformer,
    RcConditionalStatefulTransformer,
    RcConditionalTransformer,
    RcStatefulBiTransformer,
    RcStatefulTransformer,
    RcTransformer,
    RcUnaryOperator,
    StatefulBiTransformer,
    StatefulTransformer,
    Transformer,
    TransformerOnce,
    UnaryOperator,
    UnaryOperatorOnce,
};
