// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcBiPredicate` public type.

use std::ops::Not;
use std::sync::Arc;

use super::ALWAYS_FALSE_NAME;
use super::ALWAYS_TRUE_NAME;
use super::SendSyncBiPredicateFn;
use crate::BiPredicate;
use crate::predicates::macros::impl_predicate_clone;
use crate::predicates::macros::impl_predicate_common_methods;
use crate::predicates::macros::impl_predicate_debug_display;
use crate::predicates::macros::impl_shared_predicate_methods;

/// An Arc-based bi-predicate with thread-safe shared ownership.
///
/// This type is suitable for scenarios where the bi-predicate needs
/// to be shared across threads. Composition methods borrow `&self`,
/// allowing the original bi-predicate to remain usable after
/// composition.
///
///
/// # Examples
///
/// ```rust
/// # {
/// use qubit_function::{BiPredicate, ArcBiPredicate};
///
/// let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
/// assert!(pred.test(&5, &3));
///
/// // Original bi-predicate remains usable after composition
/// let combined = pred.and(ArcBiPredicate::new(|x: &i32, y: &i32| x > y));
/// assert!(pred.test(&5, &3));  // Still works
///
/// // Can be cloned and sent across threads
/// let pred_clone = pred.clone();
/// std::thread::spawn(move || {
///     assert!(pred_clone.test(&10, &5));
/// }).join().expect("thread should not panic");
/// # }
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcBiPredicate<T, U> {
    /// The wrapped callback implementation.
    pub(super) function: Arc<SendSyncBiPredicateFn<T, U>>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, U> ArcBiPredicate<T, U> {
    // Generates: new(), new_with_name(), name(), set_name(), always_true(),
    // always_false()
    impl_predicate_common_methods!(
        ArcBiPredicate<T, U>,
        semantic (BiPredicate<T, U> + Send + Sync + 'static),
        |predicate| move |first: &T, second: &U| predicate.test(first, second),
        |f| Arc::new(f)
    );

    // Generates: and(), or(), nand(), xor(), nor()
    impl_shared_predicate_methods!(
        ArcBiPredicate<T, U>,
        Send + Sync + 'static
    );
}

impl<T, U> Not for ArcBiPredicate<T, U>
where
    T: 'static,
    U: 'static,
{
    type Output = ArcBiPredicate<T, U>;

    fn not(self) -> Self::Output {
        let metadata = self.metadata;
        let function = self.function;
        ArcBiPredicate::new_with_metadata(move |first: &T, second: &U| !function(first, second), metadata)
    }
}

impl<T, U> Not for &ArcBiPredicate<T, U>
where
    T: 'static,
    U: 'static,
{
    type Output = ArcBiPredicate<T, U>;

    fn not(self) -> Self::Output {
        let function = self.function.clone();
        ArcBiPredicate::new_with_metadata(
            move |first: &T, second: &U| !function(first, second),
            self.metadata.clone(),
        )
    }
}

// Generates: impl Clone for ArcBiPredicate<T, U>
impl_predicate_clone!(ArcBiPredicate<T, U>);

// Generates: impl Debug for ArcBiPredicate<T, U> and impl Display for
// ArcBiPredicate<T, U>
impl_predicate_debug_display!(ArcBiPredicate<T, U>);

// Implements BiPredicate trait for ArcBiPredicate<T, U>
impl<T, U> BiPredicate<T, U> for ArcBiPredicate<T, U> {
    #[inline(always)]
    fn test(&self, first: &T, second: &U) -> bool {
        (self.function)(first, second)
    }
}
