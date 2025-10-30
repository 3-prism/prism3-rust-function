/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Conditional Consumer Debug Display Macro
//!
//! Generates Debug and Display trait implementations for Conditional Consumer structs
//!
//! Generates standard Debug and Display trait implementations for Conditional
//! Consumer structs that have `consumer` and `predicate` fields but no `name` field.
//!
//! # Parameters
//!
//! * `$struct_name` - The struct name
//! * `$generic` - Generic parameter list (one or more type parameters)
//!
//! # Examples
//!
//! ```ignore
//! // For single type parameter
//! impl_conditional_consumer_debug_display!(BoxConditionalConsumer<T>);
//!
//! // For two type parameters
//! impl_conditional_consumer_debug_display!(BoxConditionalBiConsumer<T, U>);
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates Debug and Display trait implementations for Conditional Consumer structs
///
/// Generates standard Debug and Display trait implementations for Conditional
/// Consumer structs that have `consumer` and `predicate` fields but no `name` field.
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list (one or more type parameters)
///
/// # Examples
///
/// ```ignore
/// // For single type parameter
/// impl_conditional_consumer_debug_display!(BoxConditionalConsumer<T>);
///
/// // For two type parameters
/// impl_conditional_consumer_debug_display!(BoxConditionalBiConsumer<T, U>);
/// ```
macro_rules! impl_conditional_consumer_debug_display {
    // Single generic parameter
    ($struct_name:ident < $generic:ident >) => {
        impl<$generic> std::fmt::Debug for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("consumer", &self.consumer)
                    .field("predicate", &self.predicate)
                    .finish()
            }
        }

        impl<$generic> std::fmt::Display for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}({}, {})",
                    stringify!($struct_name),
                    self.consumer,
                    self.predicate
                )
            }
        }
    };
    // Two generic parameters
    ($struct_name:ident < $generic1:ident, $generic2:ident >) => {
        impl<$generic1, $generic2> std::fmt::Debug for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("consumer", &self.consumer)
                    .field("predicate", &self.predicate)
                    .finish()
            }
        }

        impl<$generic1, $generic2> std::fmt::Display for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}({}, {})",
                    stringify!($struct_name),
                    self.consumer,
                    self.predicate
                )
            }
        }
    };
}

pub(crate) use impl_conditional_consumer_debug_display;
