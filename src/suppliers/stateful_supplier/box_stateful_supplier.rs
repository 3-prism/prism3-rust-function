// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxStatefulSupplier` public type.

use crate::Predicate;
use crate::StatefulSupplier;
use crate::Transformer;
use crate::suppliers::macros::impl_box_supplier_methods;
use crate::suppliers::macros::impl_supplier_common_methods;
use crate::suppliers::macros::impl_supplier_debug_display;

// ==========================================================================
// BoxStatefulSupplier - Single Ownership Implementation
// ==========================================================================

/// Box-based single ownership supplier.
///
/// Uses `Box<dyn FnMut() -> T>` for single ownership scenarios.
/// This is the most lightweight supplier with zero reference
/// counting overhead.
///
/// # Ownership Model
///
/// Methods consume `self` (move semantics). When you call a method
/// like `map()`, the original supplier is consumed and you get a new
/// one:
///
///
/// ```rust
/// # {
/// use qubit_function::{BoxStatefulSupplier, StatefulSupplier};
///
/// let supplier = BoxStatefulSupplier::new(|| 10);
/// let mapped = supplier.map(|x| x * 2);
/// // supplier is no longer usable here
/// # }
/// ```
///
/// # Examples
///
/// ## Counter
///
/// ```rust
/// use qubit_function::{BoxStatefulSupplier, StatefulSupplier};
///
/// let mut counter = 0;
/// let mut supplier = BoxStatefulSupplier::new(move || {
///     counter += 1;
///     counter
/// });
///
/// assert_eq!(supplier.get(), 1);
/// assert_eq!(supplier.get(), 2);
/// ```
///
/// ## Method Chaining
///
///
/// ```rust
/// # {
/// use qubit_function::{BoxStatefulSupplier, StatefulSupplier};
///
/// let mut pipeline = BoxStatefulSupplier::new(|| 10)
///     .map(|x| x * 2)
///     .map(|x| x + 5);
///
/// assert_eq!(pipeline.get(), 25);
/// # }
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxStatefulSupplier<T> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn FnMut() -> T>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> BoxStatefulSupplier<T> {
    // Generates: new(), new_with_name(), name(), set_name(), constant()
    impl_supplier_common_methods!(BoxStatefulSupplier<T>, (FnMut() -> T + 'static), |f| { Box::new(f) });

    // Generates: map(), filter(), zip()
    impl_box_supplier_methods!(BoxStatefulSupplier<T>, StatefulSupplier);

    /// Creates a memoizing supplier.
    ///
    /// Returns a new supplier that caches the first value it
    /// produces. All subsequent calls return the cached value.
    /// The cache is stored directly inside the returned callback.
    ///
    /// # Returns
    ///
    /// A new memoized `BoxStatefulSupplier<T>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_function::{BoxStatefulSupplier, StatefulSupplier};
    ///
    /// let mut call_count = 0;
    /// let mut memoized = BoxStatefulSupplier::new(move || {
    ///     call_count += 1;
    ///     42
    /// }).memoize();
    ///
    /// assert_eq!(memoized.get(), 42); // Calls underlying function
    /// assert_eq!(memoized.get(), 42); // Returns cached value
    /// ```
    pub fn memoize(self) -> BoxStatefulSupplier<T>
    where
        T: Clone + 'static,
    {
        let metadata = self.metadata;
        let mut function = self.function;
        let mut cache: Option<T> = None;
        BoxStatefulSupplier::new_with_metadata(
            move || {
                if let Some(ref cached) = cache {
                    cached.clone()
                } else {
                    let value = function();
                    cache = Some(value.clone());
                    value
                }
            },
            metadata,
        )
    }
}

// Generates: Debug and Display implementations for BoxStatefulSupplier<T>
impl_supplier_debug_display!(BoxStatefulSupplier<T>);

impl<T> StatefulSupplier<T> for BoxStatefulSupplier<T> {
    #[inline(always)]
    fn get(&mut self) -> T {
        (self.function)()
    }
}
