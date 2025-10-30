/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Mutators Module
//!
//! This module provides mutator-related functional programming abstractions
//! for modifying values in-place through mutable references.
//!
//! # Author
//!
//! Haixing Hu

// 模块内部宏定义（不对外导出）
#[macro_use]
mod macros;

pub mod mutator;
pub mod mutator_once;
pub mod stateful_mutator;

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
pub use stateful_mutator::{
    ArcConditionalStatefulMutator,
    ArcStatefulMutator,
    BoxConditionalStatefulMutator,
    BoxStatefulMutator,
    FnMutStatefulMutatorOps,
    RcConditionalStatefulMutator,
    RcStatefulMutator,
    StatefulMutator,
};
