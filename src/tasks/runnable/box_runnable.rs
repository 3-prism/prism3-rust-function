// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxRunnable` public type.

use crate::macros::impl_common_name_methods;
use crate::macros::impl_common_new_methods;
use crate::suppliers::macros::impl_supplier_debug_display;
use crate::suppliers::supplier::Supplier;
use crate::tasks::callable::BoxCallable;
use crate::tasks::runnable::Runnable;

// ============================================================================
// BoxRunnable
// ============================================================================

/// Box-based reusable runnable.
///
/// `BoxRunnable<E>` stores a `Box<dyn FnMut() -> Result<(), E> + Send>` and
/// can be executed repeatedly. It is the boxed concrete implementation of
/// [`Runnable`] for tasks that may be moved across threads.
///
/// # Type Parameters
///
/// * `E` - The error value returned when the action fails.
///
/// # Examples
///
/// ```rust
/// use qubit_function::{BoxRunnable, Runnable};
///
/// let mut task = BoxRunnable::new(|| Ok::<(), String>(()));
/// assert_eq!(task.run(), Ok(()));
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxRunnable<E> {
    /// The stateful closure executed by this runnable.
    pub(super) function: Box<dyn FnMut() -> Result<(), E> + Send>,
    /// The optional name of this runnable.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<E> BoxRunnable<E> {
    impl_common_new_methods!(
        semantic_mut(Runnable<E> + Send + 'static),
        |source| move || source.run(),
        |function| Box::new(function),
        "runnable"
    );

    /// Creates a boxed runnable from a reusable supplier.
    ///
    /// This is an explicit bridge from `Supplier<Result<(), E>>` to
    /// `Runnable<E>`.
    ///
    /// # Parameters
    ///
    /// * `supplier` - The supplier that produces the runnable result.
    ///
    /// # Returns
    ///
    /// A new `BoxRunnable<E>`.
    #[inline]
    pub fn from_supplier<S>(supplier: S) -> Self
    where
        S: Supplier<Result<(), E>> + Send + 'static,
    {
        Self::new(move || supplier.get())
    }

    impl_common_name_methods!("runnable");

    /// Chains another runnable after this runnable succeeds.
    ///
    /// The second runnable is not executed if this runnable returns `Err`.
    ///
    /// # Parameters
    ///
    /// * `next` - The runnable to execute after this runnable succeeds.
    ///
    /// # Returns
    ///
    /// A runnable executing both actions in sequence.
    #[inline]
    pub fn and_then<N>(self, next: N) -> BoxRunnable<E>
    where
        N: Runnable<E> + Send + 'static,
        E: 'static,
    {
        let mut function = self.function;
        let mut next = next;
        BoxRunnable::new(move || {
            function()?;
            next.run()
        })
    }

    /// Runs this runnable before a callable.
    ///
    /// The callable is not executed if this runnable returns `Err`.
    /// Because this operation sequences two independent callbacks, the
    /// returned callable is unnamed.
    ///
    /// # Parameters
    ///
    /// * `callable` - The callable to execute after this runnable succeeds.
    ///
    /// # Returns
    ///
    /// A callable producing the second computation's result.
    #[inline]
    pub fn then_callable<R, C>(self, callable: C) -> BoxCallable<R, E>
    where
        C: crate::tasks::callable::Callable<R, E> + Send + 'static,
        R: 'static,
        E: 'static,
    {
        let mut function = self.function;
        let mut callable = callable;
        BoxCallable::new(move || {
            function()?;
            callable.call()
        })
    }
}

impl<E> Runnable<E> for BoxRunnable<E> {
    /// Executes the boxed runnable.
    #[inline(always)]
    fn run(&mut self) -> Result<(), E> {
        (self.function)()
    }
}

#[cfg(feature = "once")]
impl<E> crate::suppliers::supplier_once::SupplierOnce<Result<(), E>> for BoxRunnable<E> {
    /// Executes the boxed runnable as a one-time supplier of `Result<(), E>`.
    #[inline(always)]
    fn get(mut self) -> Result<(), E> {
        self.run()
    }
}

impl_supplier_debug_display!(BoxRunnable<E>);
