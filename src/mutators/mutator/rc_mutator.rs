// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcMutator` public type.

use std::rc::Rc;

use super::RcMutatorFn;
use crate::Mutator;
use crate::Predicate;
use crate::RcConditionalMutator;
use crate::mutators::macros::impl_mutator_clone;
use crate::mutators::macros::impl_mutator_common_methods;
use crate::mutators::macros::impl_mutator_debug_display;
use crate::mutators::macros::impl_shared_mutator_methods;

// ============================================================================
// 4. RcMutator - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// RcMutator struct
///
/// A stateless mutator implementation based on `Rc<dyn Fn(&mut T)>` for
/// single-threaded shared ownership scenarios. This type allows multiple
/// references to the same mutator without the overhead of thread safety.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Shared-receiver calls**: Uses `Fn`, so invocation does not require `&mut
///   self`; interior mutability and external side effects remain possible
/// - **Chainable**: Method chaining via `&self` (non-consuming)
/// - **Ownership Cost**: `Rc` uses non-atomic reference counting; `Arc` uses
///   atomic reference counting, and neither wrapper locks callback invocation
///
/// # Use Cases
///
/// Choose `RcMutator` when:
/// - The mutator needs to be shared within a single thread for stateless
///   operations
/// - Thread safety is not required
/// - Single-threaded shared ownership without atomic reference counting is
///   preferred; benchmark the real workload when performance matters
///
/// # Examples
///
/// ```rust
/// use qubit_function::{Mutator, RcMutator};
///
/// let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
/// let clone = mutator.clone();
///
/// let mut value = 5;
/// mutator.apply(&mut value);
/// assert_eq!(value, 10);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcMutator<T> {
    /// The wrapped callback implementation.
    pub(super) function: RcMutatorFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> RcMutator<T> {
    // Generate common mutator methods (new, new_with_name, name, set_name,
    // noop)
    impl_mutator_common_methods!(RcMutator<T>, (Fn(&mut T) + 'static), |f| { Rc::new(f) });

    // Generate shared mutator methods (when, and_then, or_else, conversions)
    impl_shared_mutator_methods!(
        RcMutator<T>,
        RcConditionalMutator,
        RcPredicate,
        Mutator,
        predicate_bounds = ('static),
        chained_bounds = ('static)
    );
}

impl<T> Mutator<T> for RcMutator<T> {
    #[inline(always)]
    fn apply(&self, value: &mut T) {
        (self.function)(value)
    }
}

// Generate Clone trait implementation for mutator
impl_mutator_clone!(RcMutator<T>);

// Generate Debug and Display trait implementations
impl_mutator_debug_display!(RcMutator<T>);
