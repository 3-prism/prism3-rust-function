// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxComparator` public type.

use std::cmp::Ordering;
use std::fmt;

use crate::Comparator;
use crate::internal::CallbackMetadata;
use crate::macros::impl_common_name_methods;

/// The erased callback representation used by this implementation.
type BoxComparatorFn<T> = Box<dyn Fn(&T, &T) -> Ordering>;

/// A boxed comparator with single ownership.
///
/// `BoxComparator` wraps a comparator function in a `Box`, providing single
/// ownership semantics. It is not cloneable and consumes `self` in
/// composition operations.
///
/// # Type Parameters
///
/// * `T` - The type of values being compared
///
/// # Examples
///
/// ```rust
/// use qubit_function::comparator::{Comparator, BoxComparator};
/// use std::cmp::Ordering;
///
/// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
/// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxComparator<T> {
    /// The wrapped callback implementation.
    pub(super) function: BoxComparatorFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: CallbackMetadata,
}

impl<T> BoxComparator<T> {
    /// Creates a new `BoxComparator` from a comparator implementation.
    ///
    /// # Parameters
    ///
    /// * `source` - The comparator implementation to wrap
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::BoxComparator;
    ///
    /// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// ```
    #[inline]
    pub fn new<F>(source: F) -> Self
    where
        F: Comparator<T> + 'static,
    {
        Self {
            function: Box::new(move |left: &T, right: &T| source.compare(left, right)),
            metadata: CallbackMetadata::unnamed(),
        }
    }

    /// Creates a named `BoxComparator` from a comparator implementation.
    ///
    /// # Parameters
    ///
    /// * `name` - The diagnostic name assigned to the comparator.
    /// * `source` - The comparator implementation to wrap.
    ///
    /// # Returns
    ///
    /// A named `BoxComparator` wrapping `source`.
    #[inline]
    pub fn new_with_name<F>(name: &str, source: F) -> Self
    where
        F: Comparator<T> + 'static,
    {
        Self {
            function: Box::new(move |left: &T, right: &T| source.compare(left, right)),
            metadata: CallbackMetadata::named(name),
        }
    }

    /// Creates a `BoxComparator` with an optional diagnostic name.
    ///
    /// # Parameters
    ///
    /// * `source` - The comparator implementation to wrap.
    /// * `name` - The optional diagnostic name assigned to the comparator.
    ///
    /// # Returns
    ///
    /// A `BoxComparator` containing the supplied optional name.
    #[inline]
    pub fn new_with_optional_name<F>(source: F, name: Option<String>) -> Self
    where
        F: Comparator<T> + 'static,
    {
        Self {
            function: Box::new(move |left: &T, right: &T| source.compare(left, right)),
            metadata: CallbackMetadata::from_optional_name(name),
        }
    }

    /// Creates a comparator while preserving existing callback metadata.
    ///
    /// # Parameters
    ///
    /// * `source` - The comparator implementation to wrap.
    /// * `metadata` - The callback metadata to preserve.
    ///
    /// # Returns
    ///
    /// A `BoxComparator` containing `metadata`.
    #[inline]
    pub(crate) fn new_with_metadata<F>(source: F, metadata: CallbackMetadata) -> Self
    where
        F: Comparator<T> + 'static,
    {
        Self {
            function: Box::new(move |left: &T, right: &T| source.compare(left, right)),
            metadata,
        }
    }

    /// Returns a comparator that compares values by a key extracted by the
    /// given function.
    ///
    /// # Parameters
    ///
    /// * `key_fn` - A function that extracts a comparable key from values
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that compares by the extracted key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// #[derive(Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let by_age = BoxComparator::comparing(|p: &Person| &p.age);
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Bob".to_string(), age: 25 };
    /// assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    /// ```
    #[inline]
    pub fn comparing<K, F>(key_fn: F) -> Self
    where
        K: Ord,
        F: Fn(&T) -> &K + 'static,
    {
        BoxComparator::new(move |a: &T, b: &T| key_fn(a).cmp(key_fn(b)))
    }

    impl_common_name_methods!("comparator");

    /// Returns a comparator that imposes the reverse ordering.
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that reverses the comparison order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let rev = cmp.reversed();
    /// assert_eq!(rev.compare(&5, &3), Ordering::Less);
    /// ```
    #[inline]
    pub fn reversed(self) -> Self
    where
        T: 'static,
    {
        let metadata = self.metadata;
        let function = self.function;
        BoxComparator::new_with_metadata(move |a: &T, b: &T| function(b, a), metadata)
    }

    /// Returns a comparator that uses this comparator first, then another
    /// comparator if this one considers the values equal.
    ///
    /// # Parameters
    ///
    /// * `other` - The comparator to use for tie-breaking. **Note: This
    ///   parameter is passed by value and will transfer ownership.** If you
    ///   need to preserve the original comparator, clone it first (if it
    ///   implements `Clone`). Can be:
    ///   - A `BoxComparator<T>`
    ///   - An `RcComparator<T>`
    ///   - An `ArcComparator<T>`
    ///   - Any type implementing `Comparator<T>`
    ///
    /// # Returns
    ///
    /// A new `BoxComparator` that chains this comparator with another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// #[derive(Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let by_name = BoxComparator::new(|a: &Person, b: &Person| {
    ///     a.name.cmp(&b.name)
    /// });
    /// let by_age = BoxComparator::new(|a: &Person, b: &Person| {
    ///     a.age.cmp(&b.age)
    /// });
    ///
    /// // by_age is moved here
    /// let cmp = by_name.then_comparing(by_age);
    ///
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Alice".to_string(), age: 25 };
    /// assert_eq!(cmp.compare(&p1, &p2), Ordering::Greater);
    /// // by_age.compare(&p1, &p2); // Would not compile - moved
    /// ```
    #[inline]
    pub fn then_comparing<C>(self, other: C) -> Self
    where
        T: 'static,
        C: Comparator<T> + 'static,
    {
        BoxComparator::new(move |a: &T, b: &T| match (self.function)(a, b) {
            Ordering::Equal => other.compare(a, b),
            ord => ord,
        })
    }

    /// Converts this comparator into a closure.
    ///
    /// # Returns
    ///
    /// A closure that implements `Fn(&T, &T) -> Ordering`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::{Comparator, BoxComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let func = cmp.into_fn();
    /// assert_eq!(func(&5, &3), Ordering::Greater);
    /// ```
    #[must_use = "the returned comparator closure should be stored or invoked"]
    #[inline(always)]
    pub fn into_fn(self) -> impl Fn(&T, &T) -> Ordering {
        move |a: &T, b: &T| (self.function)(a, b)
    }
}

impl<T> Comparator<T> for BoxComparator<T> {
    #[inline(always)]
    fn compare(&self, a: &T, b: &T) -> Ordering {
        (self.function)(a, b)
    }
}

impl<T> fmt::Debug for BoxComparator<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoxComparator")
            .field("name", &self.metadata.name())
            .finish()
    }
}

impl<T> fmt::Display for BoxComparator<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.metadata.name() {
            Some(name) => write!(formatter, "BoxComparator({name})"),
            None => formatter.write_str("BoxComparator"),
        }
    }
}
