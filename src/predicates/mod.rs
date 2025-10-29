/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Predicates Module
//!
//! This module provides predicate-related functional programming abstractions
//! for testing values and returning boolean results.
//!
//! # Author
//!
//! Haixing Hu

pub mod bi_predicate;
pub mod predicate;

pub use bi_predicate::{
    ArcBiPredicate,
    BiPredicate,
    BoxBiPredicate,
    FnBiPredicateOps,
    RcBiPredicate,
};
pub use predicate::{
    ArcPredicate,
    BoxPredicate,
    FnPredicateOps,
    Predicate,
    RcPredicate,
};
