// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcPredicate` public type.

use std::ops::Not;
use std::rc::Rc;

use super::ALWAYS_FALSE_NAME;
use super::ALWAYS_TRUE_NAME;
use crate::Predicate;
use crate::predicates::macros::impl_predicate_clone;
use crate::predicates::macros::impl_predicate_common_methods;
use crate::predicates::macros::impl_predicate_debug_display;
use crate::predicates::macros::impl_shared_predicate_methods;

/// An Rc-based predicate with single-threaded shared ownership.
///
/// This type is suitable for scenarios where the predicate needs to be
/// reused in a single-threaded context. Composition methods borrow `&self`,
/// allowing the original predicate to remain usable after composition.
///
///
/// # Examples
///
/// ```rust
/// # {
/// use qubit_function::{Predicate, RcPredicate};
///
/// let pred = RcPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
///
/// // Original predicate remains usable after composition
/// let combined = pred.and(RcPredicate::new(|x: &i32| x % 2 == 0));
/// assert!(pred.test(&5));  // Still works
/// # }
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcPredicate<T> {
    /// The wrapped callback implementation.
    pub(super) function: Rc<dyn Fn(&T) -> bool>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> RcPredicate<T> {
    // Generates: new(), new_with_name(), name(), set_name(), always_true(),
    // always_false()
    impl_predicate_common_methods!(
        RcPredicate<T>,
        semantic(Predicate<T> + 'static),
        |predicate| move |value: &T| predicate.test(value),
        |f| Rc::new(f)
    );

    // Generates: and(), or(), nand(), xor(), nor()
    impl_shared_predicate_methods!(RcPredicate<T>, 'static);
}

impl<T> Not for RcPredicate<T>
where
    T: 'static,
{
    type Output = RcPredicate<T>;

    fn not(self) -> Self::Output {
        let metadata = self.metadata;
        let function = self.function;
        RcPredicate::new_with_metadata(move |value: &T| !function(value), metadata)
    }
}

impl<T> Not for &RcPredicate<T>
where
    T: 'static,
{
    type Output = RcPredicate<T>;

    fn not(self) -> Self::Output {
        let function = self.function.clone();
        RcPredicate::new_with_metadata(move |value: &T| !function(value), self.metadata.clone())
    }
}

// Generates: impl Clone for RcPredicate<T>
impl_predicate_clone!(RcPredicate<T>);

// Generates: impl Debug for RcPredicate<T> and impl Display for RcPredicate<T>
impl_predicate_debug_display!(RcPredicate<T>);

// Implements Predicate trait for RcPredicate<T>
impl<T> Predicate<T> for RcPredicate<T> {
    #[inline(always)]
    fn test(&self, value: &T) -> bool {
        (self.function)(value)
    }
}
