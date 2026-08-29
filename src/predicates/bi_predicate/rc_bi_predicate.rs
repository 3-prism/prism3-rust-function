// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcBiPredicate` public type.

use std::ops::Not;
use std::rc::Rc;

use super::ALWAYS_FALSE_NAME;
use super::ALWAYS_TRUE_NAME;
use super::BiPredicateFn;
use crate::BiPredicate;
use crate::predicates::macros::impl_predicate_clone;
use crate::predicates::macros::impl_predicate_common_methods;
use crate::predicates::macros::impl_predicate_debug_display;
use crate::predicates::macros::impl_shared_predicate_methods;

/// An Rc-based bi-predicate with single-threaded shared ownership.
///
/// This type is suitable for scenarios where the bi-predicate needs
/// to be reused in a single-threaded context. Composition methods
/// borrow `&self`, allowing the original bi-predicate to remain
/// usable after composition.
///
///
/// # Examples
///
/// ```rust
/// # {
/// use qubit_function::{BiPredicate, RcBiPredicate};
///
/// let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
/// assert!(pred.test(&5, &3));
///
/// // Original bi-predicate remains usable after composition
/// let combined = pred.and(RcBiPredicate::new(|x: &i32, y: &i32| x > y));
/// assert!(pred.test(&5, &3));  // Still works
/// # }
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcBiPredicate<T, U> {
    /// The wrapped callback implementation.
    pub(super) function: Rc<BiPredicateFn<T, U>>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T, U> RcBiPredicate<T, U> {
    // Generates: new(), new_with_name(), name(), set_name(), always_true(),
    // always_false()
    impl_predicate_common_methods!(
        RcBiPredicate<T, U>,
        semantic (BiPredicate<T, U> + 'static),
        |predicate| move |first: &T, second: &U| predicate.test(first, second),
        |f| Rc::new(f)
    );

    // Generates: and(), or(), nand(), xor(), nor()
    impl_shared_predicate_methods!(RcBiPredicate<T, U>, 'static);
}

impl<T, U> Not for RcBiPredicate<T, U>
where
    T: 'static,
    U: 'static,
{
    type Output = RcBiPredicate<T, U>;

    fn not(self) -> Self::Output {
        let metadata = self.metadata;
        let function = self.function;
        RcBiPredicate::new_with_metadata(move |first: &T, second: &U| !function(first, second), metadata)
    }
}

impl<T, U> Not for &RcBiPredicate<T, U>
where
    T: 'static,
    U: 'static,
{
    type Output = RcBiPredicate<T, U>;

    fn not(self) -> Self::Output {
        let function = self.function.clone();
        RcBiPredicate::new_with_metadata(
            move |first: &T, second: &U| !function(first, second),
            self.metadata.clone(),
        )
    }
}

// Generates: impl Clone for RcBiPredicate<T, U>
impl_predicate_clone!(RcBiPredicate<T, U>);

// Generates: impl Debug for RcBiPredicate<T, U> and impl Display for
// RcBiPredicate<T, U>
impl_predicate_debug_display!(RcBiPredicate<T, U>);

// Implements BiPredicate trait for RcBiPredicate<T, U>
impl<T, U> BiPredicate<T, U> for RcBiPredicate<T, U> {
    #[inline(always)]
    fn test(&self, first: &T, second: &U) -> bool {
        (self.function)(first, second)
    }
}
