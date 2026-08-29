// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxStatefulMutatingFunction` public type.

use crate::BoxConditionalStatefulMutatingFunction;
use crate::Function;
use crate::Predicate;
use crate::StatefulMutatingFunction;
use crate::functions::macros::impl_box_function_methods;
use crate::functions::macros::impl_function_common_methods;
use crate::functions::macros::impl_function_debug_display;
use crate::functions::macros::impl_function_identity_method;

// =======================================================================
// 3. BoxStatefulMutatingFunction - Single Ownership Implementation
// =======================================================================

/// BoxStatefulMutatingFunction struct
///
/// A stateful mutating function implementation based on
/// `Box<dyn FnMut(&mut T) -> R>` for single ownership scenarios. This is the
/// simplest and most efficient stateful mutating function type when sharing
/// is not required.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, ownership moves on use
/// - **Runtime cost**: One heap allocation and dynamic dispatch; no reference
///   counting or locking
/// - **Stateful**: Can modify captured environment (uses `FnMut`)
/// - **Builder Pattern**: Method chaining consumes `self` naturally
/// - **Factory Methods**: Convenient constructors for common patterns
///
/// # Use Cases
///
/// Choose `BoxStatefulMutatingFunction` when:
/// - The function needs to maintain internal state
/// - Building pipelines where ownership naturally flows
/// - No need to share the function across contexts
/// - Performance is critical and no sharing overhead is acceptable
///
/// # Performance
///
/// `BoxStatefulMutatingFunction` has the best performance among the three
/// function types:
/// - No reference counting overhead
/// - No lock acquisition or runtime borrow checking
/// - Direct function call through vtable
/// - Minimal memory footprint (single pointer)
///
/// # Examples
///
/// ```rust
/// use qubit_function::{StatefulMutatingFunction,
///                       BoxStatefulMutatingFunction};
///
/// let mut counter = {
///     let mut count = 0;
///     BoxStatefulMutatingFunction::new(move |x: &mut i32| {
///         count += 1;
///         *x *= 2;
///         count
///     })
/// };
/// let mut value = 5;
/// assert_eq!(counter.apply(&mut value), 1);
/// assert_eq!(value, 10);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxStatefulMutatingFunction<T, R> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn FnMut(&mut T) -> R>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, R> BoxStatefulMutatingFunction<T, R> {
    // Generates: new(), new_with_name(), new_with_optional_name(), name(),
    // set_name()
    impl_function_common_methods!(
        BoxStatefulMutatingFunction<T, R>,
        (FnMut(&mut T) -> R + 'static),
        |f| Box::new(f)
    );

    // Generates: when(), and_then(), compose()
    impl_box_function_methods!(
        BoxStatefulMutatingFunction<T, R>,
        BoxConditionalStatefulMutatingFunction,
        Function        // chains a non-mutating function after this mutating function
    );
}

// Generates: Debug and Display implementations for
// BoxStatefulMutatingFunction<T, R>
impl_function_debug_display!(BoxStatefulMutatingFunction<T, R>);

// Generates: identity() method for BoxStatefulMutatingFunction<T, T>
impl_function_identity_method!(BoxStatefulMutatingFunction<T, T>, mutating);

// Implement StatefulMutatingFunction trait for BoxStatefulMutatingFunction<T,
// R>
impl<T, R> StatefulMutatingFunction<T, R> for BoxStatefulMutatingFunction<T, R> {
    #[inline(always)]
    fn apply(&mut self, t: &mut T) -> R {
        (self.function)(t)
    }
}
