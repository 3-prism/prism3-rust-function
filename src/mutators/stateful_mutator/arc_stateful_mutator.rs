// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcStatefulMutator` public type.

use std::sync::Arc;

use parking_lot::Mutex;

use super::ArcMutMutatorFn;
use crate::ArcConditionalStatefulMutator;
use crate::Predicate;
use crate::StatefulMutator;
use crate::mutators::macros::impl_mutator_clone;
use crate::mutators::macros::impl_mutator_common_methods;
use crate::mutators::macros::impl_mutator_debug_display;
use crate::mutators::macros::impl_shared_mutator_methods;

// ============================================================================
// 4. ArcStatefulMutator - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// Arc-based stateful mutator.
///
/// A stateful mutator based on `Arc<Mutex<dyn FnMut(&mut T) + Send>>` for
/// thread-safe shared ownership scenarios. Clones share the callback and its
/// captured state.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Interior Mutability**: Uses `Mutex` for safe concurrent mutations
/// - **Mutable State**: Can modify captured environment via `FnMut`
/// - **Chainable**: Method chaining via `&self` (non-consuming)
///
/// # Use Cases
///
/// Choose `ArcStatefulMutator` when:
/// - The mutator needs to be shared across multiple threads
/// - Concurrent task processing (e.g., thread pools)
/// - Thread safety is required (Send + Sync)
///
/// # Examples
///
/// ```rust
/// use qubit_function::{ArcStatefulMutator, StatefulMutator};
///
/// let mut calls = 0;
/// let mutator = ArcStatefulMutator::new(move |x: &mut i32| {
///     calls += 1;
///     *x += calls;
/// });
/// let mut first = mutator.clone();
/// let mut second = mutator;
///
/// let mut value = 10;
/// first.apply(&mut value);
/// assert_eq!(value, 11);
/// second.apply(&mut value);
/// assert_eq!(value, 13);
/// ```
///
/// # Locking and reentrancy
///
/// Each call acquires a `parking_lot::Mutex` and holds it while the user
/// callback runs. Synchronous re-entry through the same shared state
/// deadlocks. The mutex is not poisoned after a panic, and mutations completed
/// before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcStatefulMutator<T> {
    /// The wrapped callback implementation.
    pub(super) function: ArcMutMutatorFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> ArcStatefulMutator<T> {
    impl_mutator_common_methods!(ArcStatefulMutator<T>, (FnMut(&mut T) + Send + 'static), |f| Arc::new(
        Mutex::new(f)
    ));

    // Generate shared mutator methods (when, and_then, or_else, conversions)
    impl_shared_mutator_methods!(
        ArcStatefulMutator<T>,
        ArcConditionalStatefulMutator,
        ArcPredicate,
        StatefulMutator,
        predicate_bounds = (Send + Sync + 'static),
        chained_bounds = (Send + 'static)
    );
}

impl<T> StatefulMutator<T> for ArcStatefulMutator<T> {
    #[inline]
    fn apply(&mut self, value: &mut T) {
        let mut function = self.function.lock();
        function(value)
    }
}

// Use macro to generate Clone implementation
impl_mutator_clone!(ArcStatefulMutator<T>);

// Generate Debug and Display trait implementations
impl_mutator_debug_display!(ArcStatefulMutator<T>);
