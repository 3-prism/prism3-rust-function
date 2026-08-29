// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Mutator Common Methods Macro
//!
//! Generates common Mutator methods (new, new_with_name, name,
//! set_name, noop)
//!
//! Generates constructor methods, name management methods and noop
//! constructor for Mutator structs. This macro should be called inside
//! an impl block.
//!
//! The macro automatically detects the number of generic parameters and
//! generates the appropriate implementations for single-parameter mutators.
//!
//! # Parameters
//!
//! * `$struct_name<$generics>` - Struct name with generic parameters
//! * `$fn_trait_with_bounds` - Closure trait with complete bounds (e.g.,
//!   `FnMut(&mut T) + 'static`)
//! * `$wrapper_expr` - Wrapper expression (uses `f` for the closure)
//!
//! # Usage
//!
//! ```ignore
//! // Single generic parameter - Mutator
//! impl_mutator_common_methods!(
//!     BoxMutator<T>,
//!     (FnMut(&mut T) + 'static),
//!     |f| Box::new(f)
//! );
//!
//! // Single generic parameter - StatefulMutator
//! impl_mutator_common_methods!(
//!     ArcStatefulMutator<T>,
//!     (FnMut(&mut T) + Send + 'static),
//!     |f| Arc::new(Mutex::new(f))
//! );
//! ```
//!
//! # Generated Methods
//!
//! * `new()` - Creates a new mutator
//! * `new_with_name()` - Creates a named mutator
//! * `name()` - Gets the name of the mutator
//! * `set_name()` - Sets the name of the mutator
//! * `noop()` - Creates a mutator that performs no operation

/// Generates common Mutator methods (new, new_with_name, name,
/// set_name, noop)
///
/// This macro should be used inside an impl block to generate constructor
/// methods, name management methods and noop constructor for Mutator structs.
/// The macro automatically detects the number of generic parameters and
/// generates the appropriate implementations for single-parameter mutators.
///
/// # Parameters
///
/// * `$struct_name<$generics>` - Struct name with generic parameters
/// * `$fn_trait_with_bounds` - Closure trait with complete bounds (e.g.,
///   `FnMut(&mut T) + 'static`)
/// * `$wrapper_expr` - Wrapper expression (uses `f` for the closure)
///
/// # Usage Location
///
/// This macro should be used inside an impl block for the struct type.
///
/// # Usage
///
/// ```ignore
/// impl<T> BoxMutator<T> {
///     // Inside an impl block
///     impl_mutator_common_methods!(
///         BoxMutator<T>,
///         (FnMut(&mut T) + 'static),
///         |f| Box::new(f)
///     );
/// }
///
/// impl<T> ArcStatefulMutator<T> {
///     // Inside an impl block for StatefulMutator
///     impl_mutator_common_methods!(
///         ArcStatefulMutator<T>,
///         (FnMut(&mut T) + Send + 'static),
///         |f| Arc::new(Mutex::new(f))
///     );
/// }
/// ```
///
/// # Generated Methods
///
/// * `new()` - Creates a new mutator
/// * `new_with_name()` - Creates a named mutator
/// * `name()` - Gets the name of the mutator
/// * `set_name()` - Sets the name of the mutator
/// * `noop()` - Creates a mutator that performs no operation
macro_rules! impl_mutator_new_methods {
    (BoxMutator<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::mutators::macros::impl_mutator_new_methods!(@shared Mutator, $t, ('static), |$f| $wrapper); };
    (RcMutator<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::mutators::macros::impl_mutator_new_methods!(@shared Mutator, $t, ('static), |$f| $wrapper); };
    (ArcMutator<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::mutators::macros::impl_mutator_new_methods!(@shared Mutator, $t, (Send + Sync + 'static), |$f| $wrapper); };
    (BoxStatefulMutator<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::mutators::macros::impl_mutator_new_methods!(@stateful StatefulMutator, $t, ('static), |$f| $wrapper); };
    (RcStatefulMutator<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::mutators::macros::impl_mutator_new_methods!(@stateful StatefulMutator, $t, ('static), |$f| $wrapper); };
    (ArcStatefulMutator<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::mutators::macros::impl_mutator_new_methods!(@stateful StatefulMutator, $t, (Send + 'static), |$f| $wrapper); };
    (BoxMutatorOnce<$t:ident>, |$f:ident| $wrapper:expr) => { $crate::mutators::macros::impl_mutator_new_methods!(@once MutatorOnce, $t, ('static), |$f| $wrapper); };
    (@shared $trait:ident, $t:ident, ($($bounds:tt)+), |$f:ident| $wrapper:expr) => { crate::macros::impl_common_new_methods!(semantic ($trait<$t> + $($bounds)+), |source| move |value: &mut $t| source.apply(value), |$f| $wrapper, "mutator"); };
    (@stateful $trait:ident, $t:ident, ($($bounds:tt)+), |$f:ident| $wrapper:expr) => { crate::macros::impl_common_new_methods!(semantic_mut ($trait<$t> + $($bounds)+), |source| move |value: &mut $t| source.apply(value), |$f| $wrapper, "mutator"); };
    (@once $trait:ident, $t:ident, ($($bounds:tt)+), |$f:ident| $wrapper:expr) => { crate::macros::impl_common_new_methods!(semantic ($trait<$t> + $($bounds)+), |source| move |value: &mut $t| source.apply(value), |$f| $wrapper, "mutator"); };
}

macro_rules! impl_mutator_common_methods {
    // Single generic parameter - Mutator types
    (
        $struct_name:ident < $t:ident >,
        ($($fn_trait_with_bounds:tt)+),
        |$f:ident| $wrapper_expr:expr
    ) => {
        $crate::mutators::macros::impl_mutator_new_methods!($struct_name<$t>, |$f| $wrapper_expr);
        /// Creates a no-operation mutator.
        ///
        /// Creates a mutator that does nothing when called. Useful for
        /// default values or placeholder implementations.
        ///
        /// # Returns
        ///
        /// Returns a new mutator instance that performs no operation.
        #[inline]
        pub fn noop() -> Self {
            Self::new(|_: &mut $t| {})
        }

        crate::macros::impl_common_name_methods!("mutator");
    };
}

pub(crate) use impl_mutator_common_methods;
pub(crate) use impl_mutator_new_methods;
