// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcStatefulSupplier` public type.

use std::cell::RefCell;
use std::rc::Rc;

use crate::Predicate;
use crate::StatefulSupplier;
use crate::Transformer;
use crate::suppliers::macros::impl_shared_supplier_methods;
use crate::suppliers::macros::impl_supplier_clone;
use crate::suppliers::macros::impl_supplier_common_methods;
use crate::suppliers::macros::impl_supplier_debug_display;

// ==========================================================================
// RcStatefulSupplier - Single-threaded Shared Ownership Implementation
// ==========================================================================

/// Single-threaded shared ownership supplier.
///
/// Uses `Rc<RefCell<dyn FnMut() -> T>>` for single-threaded shared
/// ownership. Can be cloned but not sent across threads.
///
/// # Ownership Model
///
/// Like `ArcStatefulSupplier`, methods borrow `&self` instead of consuming
/// `self`:
///
///
/// ```rust
/// # {
/// use qubit_function::{RcStatefulSupplier, StatefulSupplier};
///
/// let source = RcStatefulSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// # }
/// ```
///
/// # Examples
///
/// ## Shared Counter
///
/// ```rust
/// use qubit_function::{RcStatefulSupplier, StatefulSupplier};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let counter = Rc::new(RefCell::new(0));
/// let counter_clone = Rc::clone(&counter);
///
/// let supplier = RcStatefulSupplier::new(move || {
///     let mut c = counter_clone.borrow_mut();
///     *c += 1;
///     *c
/// });
///
/// let mut s1 = supplier.clone();
/// let mut s2 = supplier.clone();
/// assert_eq!(s1.get(), 1);
/// assert_eq!(s2.get(), 2);
/// ```
///
/// ## Reusable Transformations
///
///
/// ```rust
/// # {
/// use qubit_function::{RcStatefulSupplier, StatefulSupplier};
///
/// let base = RcStatefulSupplier::new(|| 10);
/// let doubled = base.map(|x| x * 2);
/// let tripled = base.map(|x| x * 3);
///
/// let mut b = base;
/// let mut d = doubled;
/// let mut t = tripled;
/// assert_eq!(b.get(), 10);
/// assert_eq!(d.get(), 20);
/// assert_eq!(t.get(), 30);
/// # }
/// ```
/// # Borrowing and reentrancy
///
/// Each call holds a mutable `RefCell` borrow while the user callback runs.
/// Synchronous re-entry through the same shared wrapper panics with a borrow
/// error. Mutations completed before a panic are not rolled back.
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcStatefulSupplier<T> {
    /// The wrapped callback implementation.
    pub(super) function: Rc<RefCell<dyn FnMut() -> T>>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> RcStatefulSupplier<T> {
    // Generates: new(), new_with_name(), name(), set_name(), constant()
    impl_supplier_common_methods!(RcStatefulSupplier<T>, (FnMut() -> T + 'static), |f| Rc::new(
        RefCell::new(f)
    ));

    // Generates: map(), filter(), zip()
    impl_shared_supplier_methods!(
        RcStatefulSupplier<T>,
        StatefulSupplier,
        ('static)
    );

    /// Creates a memoizing supplier.
    ///
    /// The cache is stored directly inside the returned callback and is
    /// protected by the same outer `RefCell` borrow as callback execution.
    ///
    /// # Returns
    ///
    /// A new memoized `RcStatefulSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::{RcStatefulSupplier, StatefulSupplier};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let call_count = Rc::new(RefCell::new(0));
    /// let call_count_clone = Rc::clone(&call_count);
    /// let source = RcStatefulSupplier::new(move || {
    ///     let mut c = call_count_clone.borrow_mut();
    ///     *c += 1;
    ///     42
    /// });
    /// let memoized = source.memoize();
    ///
    /// let mut s = memoized;
    /// assert_eq!(s.get(), 42); // Calls underlying function
    /// assert_eq!(s.get(), 42); // Returns cached value
    /// assert_eq!(*call_count.borrow(), 1);
    /// ```
    pub fn memoize(&self) -> RcStatefulSupplier<T>
    where
        T: Clone + 'static,
    {
        let self_fn = Rc::clone(&self.function);
        let metadata = self.metadata.clone();
        let mut cache: Option<T> = None;
        RcStatefulSupplier::new_with_metadata(
            move || {
                if let Some(ref cached) = cache {
                    cached.clone()
                } else {
                    let value = self_fn.borrow_mut()();
                    cache = Some(value.clone());
                    value
                }
            },
            metadata,
        )
    }
}

// Generates: Debug and Display implementations for RcStatefulSupplier<T>
impl_supplier_debug_display!(RcStatefulSupplier<T>);

// Generates: Clone implementation for RcStatefulSupplier<T>
impl_supplier_clone!(RcStatefulSupplier<T>);

impl<T> StatefulSupplier<T> for RcStatefulSupplier<T> {
    #[inline]
    fn get(&mut self) -> T {
        let mut function = self.function.borrow_mut();
        function()
    }
}
