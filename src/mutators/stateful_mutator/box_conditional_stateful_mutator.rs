// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxConditionalStatefulMutator` public type.

use crate::BoxPredicate;
use crate::BoxStatefulMutator;
use crate::Predicate;
use crate::StatefulMutator;
use crate::mutators::macros::impl_box_conditional_mutator;
use crate::mutators::macros::impl_conditional_mutator_debug_display;

// ============================================================================
// 7. BoxConditionalStatefulMutator - Box-based Conditional Stateful Mutator
// ============================================================================

/// Box-based conditional stateful mutator.
///
/// A conditional stateful mutator that only executes when a predicate is
/// satisfied. Uses `BoxStatefulMutator` and `BoxPredicate` for single ownership
/// semantics.
///
/// This type is typically created by calling `BoxStatefulMutator::when()` and
/// works with `or_else()` to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable; `apply` borrows `&mut self`
/// - **Conditional Execution**: Only mutates when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements StatefulMutator**: Can be used anywhere a `StatefulMutator`
///   is expected
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use qubit_function::{BoxStatefulMutator, StatefulMutator};
///
/// let mut calls = 0;
/// let mutator = BoxStatefulMutator::new(move |x: &mut i32| {
///     calls += 1;
///     *x += calls;
/// });
/// let mut conditional = mutator.when(|x: &i32| *x > 0);
///
/// let mut positive = 5;
/// conditional.apply(&mut positive);
/// assert_eq!(positive, 6);
///
/// let mut negative = -5;
/// conditional.apply(&mut negative);
/// assert_eq!(negative, -5); // Not executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use qubit_function::{BoxStatefulMutator, StatefulMutator};
///
/// let mut calls = 0;
/// let mut mutator = BoxStatefulMutator::new(move |x: &mut i32| {
///     calls += 1;
///     *x += calls;
/// })
///     .when(|x: &i32| *x > 0)
///     .or_else(|x: &mut i32| *x -= 1);
///
/// let mut positive = 5;
/// mutator.apply(&mut positive);
/// assert_eq!(positive, 6); // when branch executed
///
/// let mut negative = -5;
/// mutator.apply(&mut negative);
/// assert_eq!(negative, -6); // or_else branch executed
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxConditionalStatefulMutator<T> {
    /// The wrapped mutator callback.
    pub(super) mutator: BoxStatefulMutator<T>,
    /// The predicate controlling conditional execution.
    pub(super) predicate: BoxPredicate<T>,
}

// Generate box conditional mutator methods (and_then, or_else)
impl_box_conditional_mutator!(BoxConditionalStatefulMutator<T>, BoxStatefulMutator, StatefulMutator);

impl<T> StatefulMutator<T> for BoxConditionalStatefulMutator<T> {
    fn apply(&mut self, value: &mut T) {
        if self.predicate.test(value) {
            self.mutator.apply(value);
        }
    }
}

// Generate Debug and Display trait implementations for conditional mutator
impl_conditional_mutator_debug_display!(BoxConditionalStatefulMutator<T>);
