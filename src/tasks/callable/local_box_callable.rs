// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `LocalBoxCallable` public type.

use crate::functions::macros::impl_function_debug_display;
use crate::macros::impl_common_name_methods;
use crate::macros::impl_common_new_methods;
use crate::suppliers::supplier::Supplier;
use crate::tasks::callable::Callable;

/// Local box-based callable.
///
/// `LocalBoxCallable<R, E>` stores a `Box<dyn FnMut() -> Result<R, E>>` and
/// can be called repeatedly on the local thread. Use
/// [`BoxCallable`](crate::tasks::callable::BoxCallable) when the callable must
/// be movable across threads.
///
/// # Type Parameters
///
/// * `R` - The success value returned by the computation.
/// * `E` - The error value returned when the computation fails.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct LocalBoxCallable<R, E> {
    /// The stateful closure executed by this callable.
    pub(super) function: Box<dyn FnMut() -> Result<R, E>>,
    /// The optional name of this callable.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<R, E> LocalBoxCallable<R, E> {
    impl_common_new_methods!(
        semantic_mut(Callable<R, E> + 'static),
        |source| move || source.call(),
        |function| Box::new(function),
        "local callable"
    );

    /// Creates a local boxed callable from a reusable supplier.
    ///
    /// This is an explicit bridge from `Supplier<Result<R, E>>` to
    /// `Callable<R, E>` without requiring `Send`.
    ///
    /// # Parameters
    ///
    /// * `supplier` - The supplier that produces the callable result.
    ///
    /// # Returns
    ///
    /// A new `LocalBoxCallable<R, E>`.
    #[inline]
    pub fn from_supplier<S>(supplier: S) -> Self
    where
        S: Supplier<Result<R, E>> + 'static,
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
    /// A new local callable that applies `mapper` on success.
    #[inline]
    pub fn map<U, M>(self, mut mapper: M) -> LocalBoxCallable<U, E>
    where
        M: FnMut(R) -> U + 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        LocalBoxCallable::new_with_metadata(move || function().map(&mut mapper), metadata)
    }

    /// Maps the error value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the error value.
    ///
    /// # Returns
    ///
    /// A new local callable that applies `mapper` on failure.
    #[inline]
    pub fn map_err<E2, M>(self, mut mapper: M) -> LocalBoxCallable<R, E2>
    where
        M: FnMut(E) -> E2 + 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        LocalBoxCallable::new_with_metadata(move || function().map_err(&mut mapper), metadata)
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
    /// A new local callable that runs `next` only after success.
    #[inline]
    pub fn and_then<U, N>(self, mut next: N) -> LocalBoxCallable<U, E>
    where
        N: FnMut(R) -> Result<U, E> + 'static,
        R: 'static,
        E: 'static,
    {
        let mut function = self.function;
        LocalBoxCallable::new(move || {
            let value = function()?;
            next(value)
        })
    }
}

impl<R, E> Callable<R, E> for LocalBoxCallable<R, E> {
    /// Executes the local boxed callable.
    #[inline(always)]
    fn call(&mut self) -> Result<R, E> {
        (self.function)()
    }
}

impl_function_debug_display!(LocalBoxCallable<R, E>);
