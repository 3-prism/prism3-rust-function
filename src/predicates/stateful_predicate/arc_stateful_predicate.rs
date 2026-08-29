// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcStatefulPredicate` public type.

use std::ops::Not;
use std::sync::Arc;

use parking_lot::Mutex;

use super::ALWAYS_FALSE_NAME;
use super::ALWAYS_TRUE_NAME;
use crate::StatefulPredicate;
use crate::predicates::macros::impl_predicate_clone;
use crate::predicates::macros::impl_predicate_common_methods;
use crate::predicates::macros::impl_predicate_debug_display;

/// The erased callback representation used by this implementation.
type ArcStatefulPredicateFn<T> = Arc<Mutex<dyn FnMut(&T) -> bool + Send + 'static>>;

/// An Arc-based stateful predicate with thread-safe shared ownership.
///
/// This type stores the predicate closure inside `Arc<Mutex<_>>`, allowing
/// cheap clones that share mutable predicate state across threads.
///
/// # Locking and reentrancy
///
/// Each call acquires a `parking_lot::Mutex` and holds it while the user
/// callback runs. Synchronous re-entry through the same shared state
/// deadlocks. The mutex is not poisoned after a panic, and mutations completed
/// before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcStatefulPredicate<T> {
    /// The wrapped callback implementation.
    pub(super) function: ArcStatefulPredicateFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> ArcStatefulPredicate<T> {
    // Generates: new(), new_with_name(), name(), set_name(), always_true(),
    // always_false()
    impl_predicate_common_methods!(ArcStatefulPredicate<T>, (FnMut(&T) -> bool + Send + 'static), |f| {
        Arc::new(Mutex::new(f))
    });

    /// Returns a predicate representing logical AND with another predicate.
    ///
    /// This method borrows `self`; the returned predicate shares this
    /// predicate's mutable state through the same `Arc<Mutex<_>>`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with.
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulPredicate` representing logical AND.
    #[inline]
    pub fn and<P>(&self, mut other: P) -> ArcStatefulPredicate<T>
    where
        P: StatefulPredicate<T> + Send + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        ArcStatefulPredicate::new(move |value: &T| {
            let matched = {
                let mut function = self_fn.lock();
                function(value)
            };
            matched && other.test(value)
        })
    }

    /// Returns a predicate representing logical OR with another predicate.
    ///
    /// This method borrows `self`; the returned predicate shares this
    /// predicate's mutable state through the same `Arc<Mutex<_>>`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with.
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulPredicate` representing logical OR.
    #[inline]
    pub fn or<P>(&self, mut other: P) -> ArcStatefulPredicate<T>
    where
        P: StatefulPredicate<T> + Send + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        ArcStatefulPredicate::new(move |value: &T| {
            let matched = {
                let mut function = self_fn.lock();
                function(value)
            };
            matched || other.test(value)
        })
    }

    /// Returns a predicate representing logical NAND with another predicate.
    ///
    /// NAND returns `true` unless both predicates return `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with.
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulPredicate` representing logical NAND.
    #[inline]
    pub fn nand<P>(&self, mut other: P) -> ArcStatefulPredicate<T>
    where
        P: StatefulPredicate<T> + Send + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        ArcStatefulPredicate::new(move |value: &T| {
            let matched = {
                let mut function = self_fn.lock();
                function(value)
            };
            !(matched && other.test(value))
        })
    }

    /// Returns a predicate representing logical XOR with another predicate.
    ///
    /// XOR evaluates both predicates and returns `true` when exactly one
    /// predicate returns `true`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with.
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulPredicate` representing logical XOR.
    #[inline]
    pub fn xor<P>(&self, mut other: P) -> ArcStatefulPredicate<T>
    where
        P: StatefulPredicate<T> + Send + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        ArcStatefulPredicate::new(move |value: &T| {
            let matched = {
                let mut function = self_fn.lock();
                function(value)
            };
            matched ^ other.test(value)
        })
    }

    /// Returns a predicate representing logical NOR with another predicate.
    ///
    /// NOR returns `true` only when both predicates return `false`.
    ///
    /// # Parameters
    ///
    /// * `other` - The other predicate to combine with.
    ///
    /// # Returns
    ///
    /// A new `ArcStatefulPredicate` representing logical NOR.
    #[inline]
    pub fn nor<P>(&self, mut other: P) -> ArcStatefulPredicate<T>
    where
        P: StatefulPredicate<T> + Send + 'static,
        T: 'static,
    {
        let self_fn = self.function.clone();
        ArcStatefulPredicate::new(move |value: &T| {
            let matched = {
                let mut function = self_fn.lock();
                function(value)
            };
            !(matched || other.test(value))
        })
    }
}

impl<T> Not for ArcStatefulPredicate<T>
where
    T: 'static,
{
    type Output = ArcStatefulPredicate<T>;

    fn not(self) -> Self::Output {
        let metadata = self.metadata;
        let function = self.function;
        ArcStatefulPredicate::new_with_metadata(
            move |value: &T| {
                let mut function = function.lock();
                !function(value)
            },
            metadata,
        )
    }
}

impl<T> Not for &ArcStatefulPredicate<T>
where
    T: 'static,
{
    type Output = ArcStatefulPredicate<T>;

    fn not(self) -> Self::Output {
        let function = self.function.clone();
        ArcStatefulPredicate::new_with_metadata(
            move |value: &T| {
                let mut function = function.lock();
                !function(value)
            },
            self.metadata.clone(),
        )
    }
}

// Generates: impl Clone for ArcStatefulPredicate<T>
impl_predicate_clone!(ArcStatefulPredicate<T>);

// Generates: impl Debug for ArcStatefulPredicate<T> and impl Display for
// ArcStatefulPredicate<T>
impl_predicate_debug_display!(ArcStatefulPredicate<T>);

// Implements StatefulPredicate trait for ArcStatefulPredicate<T>
impl<T> StatefulPredicate<T> for ArcStatefulPredicate<T> {
    #[inline]
    fn test(&mut self, value: &T) -> bool {
        let mut function = self.function.lock();
        function(value)
    }
}
