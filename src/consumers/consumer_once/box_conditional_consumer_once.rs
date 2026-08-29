// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxConditionalConsumerOnce` public type.

use crate::BoxConsumerOnce;
use crate::BoxPredicate;
use crate::ConsumerOnce;
use crate::Predicate;
use crate::consumers::macros::impl_box_conditional_consumer;
use crate::consumers::macros::impl_conditional_consumer_debug_display;

// ============================================================================
// 5. BoxConditionalConsumerOnce - Box-based Conditional Consumer
// ============================================================================

/// BoxConditionalConsumerOnce struct
///
/// A conditional one-time consumer that only executes when a predicate is
/// satisfied. Uses `BoxConsumerOnce` and `BoxPredicate` for single ownership
/// semantics.
///
/// This type is typically created by calling `BoxConsumerOnce::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements ConsumerOnce**: Can be used anywhere a `ConsumerOnce` is
///   expected
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use qubit_function::{ConsumerOnce, BoxConsumerOnce};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l = log.clone();
/// let consumer = BoxConsumerOnce::new(move |x: &i32| {
///     l.lock().expect("mutex should not be poisoned").push(*x);
/// });
/// let conditional = consumer.when(|x: &i32| *x > 0);
///
/// conditional.accept(&5);
/// assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]); // Executed
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use qubit_function::{ConsumerOnce, BoxConsumerOnce};
/// use std::sync::{Arc, Mutex};
///
/// let log = Arc::new(Mutex::new(Vec::new()));
/// let l1 = log.clone();
/// let l2 = log.clone();
/// let consumer = BoxConsumerOnce::new(move |x: &i32| {
///     l1.lock().expect("mutex should not be poisoned").push(*x);
/// })
/// .when(|x: &i32| *x > 0)
/// .or_else(move |x: &i32| {
///     l2.lock().expect("mutex should not be poisoned").push(-*x);
/// });
///
/// consumer.accept(&5);
/// assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]); // when branch executed
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxConditionalConsumerOnce<T> {
    /// The wrapped consumer callback.
    pub(super) consumer: BoxConsumerOnce<T>,
    /// The predicate controlling conditional execution.
    pub(super) predicate: BoxPredicate<T>,
}

// Generate and_then and or_else methods using macro
impl_box_conditional_consumer!(BoxConditionalConsumerOnce<T>, BoxConsumerOnce, ConsumerOnce);

impl<T> ConsumerOnce<T> for BoxConditionalConsumerOnce<T> {
    fn accept(self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }
}

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(BoxConditionalConsumerOnce<T>);
