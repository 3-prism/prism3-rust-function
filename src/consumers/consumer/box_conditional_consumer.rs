// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxConditionalConsumer` public type.

use crate::BoxConsumer;
use crate::BoxPredicate;
use crate::Consumer;
use crate::Predicate;
use crate::consumers::macros::impl_box_conditional_consumer;
use crate::consumers::macros::impl_conditional_consumer_debug_display;

// ============================================================================
// 7. BoxConditionalConsumer - Box-based Conditional Consumer
// ============================================================================

/// BoxConditionalConsumer struct
///
/// A conditional non-mutating consumer that only executes when a predicate is
/// satisfied. Uses `BoxConsumer` and `BoxPredicate` for single ownership
/// semantics.
///
/// This type is typically created by calling `BoxConsumer::when()` and is
/// designed to work with the `or_else()` method to create if-then-else logic.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, consumes `self` on use
/// - **Conditional Execution**: Only consumes when predicate returns `true`
/// - **Chainable**: Can add `or_else` branch to create if-then-else logic
/// - **Implements Consumer**: Can be used anywhere a `Consumer` is expected
/// - **Shared access**: Receives `&self` and `&T`; interior mutability and
///   external side effects remain possible
///
/// # Examples
///
/// ## Basic Conditional Execution
///
/// ```rust
/// use qubit_function::{Consumer, BoxConsumer};
///
/// let consumer = BoxConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// });
/// let conditional = consumer.when(|x: &i32| *x > 0);
///
/// conditional.accept(&5);  // Prints: Positive: 5
/// conditional.accept(&-5); // Does nothing
/// ```
///
/// ## With or_else Branch
///
/// ```rust
/// use qubit_function::{Consumer, BoxConsumer};
///
/// let consumer = BoxConsumer::new(|x: &i32| {
///     println!("Positive: {}", x);
/// })
/// .when(|x: &i32| *x > 0)
/// .or_else(|x: &i32| {
///     println!("Non-positive: {}", x);
/// });
///
/// consumer.accept(&5);  // Prints: Positive: 5
/// consumer.accept(&-5); // Prints: Non-positive: -5
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxConditionalConsumer<T> {
    /// The wrapped consumer callback.
    pub(super) consumer: BoxConsumer<T>,
    /// The predicate controlling conditional execution.
    pub(super) predicate: BoxPredicate<T>,
}

// Use macro to generate conditional consumer implementations
impl_box_conditional_consumer!(BoxConditionalConsumer<T>, BoxConsumer, Consumer);

// Consumer trait implementation
impl<T> Consumer<T> for BoxConditionalConsumer<T> {
    fn accept(&self, value: &T) {
        if self.predicate.test(value) {
            self.consumer.accept(value);
        }
    }
}

// Use macro to generate Debug and Display implementations
impl_conditional_consumer_debug_display!(BoxConditionalConsumer<T>);
