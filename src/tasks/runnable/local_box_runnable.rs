// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `LocalBoxRunnable` public type.

use crate::macros::impl_common_name_methods;
use crate::macros::impl_common_new_methods;
use crate::suppliers::macros::impl_supplier_debug_display;
use crate::suppliers::supplier::Supplier;
use crate::tasks::callable::Callable;
use crate::tasks::callable::LocalBoxCallable;
use crate::tasks::runnable::Runnable;

/// Local box-based reusable runnable.
///
/// `LocalBoxRunnable<E>` stores a `Box<dyn FnMut() -> Result<(), E>>` and can
/// be executed repeatedly on the local thread. Use
/// [`BoxRunnable`](crate::tasks::runnable::BoxRunnable) when the runnable must
/// be movable across threads.
///
/// # Type Parameters
///
/// * `E` - The error value returned when the action fails.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct LocalBoxRunnable<E> {
    /// The stateful closure executed by this runnable.
    pub(super) function: Box<dyn FnMut() -> Result<(), E>>,
    /// The optional name of this runnable.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<E> LocalBoxRunnable<E> {
    impl_common_new_methods!(
        semantic_mut(Runnable<E> + 'static),
        |source| move || source.run(),
        |function| Box::new(function),
        "local runnable"
    );

    /// Creates a local boxed runnable from a reusable supplier.
    ///
    /// # Parameters
    ///
    /// * `supplier` - The supplier that produces the runnable result.
    ///
    /// # Returns
    ///
    /// A new `LocalBoxRunnable<E>`.
    #[inline]
    pub fn from_supplier<S>(supplier: S) -> Self
    where
        S: Supplier<Result<(), E>> + 'static,
    {
        Self::new(move || supplier.get())
    }

    impl_common_name_methods!("local runnable");

    /// Chains another runnable after this runnable succeeds.
    ///
    /// # Parameters
    ///
    /// * `next` - The runnable to execute after this runnable succeeds.
    ///
    /// # Returns
    ///
    /// A local runnable executing both actions in sequence.
    #[inline]
    pub fn and_then<N>(self, mut next: N) -> LocalBoxRunnable<E>
    where
        N: Runnable<E> + 'static,
        E: 'static,
    {
        let mut function = self.function;
        LocalBoxRunnable::new(move || {
            function()?;
            next.run()
        })
    }

    /// Runs this runnable before a local callable.
    ///
    /// # Parameters
    ///
    /// * `callable` - The callable to execute after this runnable succeeds.
    ///
    /// # Returns
    ///
    /// A local callable producing the second computation's result.
    #[inline]
    pub fn then_callable<R, C>(self, mut callable: C) -> LocalBoxCallable<R, E>
    where
        C: Callable<R, E> + 'static,
        R: 'static,
        E: 'static,
    {
        let mut function = self.function;
        LocalBoxCallable::new(move || {
            function()?;
            callable.call()
        })
    }
}

impl<E> Runnable<E> for LocalBoxRunnable<E> {
    /// Executes the local boxed runnable.
    #[inline(always)]
    fn run(&mut self) -> Result<(), E> {
        (self.function)()
    }
}

#[cfg(feature = "once")]
impl<E> crate::suppliers::supplier_once::SupplierOnce<Result<(), E>> for LocalBoxRunnable<E> {
    /// Executes the local boxed runnable as a one-time supplier.
    #[inline(always)]
    fn get(mut self) -> Result<(), E> {
        self.run()
    }
}

impl_supplier_debug_display!(LocalBoxRunnable<E>);
