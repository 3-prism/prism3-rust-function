// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxStatefulMutator` public type.

use crate::BoxConditionalStatefulMutator;
use crate::Predicate;
use crate::StatefulMutator;
use crate::mutators::macros::impl_box_mutator_methods;
use crate::mutators::macros::impl_mutator_common_methods;
use crate::mutators::macros::impl_mutator_debug_display;

// ============================================================================
// 3. BoxStatefulMutator - Single Ownership Implementation
// ============================================================================

/// Box-based stateful mutator.
///
/// A stateful mutator based on `Box<dyn FnMut(&mut T)>` for single ownership
/// scenarios. This is the simplest stateful mutator when sharing is not
/// required.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable; `apply` borrows `&mut self`
/// - **Runtime cost**: One heap allocation and dynamic dispatch; no reference
///   counting or locking
/// - **Mutable State**: Can modify captured environment via `FnMut`
/// - **Builder Pattern**: Composition methods consume `self` naturally
/// - **Factory Methods**: Convenient constructors for common patterns
///
/// # Use Cases
///
/// Choose `BoxStatefulMutator` when:
/// - The mutator is used only once or in a linear flow
/// - Building pipelines where ownership naturally flows
/// - No need to share the mutator across contexts
/// - Performance is critical and no sharing overhead is acceptable
///
/// # Performance
///
/// `BoxStatefulMutator` has the least sharing overhead among the three stateful
/// mutator wrappers:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use qubit_function::{BoxStatefulMutator, StatefulMutator};
///
/// let mut calls = 0;
/// let mut mutator = BoxStatefulMutator::new(move |x: &mut i32| {
///     calls += 1;
///     *x += calls;
/// });
/// let mut value = 10;
/// mutator.apply(&mut value);
/// assert_eq!(value, 11);
/// mutator.apply(&mut value);
/// assert_eq!(value, 13);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxStatefulMutator<T> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn FnMut(&mut T)>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> BoxStatefulMutator<T> {
    impl_mutator_common_methods!(BoxStatefulMutator<T>, (FnMut(&mut T) + 'static), |f| { Box::new(f) });

    // Generate box mutator methods (when, and_then, or_else, etc.)
    impl_box_mutator_methods!(BoxStatefulMutator<T>, BoxConditionalStatefulMutator, StatefulMutator);
}

impl<T> StatefulMutator<T> for BoxStatefulMutator<T> {
    #[inline(always)]
    fn apply(&mut self, value: &mut T) {
        (self.function)(value)
    }
}

// Generate Debug and Display trait implementations
impl_mutator_debug_display!(BoxStatefulMutator<T>);
