// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxSupplierOnce` public type.

use crate::Predicate;
use crate::SupplierOnce;
use crate::Transformer;
use crate::macros::impl_closure_once_trait;
use crate::suppliers::macros::impl_box_supplier_methods;
use crate::suppliers::macros::impl_supplier_common_methods;
use crate::suppliers::macros::impl_supplier_debug_display;

// ==========================================================================
// BoxSupplierOnce - One-time Supplier Implementation
// ==========================================================================

/// Box-based one-time supplier.
///
/// Uses `Box<dyn FnOnce() -> T>` for one-time value generation.
/// Can only call `get()` once, consuming the supplier.
///
/// # Examples
///
/// ## Lazy Initialization
///
/// ```rust
/// use qubit_function::{BoxSupplierOnce, SupplierOnce};
///
/// let once = BoxSupplierOnce::new(|| {
///     println!("Expensive initialization");
///     42
/// });
///
/// let value = once.get(); // Prints: Expensive initialization
/// assert_eq!(value, 42);
/// ```
///
/// ## Moving Captured Values
///
/// ```rust
/// use qubit_function::{BoxSupplierOnce, SupplierOnce};
///
/// let resource = String::from("data");
/// let once = BoxSupplierOnce::new(move || resource);
///
/// let value = once.get();
/// assert_eq!(value, "data");
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxSupplierOnce<T> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn FnOnce() -> T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> BoxSupplierOnce<T> {
    // Generates: new(), new_with_name(), name(), set_name(), constant()
    impl_supplier_common_methods!(BoxSupplierOnce<T>, (FnOnce() -> T + 'static), |f| Box::new(f));

    // Generates: map(), filter(), zip()
    impl_box_supplier_methods!(BoxSupplierOnce<T>, SupplierOnce);
}

// Generates: implement SupplierOnce for BoxSupplierOnce<T>
impl<T> SupplierOnce<T> for BoxSupplierOnce<T> {
    #[inline(always)]
    fn get(self) -> T {
        (self.function)()
    }
}

// Generates: Debug and Display implementations for BoxSupplierOnce<T>
impl_supplier_debug_display!(BoxSupplierOnce<T>);

// ==========================================================================
// Implement SupplierOnce for Closures
// ==========================================================================

// Implement SupplierOnce for all FnOnce() -> T using macro
impl_closure_once_trait!(
    SupplierOnce<T>,
    get,
    BoxSupplierOnce,
    FnOnce() -> T
);
