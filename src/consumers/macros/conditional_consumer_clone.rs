/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Conditional Consumer Clone Macro
//!
//! Generates Clone trait implementation for Conditional Consumer types
//!
//! Generates Clone implementation for Conditional Consumer structs that have
//! `consumer` and `predicate` fields. Both fields are cloned using their
//! respective Clone implementations.
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
//! impl_conditional_consumer_clone!(ArcConditionalConsumer<T>);
//! impl_conditional_consumer_clone!(RcConditionalConsumer<T>);
//!
//! // For two type parameters
//! impl_conditional_consumer_clone!(ArcConditionalBiConsumer<T, U>);
//! impl_conditional_consumer_clone!(RcConditionalBiConsumer<T, U>);
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates Clone trait implementation for Conditional Consumer types
///
/// Generates Clone implementation for Conditional Consumer structs that have
/// `consumer` and `predicate` fields. Both fields are cloned using their
/// respective Clone implementations.
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list (one or two type parameters)
///
/// # Examples
///
/// ```ignore
/// // For single type parameter
/// impl_conditional_consumer_clone!(ArcConditionalConsumer<T>);
/// impl_conditional_consumer_clone!(RcConditionalConsumer<T>);
///
/// // For two type parameters
/// impl_conditional_consumer_clone!(ArcConditionalBiConsumer<T, U>);
/// impl_conditional_consumer_clone!(RcConditionalBiConsumer<T, U>);
/// ```
macro_rules! impl_conditional_consumer_clone {
    // Single generic parameter
    ($struct_name:ident < $generic:ident >) => {
        impl<$generic> Clone for $struct_name<$generic> {
            fn clone(&self) -> Self {
                Self {
                    consumer: self.consumer.clone(),
                    predicate: self.predicate.clone(),
                }
            }
        }
    };
    // Two generic parameters
    ($struct_name:ident < $generic1:ident, $generic2:ident >) => {
        impl<$generic1, $generic2> Clone for $struct_name<$generic1, $generic2> {
            fn clone(&self) -> Self {
                Self {
                    consumer: self.consumer.clone(),
                    predicate: self.predicate.clone(),
                }
            }
        }
    };
}

pub(crate) use impl_conditional_consumer_clone;
