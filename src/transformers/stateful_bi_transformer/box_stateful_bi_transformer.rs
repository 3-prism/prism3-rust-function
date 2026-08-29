// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxStatefulBiTransformer` public type.

use crate::BiPredicate;
use crate::BoxConditionalStatefulBiTransformer;
use crate::StatefulBiTransformer;
use crate::StatefulTransformer;
use crate::transformers::macros::impl_box_transformer_methods;
use crate::transformers::macros::impl_transformer_common_methods;
use crate::transformers::macros::impl_transformer_constant_method;
use crate::transformers::macros::impl_transformer_debug_display;

// ============================================================================
// BoxStatefulBiTransformer - Box<dyn FnMut(T, U) -> R>
// ============================================================================

/// BoxStatefulBiTransformer - bi-transformer wrapper based on `Box<dyn Fn>`
///
/// A bi-transformer wrapper that provides single ownership with reusable
/// transformation. The bi-transformer consumes both inputs and can be called
/// multiple times.
///
/// # Features
///
/// - **Based on**: `Box<dyn FnMut(T, U) -> R>`
/// - **Ownership**: Single ownership, cannot be cloned
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: Not thread-safe (no `Send + Sync` requirement)
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxStatefulBiTransformer<T, U, R> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn FnMut(T, U) -> R>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, U, R> BoxStatefulBiTransformer<T, U, R> {
    impl_transformer_common_methods!(
        BoxStatefulBiTransformer<T, U, R>,
        (FnMut(T, U) -> R + 'static),
        |f| Box::new(f)
    );

    impl_box_transformer_methods!(
        BoxStatefulBiTransformer<T, U, R>,
        BoxConditionalStatefulBiTransformer,
        StatefulTransformer
    );
}

// Implement constant method for BoxStatefulBiTransformer
impl_transformer_constant_method!(stateful BoxStatefulBiTransformer<T, U, R>);

// Implement Debug and Display for BoxTransformer
impl_transformer_debug_display!(BoxStatefulBiTransformer<T, U, R>);

impl<T, U, R> StatefulBiTransformer<T, U, R> for BoxStatefulBiTransformer<T, U, R> {
    #[inline(always)]
    fn apply(&mut self, first: T, second: U) -> R {
        (self.function)(first, second)
    }
}
