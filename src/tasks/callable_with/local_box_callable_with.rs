// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `LocalBoxCallableWith` public type.

use crate::functions::macros::impl_function_debug_display;
use crate::macros::impl_common_name_methods;
use crate::macros::impl_common_new_methods;
use crate::tasks::callable_with::CallableWith;

/// The erased callback representation used by this implementation.
type LocalBoxCallableWithFn<T, R, E> = Box<dyn FnMut(&mut T) -> Result<R, E>>;

/// Local box-based callable with mutable input.
///
/// `LocalBoxCallableWith<T, R, E>` can be called repeatedly on the local
/// thread and permits non-`Send` captures. Use
/// [`BoxCallableWith`](crate::tasks::callable_with::BoxCallableWith) when the
/// callable must be movable across threads.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct LocalBoxCallableWith<T, R, E> {
    /// The stateful closure executed by this callable.
    pub(super) function: LocalBoxCallableWithFn<T, R, E>,
    /// The optional name of this callable.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, R, E> LocalBoxCallableWith<T, R, E> {
    impl_common_new_methods!(
        semantic_mut(CallableWith<T, R, E> + 'static),
        |source| move |input: &mut T| source.call_with(input),
        |function| Box::new(function),
        "local callable-with"
    );

    impl_common_name_methods!("local callable-with");

    /// Maps the success value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the success value.
    ///
    /// # Returns
    ///
    /// A new local callable with mutable input that maps successful results.
    #[inline]
    pub fn map<U, M>(self, mut mapper: M) -> LocalBoxCallableWith<T, U, E>
    where
        M: FnMut(R) -> U + 'static,
        T: 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        LocalBoxCallableWith::new_with_metadata(move |input: &mut T| function(input).map(&mut mapper), metadata)
    }

    /// Maps the error value of this callable.
    ///
    /// # Parameters
    ///
    /// * `mapper` - Function that transforms the error value.
    ///
    /// # Returns
    ///
    /// A new local callable with mutable input that maps failed results.
    #[inline]
    pub fn map_err<E2, M>(self, mut mapper: M) -> LocalBoxCallableWith<T, R, E2>
    where
        M: FnMut(E) -> E2 + 'static,
        T: 'static,
        R: 'static,
        E: 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        LocalBoxCallableWith::new_with_metadata(move |input: &mut T| function(input).map_err(&mut mapper), metadata)
    }

    /// Chains another computation after this callable succeeds.
    ///
    /// # Parameters
    ///
    /// * `next` - Function receiving the success value and mutable input.
    ///
    /// # Returns
    ///
    /// A new local callable that runs `next` only after success.
    #[inline]
    pub fn and_then<U, N>(self, mut next: N) -> LocalBoxCallableWith<T, U, E>
    where
        N: FnMut(R, &mut T) -> Result<U, E> + 'static,
        T: 'static,
        R: 'static,
        E: 'static,
    {
        let mut function = self.function;
        LocalBoxCallableWith::new(move |input: &mut T| {
            let value = function(&mut *input)?;
            next(value, input)
        })
    }
}

impl<T, R, E> CallableWith<T, R, E> for LocalBoxCallableWith<T, R, E> {
    /// Executes the local boxed callable with mutable input.
    #[inline(always)]
    fn call_with(&mut self, input: &mut T) -> Result<R, E> {
        (self.function)(input)
    }
}

impl_function_debug_display!(LocalBoxCallableWith<T, R, E>);
