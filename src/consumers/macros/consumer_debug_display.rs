/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer Debug Display Macro
//!
//! Generates Debug and Display trait implementations for Consumer structs
//!
//! Generates standard Debug and Display trait implementations for Consumer
//! structs that have a `name: Option<String>` field.
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
//! impl_consumer_debug_display!(BoxConsumer<T>);
//!
//! // For two type parameters
//! impl_consumer_debug_display!(BoxBiConsumer<T, U>);
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates Debug and Display trait implementations for Consumer structs
///
/// Generates standard Debug and Display trait implementations for Consumer
/// structs that have a `name: Option<String>` field.
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
/// impl_consumer_debug_display!(BoxConsumer<T>);
///
/// // For two type parameters
/// impl_consumer_debug_display!(BoxBiConsumer<T, U>);
/// ```
macro_rules! impl_consumer_debug_display {
    // Single generic parameter
    ($struct_name:ident < $generic:ident >) => {
        impl<$generic> std::fmt::Debug for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("name", &self.name)
                    .field("function", &"<function>")
                    .finish()
            }
        }

        impl<$generic> std::fmt::Display for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.name {
                    Some(name) => write!(f, "{}({})", stringify!($struct_name), name),
                    None => write!(f, "{}", stringify!($struct_name)),
                }
            }
        }
    };
    // Two generic parameters
    ($struct_name:ident < $generic1:ident, $generic2:ident >) => {
        impl<$generic1, $generic2> std::fmt::Debug for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("name", &self.name)
                    .field("function", &"<function>")
                    .finish()
            }
        }

        impl<$generic1, $generic2> std::fmt::Display for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.name {
                    Some(name) => write!(f, "{}({})", stringify!($struct_name), name),
                    None => write!(f, "{}", stringify!($struct_name)),
                }
            }
        }
    };
}

pub(crate) use impl_consumer_debug_display;
