/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Box Consumer Methods Macro
//!
//! Generates when and and_then method implementations for Box-based Consumer
//!
//! Generates conditional execution when method and chaining and_then method
//! for Box-based consumers that consume self (because Box cannot be cloned).
//!
//! This macro supports both single-parameter and two-parameter consumers through
//! pattern matching on the struct signature.
//!
//! # Parameters
//!
//! * `$struct_name<$generics>` - The struct name with its generic parameters
//!   - Single parameter: `BoxConsumer<T>`
//!   - Two parameters: `BoxBiConsumer<T, U>`
//! * `$return_type` - The return type for when (e.g., BoxConditionalConsumer)
//! * `$consumer_trait` - Consumer trait name (e.g., Consumer, BiConsumer)
//!
//! # Parameter Usage Comparison
//!
//! | Consumer Type | Struct Signature | `$return_type` | `$consumer_trait` |
//! |---------------|-----------------|----------------|------------------|
//! | **Consumer** | `BoxConsumer<T>` | BoxConditionalConsumer | Consumer |
//! | **ConsumerOnce** | `BoxConsumerOnce<T>` | BoxConditionalConsumerOnce | ConsumerOnce |
//! | **StatefulConsumer** | `BoxStatefulConsumer<T>` | BoxConditionalStatefulConsumer | StatefulConsumer |
//! | **BiConsumer** | `BoxBiConsumer<T, U>` | BoxConditionalBiConsumer | BiConsumer |
//! | **BiConsumerOnce** | `BoxBiConsumerOnce<T, U>` | BoxConditionalBiConsumerOnce | BiConsumerOnce |
//! | **StatefulBiConsumer** | `BoxStatefulBiConsumer<T, U>` | BoxConditionalStatefulBiConsumer | StatefulBiConsumer |
//!
//! # Examples
//!
//! ```ignore
//! // Single-parameter consumer
//! impl_box_consumer_methods!(
//!     BoxConsumer<T>,
//!     BoxConditionalConsumer,
//!     Consumer
//! );
//!
//! // Two-parameter consumer
//! impl_box_consumer_methods!(
//!     BoxBiConsumer<T, U>,
//!     BoxConditionalBiConsumer,
//!     BiConsumer
//! );
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates when and and_then method implementations for Box-based Consumer
///
/// Generates conditional execution when method and chaining and_then method
/// for Box-based consumers that consume self (because Box cannot be cloned).
///
/// This macro supports both single-parameter and two-parameter consumers through
/// pattern matching on the struct signature.
///
/// # Parameters
///
/// * `$struct_name<$generics>` - The struct name with its generic parameters
///   - Single parameter: `BoxConsumer<T>`
///   - Two parameters: `BoxBiConsumer<T, U>`
/// * `$return_type` - The return type for when (e.g., BoxConditionalConsumer)
/// * `$consumer_trait` - Consumer trait name (e.g., Consumer, BiConsumer)
///
/// # Parameter Usage Comparison
///
/// | Consumer Type | Struct Signature | `$return_type` | `$consumer_trait` |
/// |---------------|-----------------|----------------|------------------|
/// | **Consumer** | `BoxConsumer<T>` | BoxConditionalConsumer | Consumer |
/// | **ConsumerOnce** | `BoxConsumerOnce<T>` | BoxConditionalConsumerOnce | ConsumerOnce |
/// | **StatefulConsumer** | `BoxStatefulConsumer<T>` | BoxConditionalStatefulConsumer | StatefulConsumer |
/// | **BiConsumer** | `BoxBiConsumer<T, U>` | BoxConditionalBiConsumer | BiConsumer |
/// | **BiConsumerOnce** | `BoxBiConsumerOnce<T, U>` | BoxConditionalBiConsumerOnce | BiConsumerOnce |
/// | **StatefulBiConsumer** | `BoxStatefulBiConsumer<T, U>` | BoxConditionalStatefulBiConsumer | StatefulBiConsumer |
///
/// # Examples
///
/// ```ignore
/// // Single-parameter consumer
/// impl_box_consumer_methods!(
///     BoxConsumer<T>,
///     BoxConditionalConsumer,
///     Consumer
/// );
///
/// // Two-parameter consumer
/// impl_box_consumer_methods!(
///     BoxBiConsumer<T, U>,
///     BoxConditionalBiConsumer,
///     BiConsumer
/// );
/// ```
macro_rules! impl_box_consumer_methods {
    // Single generic parameter - Consumer
    ($struct_name:ident < $t:ident >, $return_type:ident, $consumer_trait:ident) => {
        pub fn when<P>(self, predicate: P) -> $return_type<$t>
        where
            P: Predicate<$t> + 'static,
        {
            $return_type {
                consumer: self,
                predicate: predicate.into_box(),
            }
        }

        #[allow(unused_mut)]
        pub fn and_then<C>(self, mut after: C) -> $struct_name<$t>
        where
            Self: Sized + 'static,
            $t: 'static,
            C: $consumer_trait<$t> + 'static,
        {
            let mut first = self;
            $struct_name::new(move |t: &$t| {
                first.accept(t);
                after.accept(t);
            })
        }
    };
    // Two generic parameters - BiConsumer
    ($struct_name:ident < $t:ident, $u:ident >, $return_type:ident, $consumer_trait:ident) => {
        pub fn when<P>(self, predicate: P) -> $return_type<$t, $u>
        where
            P: BiPredicate<$t, $u> + 'static,
        {
            $return_type {
                consumer: self,
                predicate: predicate.into_box(),
            }
        }

        #[allow(unused_mut)]
        pub fn and_then<C>(self, mut after: C) -> $struct_name<$t, $u>
        where
            Self: Sized + 'static,
            $t: 'static,
            $u: 'static,
            C: $consumer_trait<$t, $u> + 'static,
        {
            let mut first = self;
            $struct_name::new(move |t: &$t, u: &$u| {
                first.accept(t, u);
                after.accept(t, u);
            })
        }
    };
}

pub(crate) use impl_box_consumer_methods;
