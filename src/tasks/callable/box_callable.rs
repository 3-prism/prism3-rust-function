// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxCallable` public type.

use crate::functions::macros::impl_function_debug_display;
use crate::macros::impl_common_name_methods;
use crate::macros::impl_common_new_methods;
use crate::suppliers::supplier::Supplier;
use crate::tasks::callable::Callable;

// ============================================================================
// BoxCallable
// ============================================================================

/// Box-based callable.
///
/// `BoxCallable<R, E>` stores a
/// `Box<dyn FnMut() -> Result<R, E> + Send>` and can be called repeatedly. It
/// is the boxed concrete implementation of [`Callable`] for tasks that may be
/// moved across threads.
///
/// # Type Parameters
///
/// * `R` - The success value returned by the computation.
/// * `E` - The error value returned when the computation fails.
///
/// # Examples
///
/// ```rust
/// use qubit_function::{BoxCallable, Callable};
///
/// let mut task = BoxCallable::new(|| Ok::<i32, String>(42));
/// assert_eq!(task.call().expect("call should succeed"), 42);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxCallable<R, E> {
    /// The stateful closure executed by this callable.
    pub(super) function: Box<dyn FnMut() -> Result<R, E> + Send>,
    /// The optional name of this callable.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<R, E> BoxCallable<R, E> {
    impl_common_new_methods!(
        semantic_mut(Callable<R, E> + Send + 'static),
        |source| move || source.call(),
        |function| Box::new(function),
        "callable"
    );

    /// Creates a boxed callable from a reusable supplier.
    ///
    /// This is an explicit bridge from `Supplier<Result<R, E>>` to
    /// `Callable<R, E>`.
    ///
    /// # Parameters
    ///
    /// * `supplier` - The supplier that produces the callable result.
    ///
    /// # Returns
    ///
    /// A new `BoxCallable<R, E>`.
    #[inline]
    pub fn from_supplier<S>(supplier: S) -> Self
    where
        S: Supplier<Result<R, E>> + Send + 'static,
    {
        Self::new(move || supplier.get())
    }

    impl_common_name_methods!("callable");

    /// Maps the success value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the success value.
    ///
    /// # Returns
    ///
    /// A new callable that applies `mapper` when this callable succeeds.
    #[inline]
    pub fn map<U, M>(self, mut mapper: M) -> BoxCallable<U, E>
    where
        M: FnMut(R) -> U + Send + 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        BoxCallable::new_with_metadata(move || function().map(&mut mapper), metadata)
    }

    /// Maps the error value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the error value.
    ///
    /// # Returns
    ///
    /// A new callable that applies `mapper` when this callable fails.
    #[inline]
    pub fn map_err<E2, M>(self, mut mapper: M) -> BoxCallable<R, E2>
    where
        M: FnMut(E) -> E2 + Send + 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        BoxCallable::new_with_metadata(move || function().map_err(&mut mapper), metadata)
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
    /// A new callable that runs `next` only when this callable succeeds.
    #[inline]
    pub fn and_then<U, N>(self, next: N) -> BoxCallable<U, E>
    where
        N: FnMut(R) -> Result<U, E> + Send + 'static,
        R: 'static,
        E: 'static,
    {
        let mut function = self.function;
        let mut next = next;
        BoxCallable::new(move || {
            let value = function()?;
            next(value)
        })
    }
}

impl<R, E> Callable<R, E> for BoxCallable<R, E> {
    /// Executes the boxed callable.
    #[inline(always)]
    fn call(&mut self) -> Result<R, E> {
        (self.function)()
    }
}

impl_function_debug_display!(BoxCallable<R, E>);
