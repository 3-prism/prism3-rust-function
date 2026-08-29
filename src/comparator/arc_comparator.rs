// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `ArcComparator` public type.

use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

use crate::Comparator;
use crate::internal::CallbackMetadata;
use crate::macros::impl_common_name_methods;

/// The erased callback representation used by this implementation.
type ArcComparatorFn<T> = Arc<dyn Fn(&T, &T) -> Ordering + Send + Sync>;

/// An Arc-based thread-safe comparator with shared ownership.
///
/// `ArcComparator` wraps a comparator function in an `Arc`, providing
/// thread-safe shared ownership semantics. It is cloneable and uses `&self`
/// in composition operations.
///
/// # Type Parameters
///
/// * `T` - The type of values being compared
///
/// # Examples
///
/// ```rust
/// use qubit_function::comparator::{Comparator, ArcComparator};
/// use std::cmp::Ordering;
///
/// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
/// let cloned = cmp.clone();
/// assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
/// assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
/// ```
#[derive(Clone)]
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct ArcComparator<T> {
    /// The wrapped callback implementation.
    pub(super) function: ArcComparatorFn<T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: CallbackMetadata,
}

impl<T> ArcComparator<T> {
    /// Creates a new `ArcComparator` from a comparator implementation.
    ///
    /// # Parameters
    ///
    /// * `source` - The comparator implementation to wrap
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::ArcComparator;
    ///
    /// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// ```
    #[inline]
    pub fn new<F>(source: F) -> Self
    where
        F: Comparator<T> + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(move |left: &T, right: &T| source.compare(left, right)),
            metadata: CallbackMetadata::unnamed(),
        }
    }

    /// Creates a named `ArcComparator` from a thread-safe comparator.
    ///
    /// # Parameters
    ///
    /// * `name` - The diagnostic name assigned to the comparator.
    /// * `source` - The thread-safe comparator implementation to wrap.
    ///
    /// # Returns
    ///
    /// A named `ArcComparator` wrapping `source`.
    #[inline]
    pub fn new_with_name<F>(name: &str, source: F) -> Self
    where
        F: Comparator<T> + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(move |left: &T, right: &T| source.compare(left, right)),
            metadata: CallbackMetadata::named(name),
        }
    }

    /// Creates an `ArcComparator` with an optional diagnostic name.
    ///
    /// # Parameters
    ///
    /// * `source` - The thread-safe comparator implementation to wrap.
    /// * `name` - The optional diagnostic name assigned to the comparator.
    ///
    /// # Returns
    ///
    /// An `ArcComparator` containing the supplied optional name.
    #[inline]
    pub fn new_with_optional_name<F>(source: F, name: Option<String>) -> Self
    where
        F: Comparator<T> + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(move |left: &T, right: &T| source.compare(left, right)),
            metadata: CallbackMetadata::from_optional_name(name),
        }
    }

    /// Creates a comparator while preserving existing callback metadata.
    ///
    /// # Parameters
    ///
    /// * `source` - The thread-safe comparator implementation to wrap.
    /// * `metadata` - The callback metadata to preserve.
    ///
    /// # Returns
    ///
    /// An `ArcComparator` containing `metadata`.
    #[inline]
    pub(crate) fn new_with_metadata<F>(source: F, metadata: CallbackMetadata) -> Self
    where
        F: Comparator<T> + Send + Sync + 'static,
    {
        Self {
            function: Arc::new(move |left: &T, right: &T| source.compare(left, right)),
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
    /// A new `ArcComparator` that compares by the extracted key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::{Comparator, ArcComparator};
    /// use std::cmp::Ordering;
    ///
    /// #[derive(Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: i32,
    /// }
    ///
    /// let by_age = ArcComparator::comparing(|p: &Person| &p.age);
    /// let p1 = Person { name: "Alice".to_string(), age: 30 };
    /// let p2 = Person { name: "Bob".to_string(), age: 25 };
    /// assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    /// ```
    #[inline]
    pub fn comparing<K, F>(key_fn: F) -> Self
    where
        K: Ord,
        F: Fn(&T) -> &K + Send + Sync + 'static,
    {
        ArcComparator::new(move |a: &T, b: &T| key_fn(a).cmp(key_fn(b)))
    }

    impl_common_name_methods!("comparator");

    /// Returns a comparator that imposes the reverse ordering.
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` that reverses the comparison order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::{Comparator, ArcComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let rev = cmp.reversed();
    /// assert_eq!(rev.compare(&5, &3), Ordering::Less);
    /// assert_eq!(cmp.compare(&5, &3), Ordering::Greater); // cmp still works
    /// ```
    #[inline]
    pub fn reversed(&self) -> Self
    where
        T: 'static,
    {
        let self_fn = self.function.clone();
        ArcComparator::new_with_metadata(move |a: &T, b: &T| self_fn(b, a), self.metadata.clone())
    }

    /// Returns a comparator that uses this comparator first, then another
    /// comparator if this one considers the values equal.
    ///
    /// # Parameters
    ///
    /// * `other` - The comparator to move into the result for tie-breaking;
    ///   clone shared wrappers first if they must remain available
    ///
    /// # Returns
    ///
    /// A new `ArcComparator` that chains this comparator with another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::comparator::{Comparator, ArcComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp1 = ArcComparator::new(|a: &i32, b: &i32| {
    ///     (a % 2).cmp(&(b % 2))
    /// });
    /// let cmp2 = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let chained = cmp1.then_comparing(cmp2.clone());
    /// assert_eq!(chained.compare(&4, &2), Ordering::Greater);
    /// ```
    #[inline]
    pub fn then_comparing<C>(&self, other: C) -> Self
    where
        T: 'static,
        C: Comparator<T> + Send + Sync + 'static,
    {
        let first = self.function.clone();
        ArcComparator::new(move |a: &T, b: &T| match first(a, b) {
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
    /// use qubit_function::comparator::{Comparator, ArcComparator};
    /// use std::cmp::Ordering;
    ///
    /// let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
    /// let func = cmp.into_fn();
    /// assert_eq!(func(&5, &3), Ordering::Greater);
    /// ```
    #[must_use = "the returned comparator closure should be stored or invoked"]
    #[inline(always)]
    pub fn into_fn(self) -> impl Fn(&T, &T) -> Ordering {
        move |a: &T, b: &T| (self.function)(a, b)
    }
}

impl<T> Comparator<T> for ArcComparator<T> {
    #[inline(always)]
    fn compare(&self, a: &T, b: &T) -> Ordering {
        (self.function)(a, b)
    }
}

impl<T> fmt::Debug for ArcComparator<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArcComparator")
            .field("name", &self.metadata.name())
            .finish()
    }
}

impl<T> fmt::Display for ArcComparator<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.metadata.name() {
            Some(name) => write!(formatter, "ArcComparator({name})"),
            None => formatter.write_str("ArcComparator"),
        }
    }
}
