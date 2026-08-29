// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `LocalBoxCallableOnce` public type.

use crate::functions::macros::impl_function_debug_display;
use crate::macros::impl_common_name_methods;
use crate::macros::impl_common_new_methods;
use crate::suppliers::supplier_once::SupplierOnce;
use crate::tasks::callable_once::CallableOnce;

// ============================================================================
// LocalBoxCallableOnce
// ============================================================================

/// Local box-based one-time callable.
///
/// `LocalBoxCallableOnce<R, E>` stores a `Box<dyn FnOnce() -> Result<R, E>>`
/// and can be executed only once on the local thread. Use
/// [`BoxCallableOnce`](crate::tasks::callable_once::BoxCallableOnce) when the
/// callable must be movable across threads.
///
/// # Type Parameters
///
/// * `R` - The success value returned by the computation.
/// * `E` - The error value returned when the computation fails.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct LocalBoxCallableOnce<R, E> {
    /// The one-time closure executed by this callable.
    pub(super) function: Box<dyn FnOnce() -> Result<R, E>>,
    /// The optional name of this callable.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<R, E> LocalBoxCallableOnce<R, E> {
    impl_common_new_methods!(
        semantic(CallableOnce<R, E> + 'static),
        |source| move || source.call_once(),
        |function| Box::new(function),
        "local callable"
    );

    /// Creates a local boxed callable from a one-time supplier.
    ///
    /// This is an explicit bridge from `SupplierOnce<Result<R, E>>` to
    /// `CallableOnce<R, E>` without requiring `Send`.
    ///
    /// # Parameters
    ///
    /// * `supplier` - The supplier that produces the callable result.
    ///
    /// # Returns
    ///
    /// A new `LocalBoxCallableOnce<R, E>`.
    #[inline]
    pub fn from_supplier<S>(supplier: S) -> Self
    where
        S: SupplierOnce<Result<R, E>> + 'static,
    {
        Self::new(move || supplier.get())
    }

    impl_common_name_methods!("local callable");

    /// Maps the success value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the success value.
    ///
    /// # Returns
    ///
    /// A new local callable that applies `mapper` when this callable succeeds.
    #[inline]
    pub fn map<U, M>(self, mapper: M) -> LocalBoxCallableOnce<U, E>
    where
        M: FnOnce(R) -> U + 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let function = self.function;
        LocalBoxCallableOnce::new_with_metadata(move || function().map(mapper), metadata)
    }

    /// Maps the error value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the error value.
    ///
    /// # Returns
    ///
    /// A new local callable that applies `mapper` when this callable fails.
    #[inline]
    pub fn map_err<E2, M>(self, mapper: M) -> LocalBoxCallableOnce<R, E2>
    where
        M: FnOnce(E) -> E2 + 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let function = self.function;
        LocalBoxCallableOnce::new_with_metadata(move || function().map_err(mapper), metadata)
    }

    /// Chains another fallible computation after this callable succeeds.
    ///
    /// # Parameters
    ///
    /// * `next` - Function that receives the success value and returns the next
    ///   result.
    ///
    /// # Returns
    ///
    /// A new local callable that runs `next` only when this callable succeeds.
    #[inline]
    pub fn and_then<U, N>(self, next: N) -> LocalBoxCallableOnce<U, E>
    where
        N: FnOnce(R) -> Result<U, E> + 'static,
        R: 'static,
        E: 'static,
    {
        let function = self.function;
        LocalBoxCallableOnce::new(move || function().and_then(next))
    }
}

impl<R, E> CallableOnce<R, E> for LocalBoxCallableOnce<R, E> {
    /// Executes the local boxed callable.
    #[inline(always)]
    fn call_once(self) -> Result<R, E> {
        (self.function)()
    }
}

impl<R, E> SupplierOnce<Result<R, E>> for LocalBoxCallableOnce<R, E> {
    /// Executes the local boxed callable as a one-time supplier of
    /// `Result<R, E>`.
    #[inline(always)]
    fn get(self) -> Result<R, E> {
        self.call_once()
    }
}

impl_function_debug_display!(LocalBoxCallableOnce<R, E>);
