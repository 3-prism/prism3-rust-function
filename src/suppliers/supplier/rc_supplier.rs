// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `RcSupplier` public type.

use std::rc::Rc;

use crate::Predicate;
use crate::Supplier;
use crate::Transformer;
use crate::suppliers::macros::impl_shared_supplier_methods;
use crate::suppliers::macros::impl_supplier_clone;
use crate::suppliers::macros::impl_supplier_common_methods;
use crate::suppliers::macros::impl_supplier_debug_display;

// ======================================================================
// RcSupplier - Single-threaded Shared Ownership
// ======================================================================

/// Single-threaded shared ownership stateless supplier.
///
/// Uses `Rc<dyn Fn() -> T>` for single-threaded shared ownership.
/// Can be cloned but not sent across threads.
///
/// # Ownership Model
///
/// Like `ArcSupplier`, methods borrow `&self` instead of
/// consuming `self`:
///
/// ```rust
/// # {
/// use qubit_function::{RcSupplier, Supplier};
///
/// let source = RcSupplier::new(|| 10);
/// let mapped = source.map(|x| x * 2);
/// // source is still usable here!
/// # }
/// ```
///
/// # Examples
///
/// ## Shared Factory
///
/// ```rust
/// use qubit_function::{RcSupplier, Supplier};
///
/// let factory = RcSupplier::new(|| {
///     String::from("Hello")
/// });
///
/// let f1 = factory.clone();
/// let f2 = factory.clone();
/// assert_eq!(f1.get(), "Hello");
/// assert_eq!(f2.get(), "Hello");
/// ```
///
/// ## Reusable Transformations
///
/// ```rust
/// # {
/// use qubit_function::{RcSupplier, Supplier};
///
/// let base = RcSupplier::new(|| 10);
/// let doubled = base.map(|x| x * 2);
/// let tripled = base.map(|x| x * 3);
///
/// assert_eq!(base.get(), 10);
/// assert_eq!(doubled.get(), 20);
/// assert_eq!(tripled.get(), 30);
/// # }
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct RcSupplier<T> {
    /// The wrapped callback implementation.
    pub(super) function: Rc<dyn Fn() -> T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> RcSupplier<T> {
    // Generates: new(), new_with_name(), name(), set_name(), constant()
    impl_supplier_common_methods!(RcSupplier<T>, (Fn() -> T + 'static), |f| { Rc::new(f) });

    // Generates: map(), filter(), zip()
    impl_shared_supplier_methods!(
        RcSupplier<T>,
        Supplier,
        ('static)
    );
}

// Generates: Debug and Display implementations for RcSupplier<T>
impl_supplier_debug_display!(RcSupplier<T>);

// Generates: Clone implementation for RcSupplier<T>
impl_supplier_clone!(RcSupplier<T>);

impl<T> Supplier<T> for RcSupplier<T> {
    #[inline(always)]
    fn get(&self) -> T {
        (self.function)()
    }
}
