// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcMutator` public type.

use std::sync::Arc;

use super::ArcMutatorFn;
use crate::ArcConditionalMutator;
use crate::Mutator;
use crate::Predicate;
use crate::mutators::macros::impl_mutator_clone;
use crate::mutators::macros::impl_mutator_common_methods;
use crate::mutators::macros::impl_mutator_debug_display;
use crate::mutators::macros::impl_shared_mutator_methods;

// ============================================================================
// 5. ArcMutator - Thread-Safe Shared Ownership Implementation
// ============================================================================

/// ArcMutator struct
///
/// A stateless mutator implementation based on `Arc<dyn Fn(&mut T) + Send +
/// Sync>` for thread-safe shared ownership scenarios. This type allows the
/// mutator to be safely shared and used across multiple threads.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Arc`, multiple owners allowed
/// - **Thread-Safe**: Implements `Send + Sync`, safe for concurrent use
/// - **Shared-receiver calls**: Uses `Fn`, so invocation does not require `&mut
///   self`; interior mutability and external side effects remain possible
/// - **Chainable**: Method chaining via `&self` (non-consuming)
///
/// # Use Cases
///
/// Choose `ArcMutator` when:
/// - The mutator needs to be shared across multiple threads for stateless
///   operations
/// - Concurrent task processing (e.g., thread pools)
/// - Thread safety is required (Send + Sync)
///
/// # Examples
///
/// ```rust
/// use qubit_function::{Mutator, ArcMutator};
///
/// let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
/// let clone = mutator.clone();
///
/// let mut value = 5;
/// mutator.apply(&mut value);
/// assert_eq!(value, 10);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcMutator<T> {
    /// The wrapped callback implementation.
    pub(super) function: ArcMutatorFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> ArcMutator<T> {
    // Generate common mutator methods (new, new_with_name, name, set_name,
    // noop)
    impl_mutator_common_methods!(ArcMutator<T>, (Fn(&mut T) + Send + Sync + 'static), |f| { Arc::new(f) });

    // Generate shared mutator methods (when, and_then, or_else, conversions)
    impl_shared_mutator_methods!(
        ArcMutator<T>,
        ArcConditionalMutator,
        ArcPredicate,
        Mutator,
        predicate_bounds = (Send + Sync + 'static),
        chained_bounds = (Send + Sync + 'static)
    );
}

impl<T> Mutator<T> for ArcMutator<T> {
    #[inline(always)]
    fn apply(&self, value: &mut T) {
        (self.function)(value)
    }
}

// Generate Clone trait implementation for mutator
impl_mutator_clone!(ArcMutator<T>);

// Generate Debug and Display trait implementations
impl_mutator_debug_display!(ArcMutator<T>);
