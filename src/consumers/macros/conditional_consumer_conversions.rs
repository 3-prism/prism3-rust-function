/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Conditional Consumer Conversions Macro
//!
//! Generates conversion methods for Conditional Consumer implementations
//!
//! This macro generates the conversion methods (`into_box`, `into_rc`, `into_fn`) for
//! conditional consumer types. It handles both immutable (Consumer) and mutable
//! (StatefulConsumer) cases using the `#[allow(unused_mut)]` annotation.
//!
//! The macro works by always declaring variables as `mut`, which is necessary for
//! StatefulConsumer cases, while suppressing unused_mut warnings for Consumer cases
//! where the mutability is not needed.
//!
//! # Parameters
//!
//! * `$box_type<$t:ident>` - The box-based consumer type (e.g., `BoxConsumer<T>`)
//! * `$rc_type:ident` - The rc-based consumer type name (e.g., `RcConsumer`)
//! * `$fn_trait:ident` - The function trait (e.g., `Fn` or `FnMut`)
//!
//! # Usage Examples
//!
//! For Consumer (immutable):
//! ```ignore
//! impl<T> Consumer<T> for BoxConditionalConsumer<T>
//! where
//!     T: 'static,
//! {
//!     fn accept(&self, value: &T) {
//!         if self.predicate.test(value) {
//!             self.consumer.accept(value);
//!         }
//!     }
//!
//!     impl_conditional_consumer_conversions!(
//!         BoxConsumer<T>,
//!         RcConsumer,
//!         Fn
//!     );
//! }
//! ```
//!
//! For StatefulConsumer (mutable):
//! ```ignore
//! impl<T> StatefulConsumer<T> for BoxConditionalStatefulConsumer<T>
//! where
//!     T: 'static,
//! {
//!     fn accept(&mut self, value: &T) {
//!         if self.predicate.test(value) {
//!             self.consumer.accept(value);
//!         }
//!     }
//!
//!     impl_conditional_consumer_conversions!(
//!         BoxStatefulConsumer<T>,
//!         RcStatefulConsumer,
//!         FnMut
//!     );
//! }
//! ```
//!
//! # Implementation Details
//!
//! - Uses `#[allow(unused_mut)]` to handle Consumer cases where `mut` is not needed
//! - The closures inside `into_box` and `into_rc` will automatically capture as `Fn`
//!   or `FnMut` based on their internal operations
//! - The `into_fn` method uses the provided `$fn_trait` parameter to match the
//!   intended trait type
//!
//! # Author
//!
//! Haixing Hu

/// Generates conversion methods for Conditional Consumer implementations
///
/// This macro generates the conversion methods (`into_box`, `into_rc`, `into_fn`) for
/// conditional consumer types. It handles both immutable (Consumer) and mutable
/// (StatefulConsumer) cases using the `#[allow(unused_mut)]` annotation.
///
/// The macro works by always declaring variables as `mut`, which is necessary for
/// StatefulConsumer cases, while suppressing unused_mut warnings for Consumer cases
/// where the mutability is not needed.
///
/// # Parameters
///
/// * `$box_type<$t:ident>` - The box-based consumer type (e.g., `BoxConsumer<T>`)
/// * `$rc_type:ident` - The rc-based consumer type name (e.g., `RcConsumer`)
/// * `$fn_trait:ident` - The function trait (e.g., `Fn` or `FnMut`)
///
/// # Usage Examples
///
/// For Consumer (immutable):
/// ```ignore
/// impl<T> Consumer<T> for BoxConditionalConsumer<T>
/// where
///     T: 'static,
/// {
///     fn accept(&self, value: &T) {
///         if self.predicate.test(value) {
///             self.consumer.accept(value);
///         }
///     }
///
///     impl_conditional_consumer_conversions!(
///         BoxConsumer<T>,
///         RcConsumer,
///         Fn
///     );
/// }
/// ```
///
/// For StatefulConsumer (mutable):
/// ```ignore
/// impl<T> StatefulConsumer<T> for BoxConditionalStatefulConsumer<T>
/// where
///     T: 'static,
/// {
///     fn accept(&mut self, value: &T) {
///         if self.predicate.test(value) {
///             self.consumer.accept(value);
///         }
///     }
///
///     impl_conditional_consumer_conversions!(
///         BoxStatefulConsumer<T>,
///         RcStatefulConsumer,
///         FnMut
///     );
/// }
/// ```
///
/// # Implementation Details
///
/// - Uses `#[allow(unused_mut)]` to handle Consumer cases where `mut` is not needed
/// - The closures inside `into_box` and `into_rc` will automatically capture as `Fn`
///   or `FnMut` based on their internal operations
/// - The `into_fn` method uses the provided `$fn_trait` parameter to match the
///   intended trait type
macro_rules! impl_conditional_consumer_conversions {
    // Single generic parameter - Consumer
    (
        $box_type:ident < $t:ident >,
        $rc_type:ident,
        $fn_trait:ident
    ) => {
        #[allow(unused_mut)]
        fn into_box(self) -> $box_type<$t> {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            $box_type::new(move |t| {
                if pred.test(t) {
                    consumer.accept(t);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_rc(self) -> $rc_type<$t> {
            let pred = self.predicate.into_rc();
            let mut consumer = self.consumer.into_rc();
            let mut consumer_fn = consumer;
            $rc_type::new(move |t| {
                if pred.test(t) {
                    consumer_fn.accept(t);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_fn(self) -> impl $fn_trait(&$t) {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            move |t: &$t| {
                if pred.test(t) {
                    consumer.accept(t);
                }
            }
        }
    };

    // Two generic parameters - BiConsumer
    (
        $box_type:ident < $t:ident, $u:ident >,
        $rc_type:ident,
        $fn_trait:ident
    ) => {
        #[allow(unused_mut)]
        fn into_box(self) -> $box_type<$t, $u> {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            $box_type::new(move |t, u| {
                if pred.test(t, u) {
                    consumer.accept(t, u);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_rc(self) -> $rc_type<$t, $u> {
            let pred = self.predicate.into_rc();
            let mut consumer = self.consumer.into_rc();
            let mut consumer_fn = consumer;
            $rc_type::new(move |t, u| {
                if pred.test(t, u) {
                    consumer_fn.accept(t, u);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_fn(self) -> impl $fn_trait(&$t, &$u) {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            move |t: &$t, u: &$u| {
                if pred.test(t, u) {
                    consumer.accept(t, u);
                }
            }
        }
    };
}

pub(crate) use impl_conditional_consumer_conversions;
