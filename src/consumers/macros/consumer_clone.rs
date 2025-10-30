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
//! and `name` fields. The function field is cloned using the specified
//! smart pointer clone method (Arc::clone or Rc::clone).
//!
//! # Parameters
//!
//! * `$struct_name` - The struct name
//! * `$generic` - Generic parameter list (one or two type parameters)
//! * `$clone_method` - Smart pointer clone method (Arc::clone or Rc::clone)
//!
//! # Examples
//!
//! ```ignore
//! // For single type parameter with Arc
//! impl_consumer_clone!(ArcConsumer<T>, Arc::clone);
//!
//! // For single type parameter with Rc
//! impl_consumer_clone!(RcConsumer<T>, Rc::clone);
//!
//! // For two type parameters with Arc
//! impl_consumer_clone!(ArcBiConsumer<T, U>, Arc::clone);
//!
//! // For two type parameters with Rc
//! impl_consumer_clone!(RcBiConsumer<T, U>, Rc::clone);
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates Clone trait implementation for basic Consumer types
///
/// Generates Clone implementation for Consumer structs that have `function`
/// and `name` fields. The function field is cloned using the specified
/// smart pointer clone method (Arc::clone or Rc::clone).
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list (one or two type parameters)
/// * `$clone_method` - Smart pointer clone method (Arc::clone or Rc::clone)
///
/// # Examples
///
/// ```ignore
/// // For single type parameter with Arc
/// impl_consumer_clone!(ArcConsumer<T>, Arc::clone);
///
/// // For single type parameter with Rc
/// impl_consumer_clone!(RcConsumer<T>, Rc::clone);
///
/// // For two type parameters with Arc
/// impl_consumer_clone!(ArcBiConsumer<T, U>, Arc::clone);
///
/// // For two type parameters with Rc
/// impl_consumer_clone!(RcBiConsumer<T, U>, Rc::clone);
/// ```
macro_rules! impl_consumer_clone {
    // Single generic parameter
    ($struct_name:ident < $generic:ident >, $clone_method:path) => {
        impl<$generic> Clone for $struct_name<$generic> {
            fn clone(&self) -> Self {
                Self {
                    function: $clone_method(&self.function),
                    name: self.name.clone(),
                }
            }
        }
    };
    // Two generic parameters
    ($struct_name:ident < $generic1:ident, $generic2:ident >, $clone_method:path) => {
        impl<$generic1, $generic2> Clone for $struct_name<$generic1, $generic2> {
            fn clone(&self) -> Self {
                Self {
                    function: $clone_method(&self.function),
                    name: self.name.clone(),
                }
            }
        }
    };
}

pub(crate) use impl_consumer_clone;
