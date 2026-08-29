// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcStatefulMutatingFunction` public type.

use std::sync::Arc;

use parking_lot::Mutex;

use super::ArcStatefulMutatingFunctionFn;
use crate::ArcConditionalStatefulMutatingFunction;
use crate::Function;
use crate::Predicate;
use crate::StatefulMutatingFunction;
use crate::functions::macros::impl_function_clone;
use crate::functions::macros::impl_function_common_methods;
use crate::functions::macros::impl_function_debug_display;
use crate::functions::macros::impl_function_identity_method;
use crate::functions::macros::impl_shared_function_methods;

// =======================================================================
// 5. ArcStatefulMutatingFunction - Thread-Safe Shared Ownership
// =======================================================================

/// ArcStatefulMutatingFunction struct
///
/// A stateful mutating function implementation based on
/// `Arc<Mutex<dyn FnMut(&mut T) -> R + Send>>` for thread-safe shared
/// ownership scenarios. This type allows the function to be safely shared
/// and used across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Stateful**: Can modify captured environment (uses `FnMut`)
/// - **Chainable**: Method chaining via `&self` (non-consuming)
///
/// # Use Cases
///
/// Choose `ArcStatefulMutatingFunction` when:
/// - The function needs to be shared across multiple threads for stateful
///   operations
/// - Concurrent task processing (e.g., thread pools)
/// - Thread safety is required (Send + Sync)
///
/// # Examples
///
/// ```rust
/// use qubit_function::{StatefulMutatingFunction,
///                       ArcStatefulMutatingFunction};
///
/// let counter = {
///     let mut count = 0;
///     ArcStatefulMutatingFunction::new(move |x: &mut i32| {
///         count += 1;
///         *x *= 2;
///         count
///     })
/// };
/// let mut clone = counter.clone();
///
/// let mut value = 5;
/// assert_eq!(clone.apply(&mut value), 1);
/// ```
///
/// # Locking and reentrancy
///
/// Each call acquires a `parking_lot::Mutex` and holds it while the user
/// callback runs. Synchronous re-entry through the same shared state
/// deadlocks. The mutex is not poisoned after a panic, and mutations completed
/// before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcStatefulMutatingFunction<T, R> {
    /// The wrapped callback implementation.
    pub(super) function: ArcStatefulMutatingFunctionFn<T, R>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, R> ArcStatefulMutatingFunction<T, R> {
    // Generates: new(), new_with_name(), new_with_optional_name(), name(),
    // set_name()
    impl_function_common_methods!(
        ArcStatefulMutatingFunction<T, R>,
        (FnMut(&mut T) -> R + Send + 'static),
        |f| Arc::new(Mutex::new(f))
    );

    // Generates: when(), and_then(), compose()
    impl_shared_function_methods!(
        ArcStatefulMutatingFunction<T, R>,
        ArcConditionalStatefulMutatingFunction,
        ArcPredicate,
        Function,  // chains a non-mutating function after this mutating function
        predicate_bounds = (Send + Sync + 'static),
        chained_bounds = (Send + 'static)
    );
}

// Generates: Clone implementation for ArcStatefulMutatingFunction<T, R>
impl_function_clone!(ArcStatefulMutatingFunction<T, R>);

// Generates: Debug and Display implementations for
// ArcStatefulMutatingFunction<T, R>
impl_function_debug_display!(ArcStatefulMutatingFunction<T, R>);

// Generates: identity() method for ArcStatefulMutatingFunction<T, T>
impl_function_identity_method!(ArcStatefulMutatingFunction<T, T>, mutating);

// Implement StatefulMutatingFunction trait for ArcStatefulMutatingFunction<T,
// R>
impl<T, R> StatefulMutatingFunction<T, R> for ArcStatefulMutatingFunction<T, R> {
    #[inline]
    fn apply(&mut self, t: &mut T) -> R {
        let mut function = self.function.lock();
        function(t)
    }
}
