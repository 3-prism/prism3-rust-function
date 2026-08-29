// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxBiMutatingFunctionOnce` public type.

use crate::BiMutatingFunctionOnce;
use crate::BiPredicate;
use crate::BoxConditionalBiMutatingFunctionOnce;
use crate::MutatingFunctionOnce;
use crate::functions::macros::impl_box_function_methods;
use crate::functions::macros::impl_function_common_methods;
use crate::functions::macros::impl_function_constant_method;
use crate::functions::macros::impl_function_debug_display;
use crate::macros::impl_closure_once_trait;

/// The erased callback representation used by this implementation.
type BoxBiMutatingFunctionOnceFn<T, U, R> = Box<dyn FnOnce(&mut T, &mut U) -> R>;

// ============================================================================
// BoxBiMutatingFunctionOnce - Box<dyn FnOnce(&mut T, &mut U) -> R>
// ============================================================================

/// BoxBiMutatingFunctionOnce - consuming bi-mutating-function wrapper based on
/// `Box<dyn FnOnce>`
///
/// A bi-mutating-function wrapper that provides single ownership with one-time
/// use semantics. Consumes self and borrows both input values mutably.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnOnce(&mut T, &mut U) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can only be called once (consumes self)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxBiMutatingFunctionOnce<T, U, R> {
    /// The wrapped callback implementation.
    pub(super) function: BoxBiMutatingFunctionOnceFn<T, U, R>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

// Implement BoxBiMutatingFunctionOnce
impl<T, U, R> BoxBiMutatingFunctionOnce<T, U, R> {
    // Generate new(), new_with_name(), new_with_optional_name(), name(),
    // set_name()
    impl_function_common_methods!(
        BoxBiMutatingFunctionOnce<T, U, R>,
        (FnOnce(&mut T, &mut U) -> R + 'static),
        |f| Box::new(f)
    );

    // Generate when(), and_then()
    impl_box_function_methods!(
        BoxBiMutatingFunctionOnce<T, U, R>,
        BoxConditionalBiMutatingFunctionOnce,
        MutatingFunctionOnce
    );
}

// Implement BiMutatingFunctionOnce trait for BoxBiMutatingFunctionOnce
impl<T, U, R> BiMutatingFunctionOnce<T, U, R> for BoxBiMutatingFunctionOnce<T, U, R> {
    #[inline(always)]
    fn apply(self, first: &mut T, second: &mut U) -> R {
        (self.function)(first, second)
    }
}

// Implement constant method for BoxBiMutatingFunctionOnce
impl_function_constant_method!(BoxBiMutatingFunctionOnce<T, U, R>, mut 'static);

// Use macro to generate Debug and Display implementations
impl_function_debug_display!(BoxBiMutatingFunctionOnce<T, U, R>);

// ============================================================================
// Blanket implementation for standard FnOnce trait
// ============================================================================

// Implement BiMutatingFunctionOnce for all FnOnce(&mut T, &mut U) -> R using
// macro
impl_closure_once_trait!(
    BiMutatingFunctionOnce<T, U, R>,
    apply,
    BoxBiMutatingFunctionOnce,
    FnOnce(first: &mut T, second: &mut U) -> R
);
