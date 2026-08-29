// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcStatefulMutator` public type.

use std::cell::RefCell;
use std::rc::Rc;

use super::RcMutMutatorFn;
use crate::Predicate;
use crate::RcConditionalStatefulMutator;
use crate::StatefulMutator;
use crate::mutators::macros::impl_mutator_clone;
use crate::mutators::macros::impl_mutator_common_methods;
use crate::mutators::macros::impl_mutator_debug_display;
use crate::mutators::macros::impl_shared_mutator_methods;

// ============================================================================
// 3. RcStatefulMutator - Single-Threaded Shared Ownership Implementation
// ============================================================================

/// Rc-based stateful mutator.
///
/// A stateful mutator based on `Rc<RefCell<dyn FnMut(&mut T)>>` for
/// single-threaded shared ownership scenarios. Clones share the callback and
/// its captured state.
///
/// # Features
///
/// - **Shared Ownership**: Cloneable via `Rc`, multiple owners allowed
/// - **Single-Threaded**: Not thread-safe, cannot be sent across threads
/// - **Interior Mutability**: Uses `RefCell` for runtime borrow checking
/// - **Mutable State**: Can modify captured environment via `FnMut`
/// - **Chainable**: Method chaining via `&self` (non-consuming)
/// - **Performance**: More efficient than `ArcStatefulMutator` (no locking)
///
/// # Use Cases
///
/// Choose `RcStatefulMutator` when:
/// - The mutator needs to be shared within a single thread
/// - Thread safety is not required
/// - Performance is important (avoiding lock overhead)
///
/// # Examples
///
/// ```rust
/// use qubit_function::{RcStatefulMutator, StatefulMutator};
///
/// let mut calls = 0;
/// let mutator = RcStatefulMutator::new(move |x: &mut i32| {
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
/// # Borrowing and reentrancy
///
/// Each call holds a mutable `RefCell` borrow while the user callback runs.
/// Synchronous re-entry through the same shared wrapper panics with a borrow
/// error. Mutations completed before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcStatefulMutator<T> {
    /// The wrapped callback implementation.
    pub(super) function: RcMutMutatorFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> RcStatefulMutator<T> {
    impl_mutator_common_methods!(RcStatefulMutator<T>, (FnMut(&mut T) + 'static), |f| Rc::new(
        RefCell::new(f)
    ));

    // Generate shared mutator methods (when, and_then, or_else, conversions)
    impl_shared_mutator_methods!(
        RcStatefulMutator<T>,
        RcConditionalStatefulMutator,
        RcPredicate,
        StatefulMutator,
        predicate_bounds = ('static),
        chained_bounds = ('static)
    );
}

impl<T> StatefulMutator<T> for RcStatefulMutator<T> {
    #[inline]
    fn apply(&mut self, value: &mut T) {
        let mut function = self.function.borrow_mut();
        function(value)
    }
}

// Use macro to generate Clone implementation
impl_mutator_clone!(RcStatefulMutator<T>);

// Generate Debug and Display trait implementations
impl_mutator_debug_display!(RcStatefulMutator<T>);
