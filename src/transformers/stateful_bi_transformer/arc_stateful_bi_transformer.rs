// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcStatefulBiTransformer` public type.

use std::sync::Arc;

use parking_lot::Mutex;

use crate::ArcConditionalStatefulBiTransformer;
use crate::BiPredicate;
use crate::StatefulBiTransformer;
use crate::StatefulTransformer;
use crate::transformers::macros::impl_shared_transformer_methods;
use crate::transformers::macros::impl_transformer_clone;
use crate::transformers::macros::impl_transformer_common_methods;
use crate::transformers::macros::impl_transformer_constant_method;
use crate::transformers::macros::impl_transformer_debug_display;

// ============================================================================
// ArcStatefulBiTransformer - Arc<Mutex<dyn FnMut(T, U) -> R + Send>>
// ============================================================================

/// ArcStatefulBiTransformer - thread-safe bi-transformer wrapper
///
/// A thread-safe, clonable bi-transformer wrapper suitable for multi-threaded
/// scenarios. Can be called multiple times and shared across threads.
///
/// # Features
///
/// - **Based on**: `Arc<Mutex<dyn FnMut(T, U) -> R + Send>>`
/// - **Ownership**: Shared ownership via reference counting
/// - **Reusability**: Can be called multiple times (each call consumes its
///   inputs)
/// - **Thread Safety**: The callback itself requires only `Send`; the
///   `Arc<Mutex<_>>` wrapper is both `Send` and `Sync`, and serializes calls
/// - **Clonable**: Cheap cloning via `Arc::clone`
///
/// # Locking and reentrancy
///
/// Each call acquires a `parking_lot::Mutex` and holds it while the user
/// callback runs. Synchronous re-entry through the same shared state
/// deadlocks. The mutex is not poisoned after a panic, and mutations completed
/// before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcStatefulBiTransformer<T, U, R> {
    /// The wrapped callback implementation.
    pub(super) function: Arc<Mutex<dyn FnMut(T, U) -> R + Send>>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, U, R> ArcStatefulBiTransformer<T, U, R> {
    impl_transformer_common_methods!(
        ArcStatefulBiTransformer<T, U, R>,
        (FnMut(T, U) -> R + Send + 'static),
        |f| Arc::new(Mutex::new(f))
    );

    impl_shared_transformer_methods!(
        ArcStatefulBiTransformer<T, U, R>,
        ArcConditionalStatefulBiTransformer,
        ArcBiPredicate,
        StatefulTransformer,
        predicate_bounds = (Send + Sync + 'static),
        chained_bounds = (Send + 'static)
    );
}

// Implement constant method for ArcStatefulBiTransformer
impl_transformer_constant_method!(stateful thread_safe ArcStatefulBiTransformer<T, U, R>);

// Implement Debug and Display for ArcStatefulBiTransformer
impl_transformer_debug_display!(ArcStatefulBiTransformer<T, U, R>);

// Implement Clone for ArcStatefulBiTransformer
impl_transformer_clone!(ArcStatefulBiTransformer<T, U, R>);

// Implement StatefulBiTransformer trait for ArcStatefulBiTransformer
impl<T, U, R> StatefulBiTransformer<T, U, R> for ArcStatefulBiTransformer<T, U, R> {
    #[inline]
    fn apply(&mut self, first: T, second: U) -> R {
        let mut func = self.function.lock();
        func(first, second)
    }
}
