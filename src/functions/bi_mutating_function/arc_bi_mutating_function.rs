// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcBiMutatingFunction` public type.

use std::sync::Arc;

use crate::ArcConditionalBiMutatingFunction;
use crate::BiMutatingFunction;
use crate::BiPredicate;
use crate::MutatingFunction;
use crate::functions::macros::impl_function_clone;
use crate::functions::macros::impl_function_common_methods;
use crate::functions::macros::impl_function_constant_method;
use crate::functions::macros::impl_function_debug_display;
use crate::functions::macros::impl_shared_function_methods;

/// The erased callback representation used by this implementation.
type ArcBiMutatingFunctionFn<T, U, R> = Arc<dyn Fn(&mut T, &mut U) -> R + Send + Sync>;

// ============================================================================
// ArcBiMutatingFunction - Arc<dyn Fn(&mut T, &mut U) -> R + Send + Sync>
// ============================================================================

/// ArcBiMutatingFunction - thread-safe bi-mutating-function wrapper
///
/// A thread-safe, clonable bi-mutating-function wrapper suitable for
/// multi-threaded scenarios. Can be called multiple times and shared across
/// threads.
///
/// # Features
///
/// - **Based on**: `Arc<dyn Fn(&mut T, &mut U) -> R + Send + Sync>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (borrows inputs mutably each
///   time)
/// - **Thread Safety**: Thread-safe (`Send + Sync` required)
/// - **Clonable**: Cheap cloning via `Arc::clone`
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcBiMutatingFunction<T, U, R> {
    /// The wrapped callback implementation.
    pub(super) function: ArcBiMutatingFunctionFn<T, U, R>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, U, R> ArcBiMutatingFunction<T, U, R> {
    // Generates: new(), new_with_name(), new_with_optional_name(), name(),
    // set_name()
    impl_function_common_methods!(
        ArcBiMutatingFunction<T, U, R>,
        (Fn(&mut T, &mut U) -> R + Send + Sync + 'static),
        |f| Arc::new(f)
    );

    // Generate shared-wrapper composition methods.
    impl_shared_function_methods!(
        ArcBiMutatingFunction<T, U, R>,
        ArcConditionalBiMutatingFunction,
        ArcBiPredicate,
        MutatingFunction,
        predicate_bounds = (Send + Sync + 'static),
        chained_bounds = (Send + Sync + 'static)
    );
}

// Implement BiMutatingFunction trait for ArcBiMutatingFunction
impl<T, U, R> BiMutatingFunction<T, U, R> for ArcBiMutatingFunction<T, U, R> {
    #[inline(always)]
    fn apply(&self, first: &mut T, second: &mut U) -> R {
        (self.function)(first, second)
    }
}

// Implement constant method for ArcBiMutatingFunction
impl_function_constant_method!(ArcBiMutatingFunction<T, U, R>, mut Send + Sync + 'static);

// Implement Debug and Display for ArcBiMutatingFunction
impl_function_debug_display!(ArcBiMutatingFunction<T, U, R>);

// Implement Clone for ArcBiMutatingFunction
impl_function_clone!(ArcBiMutatingFunction<T, U, R>);
