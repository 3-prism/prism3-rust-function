// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcPredicate` public type.

use std::ops::Not;
use std::sync::Arc;

use super::ALWAYS_FALSE_NAME;
use super::ALWAYS_TRUE_NAME;
use crate::Predicate;
use crate::predicates::macros::impl_predicate_clone;
use crate::predicates::macros::impl_predicate_common_methods;
use crate::predicates::macros::impl_predicate_debug_display;
use crate::predicates::macros::impl_shared_predicate_methods;

/// An Arc-based predicate with thread-safe shared ownership.
///
/// This type is suitable for scenarios where the predicate needs to be
/// shared across threads. Composition methods borrow `&self`, allowing the
/// original predicate to remain usable after composition.
///
///
/// # Examples
///
/// ```rust
/// # {
/// use qubit_function::{Predicate, ArcPredicate};
///
/// let pred = ArcPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
///
/// // Original predicate remains usable after composition
/// let combined = pred.and(ArcPredicate::new(|x: &i32| x % 2 == 0));
/// assert!(pred.test(&5));  // Still works
///
/// // Can be cloned and sent across threads
/// let pred_clone = pred.clone();
/// std::thread::spawn(move || {
///     assert!(pred_clone.test(&10));
/// }).join().expect("thread should not panic");
/// # }
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcPredicate<T> {
    /// The wrapped callback implementation.
    pub(super) function: Arc<dyn Fn(&T) -> bool + Send + Sync>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> ArcPredicate<T> {
    // Generates: new(), new_with_name(), name(), set_name(), always_true(),
    // always_false()
    impl_predicate_common_methods!(
        ArcPredicate<T>,
        semantic(Predicate<T> + Send + Sync + 'static),
        |predicate| move |value: &T| predicate.test(value),
        |f| Arc::new(f)
    );

    // Generates: and(), or(), nand(), xor(), nor()
    impl_shared_predicate_methods!(ArcPredicate<T>, Send + Sync + 'static);
}

impl<T> Not for ArcPredicate<T>
where
    T: 'static,
{
    type Output = ArcPredicate<T>;

    fn not(self) -> Self::Output {
        let metadata = self.metadata;
        let function = self.function;
        ArcPredicate::new_with_metadata(move |value: &T| !function(value), metadata)
    }
}

impl<T> Not for &ArcPredicate<T>
where
    T: 'static,
{
    type Output = ArcPredicate<T>;

    fn not(self) -> Self::Output {
        let function = self.function.clone();
        ArcPredicate::new_with_metadata(move |value: &T| !function(value), self.metadata.clone())
    }
}

// Generates: impl Clone for ArcPredicate<T>
impl_predicate_clone!(ArcPredicate<T>);

// Generates: impl Debug for ArcPredicate<T> and impl Display for
// ArcPredicate<T>
impl_predicate_debug_display!(ArcPredicate<T>);

// Implements Predicate trait for ArcPredicate<T>
impl<T> Predicate<T> for ArcPredicate<T> {
    #[inline(always)]
    fn test(&self, value: &T) -> bool {
        (self.function)(value)
    }
}
