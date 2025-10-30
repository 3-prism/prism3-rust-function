/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer Clone Macro
//!
//! Generates Clone trait implementation for basic Consumer types
//!
//! Generates Clone implementation for Consumer structs that have `function`
//! and `name` fields. The function field is cloned using its inherent `clone`
//! method, which performs a shallow clone for smart pointers like `Arc` or `Rc`.
//!
//! # Parameters
//!
//! * `$struct_name` - The struct name
//! * `$generic` - Generic parameter list (one or two type parameters)
//!
//! # Examples
//!
//! ```ignore
//! // For single type parameter
//! impl_consumer_clone!(ArcConsumer<T>);
//!
//! // For single type parameter with Rc
//! impl_consumer_clone!(RcConsumer<T>);
//!
//! // For two type parameters
//! impl_consumer_clone!(ArcBiConsumer<T, U>);
//!
//! // For two type parameters with Rc
//! impl_consumer_clone!(RcBiConsumer<T, U>);
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates Clone trait implementation for basic Consumer types
///
/// Generates Clone implementation for Consumer structs that have `function`
/// and `name` fields. The function field is cloned using its inherent `clone`
/// method, which performs a shallow clone for smart pointers like `Arc` or `Rc`.
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list (one or two type parameters)
///
/// # Examples
///
/// ```ignore
/// // For single type parameter with Arc
/// impl_consumer_clone!(ArcConsumer<T>);
///
/// // For single type parameter with Rc
/// impl_consumer_clone!(RcConsumer<T>);
///
/// // For two type parameters with Arc
/// impl_consumer_clone!(ArcBiConsumer<T, U>);
///
/// // For two type parameters with Rc
/// impl_consumer_clone!(RcBiConsumer<T, U>);
/// ```
macro_rules! impl_consumer_clone {
    // Single generic parameter
    ($struct_name:ident < $generic:ident >) => {
        impl<$generic> Clone for $struct_name<$generic> {
            fn clone(&self) -> Self {
                Self {
                    function: self.function.clone(),
                    name: self.name.clone(),
                }
            }
        }
    };
    // Two generic parameters
    ($struct_name:ident < $generic1:ident, $generic2:ident >) => {
        impl<$generic1, $generic2> Clone for $struct_name<$generic1, $generic2> {
            fn clone(&self) -> Self {
                Self {
                    function: self.function.clone(),
                    name: self.name.clone(),
                }
            }
        }
    };
}

pub(crate) use impl_consumer_clone;
