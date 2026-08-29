// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! # Supplier Common Methods Macro
//!
//! Generates common Supplier methods using shared implementations
//! (new, new_with_name, new_with_optional_name, name, set_name, constant)
//!
//! This macro uses `impl_common_new_methods` and `impl_common_name_methods`
//! to generate constructor and name management methods, plus a specialized
//! constant constructor for Supplier structs. This macro should be called
//! inside an impl block.
//!
//! The macro automatically detects the number of generic parameters and
//! generates the appropriate implementations for single-parameter or
//! two-parameter suppliers.
//!
//! # Parameters
//!
//! * `$struct_name<$generics>` - Struct name with generic parameters
//! * `$fn_trait_with_bounds` - Closure trait with complete bounds (e.g., `Fn()
//!   -> T + 'static`)
//! * `$wrapper_expr` - Wrapper expression (uses `f` for the closure)
//!
//! # Usage
//!
//! ```ignore
//! // Single generic parameter - Supplier
//! impl_supplier_common_methods!(
//!     BoxSupplier<T>,
//!     (Fn() -> T + 'static),
//!     |f| Box::new(f)
//! );
//!
//! // Single generic parameter - StatefulSupplier
//! impl_supplier_common_methods!(
//!     ArcStatefulSupplier<T>,
//!     (FnMut() -> T + Send + 'static),
//!     |f| Arc::new(Mutex::new(f))
//! );
//! ```
//!
//! # Generated Methods
//!
//! * `new()` - Creates a new supplier
//! * `new_with_name()` - Creates a named supplier
//! * `new_with_optional_name()` - Creates a supplier with optional name
//! * `name()` - Gets the name of the supplier
//! * `set_name()` - Sets the name of the supplier
//! * `constant()` - Creates a supplier that returns a constant value

/// Generates common Supplier methods using shared implementations
/// (new, new_with_name, new_with_optional_name, name, set_name, constant)
///
/// This macro uses `impl_common_new_methods` and `impl_common_name_methods`
/// to generate constructor and name management methods, plus a specialized
/// constant constructor for Supplier structs. This macro should be used
/// inside an existing impl block for the target struct.
///
/// The macro automatically detects the number of generic parameters and
/// generates the appropriate implementations for single-parameter or
/// two-parameter suppliers.
///
/// # Parameters
///
/// * `$struct_name<$generics>` - Struct name with generic parameters
/// * `$fn_trait_with_bounds` - Closure trait with complete bounds (e.g., `Fn()
///   -> T + 'static`)
/// * `$wrapper_expr` - Wrapper expression (uses `f` for the closure)
///
/// # Usage
///
/// ```ignore
/// // Single generic parameter - Supplier
/// impl_supplier_common_methods!(
///     BoxSupplier<T>,
///     (Fn() -> T + 'static),
///     |f| Box::new(f)
/// );
///
/// // Single generic parameter - StatefulSupplier
/// impl_supplier_common_methods!(
///     ArcStatefulSupplier<T>,
///     (FnMut() -> T + Send + 'static),
///     |f| Arc::new(Mutex::new(f))
/// );
/// ```
///
/// # Generated Methods
///
/// * `new()` - Creates a new supplier
/// * `new_with_name()` - Creates a named supplier
/// * `new_with_optional_name()` - Creates a supplier with optional name
/// * `name()` - Gets the name of the supplier
/// * `set_name()` - Sets the name of the supplier
/// * `constant()` - Creates a supplier that returns a constant value
macro_rules! impl_supplier_new_methods {
    (BoxSupplier<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::suppliers::macros::impl_supplier_new_methods!(@shared Supplier, $t, ('static), |$f| $wrapper); };
    (RcSupplier<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::suppliers::macros::impl_supplier_new_methods!(@shared Supplier, $t, ('static), |$f| $wrapper); };
    (BoxStatefulSupplier<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::suppliers::macros::impl_supplier_new_methods!(@stateful StatefulSupplier, $t, ('static), |$f| $wrapper); };
    (RcStatefulSupplier<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::suppliers::macros::impl_supplier_new_methods!(@stateful StatefulSupplier, $t, ('static), |$f| $wrapper); };
    (BoxSupplierOnce<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::suppliers::macros::impl_supplier_new_methods!(@once SupplierOnce, $t, ('static), |$f| $wrapper); };
    (@shared $trait:ident, $t:ident, ($($bounds:tt)+), |$f:ident| $wrapper:expr) => { crate::macros::impl_common_new_methods!(semantic ($trait<$t> + $($bounds)+), |source| move || source.get(), |$f| $wrapper, "supplier"); };
    (@stateful $trait:ident, $t:ident, ($($bounds:tt)+), |$f:ident| $wrapper:expr) => { crate::macros::impl_common_new_methods!(semantic_mut ($trait<$t> + $($bounds)+), |source| move || source.get(), |$f| $wrapper, "supplier"); };
    (@once $trait:ident, $t:ident, ($($bounds:tt)+), |$f:ident| $wrapper:expr) => { crate::macros::impl_common_new_methods!(semantic ($trait<$t> + $($bounds)+), |source| move || source.get(), |$f| $wrapper, "supplier"); };
}

macro_rules! impl_supplier_common_methods {
    // Single generic parameter - Supplier types
    (
        $struct_name:ident < $t:ident >,
        ($($fn_trait_with_bounds:tt)+),
        |$f:ident| $wrapper_expr:expr
    ) => {
        $crate::suppliers::macros::impl_supplier_new_methods!($struct_name<$t>, |$f| $wrapper_expr);

        /// Creates a supplier that returns a constant value.
        ///
        /// Creates a supplier that always returns the same value. Useful for
        /// default values or placeholder implementations.
        ///
        /// # Parameters
        ///
        /// * `value` - The constant value to return
        ///
        /// # Returns
        ///
        /// Returns a new supplier instance that returns the constant value.
        #[inline]
        pub fn constant(value: $t) -> Self
        where
            $t: Clone + 'static,
        {
            Self::new(move || value.clone())
        }

        crate::macros::impl_common_name_methods!("supplier");
    };
}

pub(crate) use impl_supplier_common_methods;
pub(crate) use impl_supplier_new_methods;
