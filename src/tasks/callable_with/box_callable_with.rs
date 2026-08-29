// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxCallableWith` public type.

use crate::functions::macros::impl_function_debug_display;
use crate::macros::impl_common_name_methods;
use crate::macros::impl_common_new_methods;
use crate::tasks::callable_with::CallableWith;

/// The erased callback representation used by this implementation.
type BoxCallableWithFn<T, R, E> = Box<dyn FnMut(&mut T) -> Result<R, E> + Send>;

/// Box-based callable with mutable input.
///
/// `BoxCallableWith<T, R, E>` stores a
/// `Box<dyn FnMut(&mut T) -> Result<R, E> + Send>` and can be called
/// repeatedly on tasks that may be moved across threads.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxCallableWith<T, R, E> {
    /// The stateful closure executed by this callable.
    pub(super) function: BoxCallableWithFn<T, R, E>,
    /// The optional name of this callable.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, R, E> BoxCallableWith<T, R, E> {
    impl_common_new_methods!(
        semantic_mut(CallableWith<T, R, E> + Send + 'static),
        |source| move |input: &mut T| source.call_with(input),
        |function| Box::new(function),
        "callable-with"
    );

    impl_common_name_methods!("callable-with");

    /// Maps the success value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the success value.
    ///
    /// # Returns
    ///
    /// A new callable with mutable input that applies `mapper` on success.
    #[inline]
    pub fn map<U, M>(self, mut mapper: M) -> BoxCallableWith<T, U, E>
    where
        M: FnMut(R) -> U + Send + 'static,
        T: 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        BoxCallableWith::new_with_metadata(move |input: &mut T| function(input).map(&mut mapper), metadata)
    }

    /// Maps the error value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the error value.
    ///
    /// # Returns
    ///
    /// A new callable with mutable input that applies `mapper` on failure.
    #[inline]
    pub fn map_err<E2, M>(self, mut mapper: M) -> BoxCallableWith<T, R, E2>
    where
        M: FnMut(E) -> E2 + Send + 'static,
        T: 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        BoxCallableWith::new_with_metadata(move |input: &mut T| function(input).map_err(&mut mapper), metadata)
    }

    /// Chains another fallible computation after this callable succeeds.
    ///
    /// # Parameters
    ///
    /// * `next` - Function receiving the success value and mutable input.
    ///
    /// # Returns
    ///
    /// A new callable that runs `next` only when this callable succeeds.
    #[inline]
    pub fn and_then<U, N>(self, next: N) -> BoxCallableWith<T, U, E>
    where
        N: FnMut(R, &mut T) -> Result<U, E> + Send + 'static,
        T: 'static,
        R: 'static,
        E: 'static,
    {
        let mut function = self.function;
        let mut next = next;
        BoxCallableWith::new(move |input: &mut T| {
            let value = function(&mut *input)?;
            next(value, input)
        })
    }
}

impl<T, R, E> CallableWith<T, R, E> for BoxCallableWith<T, R, E> {
    /// Executes the boxed callable with mutable input.
    #[inline(always)]
    fn call_with(&mut self, input: &mut T) -> Result<R, E> {
        (self.function)(input)
    }
}

impl_function_debug_display!(BoxCallableWith<T, R, E>);
