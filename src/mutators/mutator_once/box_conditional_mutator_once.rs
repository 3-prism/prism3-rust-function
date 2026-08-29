// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxConditionalMutatorOnce` public type.

use crate::BoxMutatorOnce;
use crate::BoxPredicate;
use crate::MutatorOnce;
use crate::Predicate;
use crate::mutators::macros::impl_box_conditional_mutator;
use crate::mutators::macros::impl_conditional_mutator_debug_display;

// ============================================================================
// 5. BoxConditionalMutatorOnce - Box-based Conditional Mutator
// ============================================================================

/// BoxConditionalMutatorOnce struct
///
/// A conditional one-time mutator that only executes when a predicate is
/// satisfied. Uses `BoxMutatorOnce` and `BoxPredicate` for single ownership
/// semantics.
///
/// This type is typically created by calling `BoxMutatorOnce::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only mutates when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements MutatorOnce**: Can be used anywhere a `MutatorOnce` is
///   expected
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use qubit_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data = vec![1, 2, 3];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data);
/// });
/// let conditional = mutator.when(|x: &Vec<i32>| !x.is_empty());
///
/// let mut target = vec![0];
/// conditional.apply(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3]); // Executed
///
/// let mut empty = Vec::new();
/// let data2 = vec![4, 5];
/// let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data2);
/// });
/// let conditional2 = mutator2.when(|x: &Vec<i32>| x.len() > 5);
/// conditional2.apply(&mut empty);
/// assert_eq!(empty, Vec::<i32>::new()); // Not executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use qubit_function::{MutatorOnce, BoxMutatorOnce};
///
/// let data1 = vec![1, 2, 3];
/// let data2 = vec![99];
/// let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data1);
/// })
/// .when(|x: &Vec<i32>| !x.is_empty())
/// .or_else(move |x: &mut Vec<i32>| {
///     x.extend(data2);
/// });
///
/// let mut target = vec![0];
/// mutator.apply(&mut target);
/// assert_eq!(target, vec![0, 1, 2, 3]); // when branch executed
///
/// let data3 = vec![4, 5];
/// let data4 = vec![99];
/// let mutator2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
///     x.extend(data3);
/// })
/// .when(|x: &Vec<i32>| x.is_empty())
/// .or_else(move |x: &mut Vec<i32>| {
///     x.extend(data4);
/// });
///
/// let mut target2 = vec![0];
/// mutator2.apply(&mut target2);
/// assert_eq!(target2, vec![0, 99]); // or_else branch executed
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxConditionalMutatorOnce<T> {
    /// The wrapped mutator callback.
    pub(super) mutator: BoxMutatorOnce<T>,
    /// The predicate controlling conditional execution.
    pub(super) predicate: BoxPredicate<T>,
}

// Generate and_then and or_else methods using macro
impl_box_conditional_mutator!(BoxConditionalMutatorOnce<T>, BoxMutatorOnce, MutatorOnce);

impl<T> MutatorOnce<T> for BoxConditionalMutatorOnce<T> {
    fn apply(self, value: &mut T) {
        if self.predicate.test(value) {
            self.mutator.apply(value);
        }
    }
}

// Use macro to generate Debug and Display implementations
impl_conditional_mutator_debug_display!(BoxConditionalMutatorOnce<T>);
