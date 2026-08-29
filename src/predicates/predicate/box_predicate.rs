// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxPredicate` public type.

use std::ops::Not;

use super::ALWAYS_FALSE_NAME;
use super::ALWAYS_TRUE_NAME;
use crate::Predicate;
use crate::predicates::macros::impl_box_predicate_methods;
use crate::predicates::macros::impl_predicate_common_methods;
use crate::predicates::macros::impl_predicate_debug_display;

/// A Box-based predicate with single ownership.
///
/// This type is suitable when the predicate needs a stable single-owner type
/// but does not need to be cloned or shared. Composition methods consume
/// `self`, reflecting the single-ownership model.
///
///
/// # Examples
///
/// ```rust
/// # {
/// use qubit_function::{Predicate, BoxPredicate};
///
/// let pred = BoxPredicate::new(|x: &i32| *x > 0);
/// assert!(pred.test(&5));
///
/// // Chaining consumes the predicate
/// let combined = pred.and(BoxPredicate::new(|x: &i32| x % 2 == 0));
/// assert!(combined.test(&4));
/// # }
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxPredicate<T> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn Fn(&T) -> bool>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> BoxPredicate<T> {
    // Generates: new(), new_with_name(), name(), set_name(), always_true(),
    // always_false()
    impl_predicate_common_methods!(
        BoxPredicate<T>,
        semantic(Predicate<T> + 'static),
        |predicate| move |value: &T| predicate.test(value),
        |f| Box::new(f)
    );

    // Generates: and(), or(), nand(), xor(), nor()
    impl_box_predicate_methods!(BoxPredicate<T>);
}

impl<T> Not for BoxPredicate<T>
where
    T: 'static,
{
    type Output = BoxPredicate<T>;

    fn not(self) -> Self::Output {
        let metadata = self.metadata;
        let function = self.function;
        BoxPredicate::new_with_metadata(move |value: &T| !function(value), metadata)
    }
}

// Generates: impl Debug for BoxPredicate<T> and impl Display for
// BoxPredicate<T>
impl_predicate_debug_display!(BoxPredicate<T>);

// Implements Predicate trait for BoxPredicate<T>
impl<T> Predicate<T> for BoxPredicate<T> {
    #[inline(always)]
    fn test(&self, value: &T) -> bool {
        (self.function)(value)
    }
}
