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
//! * `$conditional_type` - The conditional consumer type for when (e.g., BoxConditionalConsumer)
//! * `$consumer_trait` - Consumer trait name (e.g., Consumer, BiConsumer)
//!
//! # Parameter Usage Comparison
//!
//! | Consumer Type | Struct Signature | `$conditional_type` | `$consumer_trait` |
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
/// * `$conditional_type` - The conditional consumer type for when (e.g., BoxConditionalConsumer)
/// * `$consumer_trait` - Consumer trait name (e.g., Consumer, BiConsumer)
///
/// # Parameter Usage Comparison
///
/// | Consumer Type | Struct Signature | `$conditional_type` | `$consumer_trait` |
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
    ($struct_name:ident < $t:ident >, $conditional_type:ident, $consumer_trait:ident) => {
        /// Creates a conditional consumer that executes based on predicate
        /// result.
        ///
        /// # Parameters
        ///
        /// * `predicate` - The predicate to determine whether to execute
        ///   the consumption operation
        ///
        /// # Returns
        ///
        /// Returns a conditional consumer that only executes when the
        /// predicate returns `true`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use std::sync::Arc;
        /// use std::sync::atomic::{AtomicI32, Ordering};
        /// use prism3_rust_function::consumers::*;
        ///
        /// let counter = Arc::new(AtomicI32::new(0));
        /// let consumer = BoxConsumer::new({
        ///     let counter = Arc::clone(&counter);
        ///     move |value: &i32| {
        ///         counter.fetch_add(*value, Ordering::SeqCst);
        ///     }
        /// });
        ///
        /// let conditional = consumer.when(|value: &i32| *value > 0);
        /// conditional.accept(&1);  // counter = 1
        /// conditional.accept(&-1); // not executed, counter remains 1
        /// ```
        pub fn when<P>(self, predicate: P) -> $conditional_type<$t>
        where
            P: Predicate<$t> + 'static,
        {
            $conditional_type {
                consumer: self,
                predicate: predicate.into_box(),
            }
        }

        /// Chains execution with another consumer, executing the current
        /// consumer first, then the subsequent consumer.
        ///
        /// # Parameters
        ///
        /// * `after` - The subsequent consumer to execute after the current
        ///   consumer completes
        ///
        /// # Returns
        ///
        /// Returns a new consumer that executes the current consumer and
        /// the subsequent consumer in sequence.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use std::sync::Arc;
        /// use std::sync::atomic::{AtomicI32, Ordering};
        /// use prism3_rust_function::consumers::*;
        ///
        /// let counter1 = Arc::new(AtomicI32::new(0));
        /// let counter2 = Arc::new(AtomicI32::new(0));
        ///
        /// let consumer1 = BoxConsumer::new({
        ///     let counter = Arc::clone(&counter1);
        ///     move |value: &i32| {
        ///         counter.fetch_add(*value, Ordering::SeqCst);
        ///     }
        /// });
        ///
        /// let consumer2 = BoxConsumer::new({
        ///     let counter = Arc::clone(&counter2);
        ///     move |value: &i32| {
        ///         counter.fetch_add(*value * 2, Ordering::SeqCst);
        ///     }
        /// });
        ///
        /// let chained = consumer1.and_then(consumer2);
        /// chained.accept(&5);
        /// // counter1 = 5, counter2 = 10
        /// ```
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
    ($struct_name:ident < $t:ident, $u:ident >, $conditional_type:ident, $consumer_trait:ident) => {
        /// Creates a conditional two-parameter consumer that executes based
        /// on bi-predicate result.
        ///
        /// # Parameters
        ///
        /// * `predicate` - The bi-predicate to determine whether to execute
        ///   the consumption operation
        ///
        /// # Returns
        ///
        /// Returns a conditional two-parameter consumer that only executes
        /// when the predicate returns `true`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use std::sync::Arc;
        /// use std::sync::atomic::{AtomicI32, Ordering};
        /// use prism3_rust_function::consumers::*;
        ///
        /// let counter = Arc::new(AtomicI32::new(0));
        /// let bi_consumer = BoxBiConsumer::new({
        ///     let counter = Arc::clone(&counter);
        ///     move |key: &String, value: &i32| {
        ///         if key == "increment" {
        ///             counter.fetch_add(*value, Ordering::SeqCst);
        ///         }
        ///     }
        /// });
        ///
        /// let conditional = bi_consumer.when(|key: &String, value: &i32|
        ///     *value > 0);
        /// conditional.accept(&"increment".to_string(), &5);  // counter = 5
        /// conditional.accept(&"increment".to_string(), &-2); // not executed
        /// ```
        pub fn when<P>(self, predicate: P) -> $conditional_type<$t, $u>
        where
            P: BiPredicate<$t, $u> + 'static,
        {
            $conditional_type {
                consumer: self,
                predicate: predicate.into_box(),
            }
        }

        /// Chains execution with another two-parameter consumer, executing
        /// the current consumer first, then the subsequent consumer.
        ///
        /// # Parameters
        ///
        /// * `after` - The subsequent two-parameter consumer to execute after
        ///   the current consumer completes
        ///
        /// # Returns
        ///
        /// Returns a new two-parameter consumer that executes the current
        /// consumer and the subsequent consumer in sequence.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use std::sync::Arc;
        /// use std::sync::atomic::{AtomicI32, Ordering};
        /// use prism3_rust_function::consumers::*;
        ///
        /// let counter1 = Arc::new(AtomicI32::new(0));
        /// let counter2 = Arc::new(AtomicI32::new(0));
        ///
        /// let bi_consumer1 = BoxBiConsumer::new({
        ///     let counter = Arc::clone(&counter1);
        ///     move |key: &String, value: &i32| {
        ///         counter.fetch_add(*value, Ordering::SeqCst);
        ///     }
        /// });
        ///
        /// let bi_consumer2 = BoxBiConsumer::new({
        ///     let counter = Arc::clone(&counter2);
        ///     move |key: &String, value: &i32| {
        ///         counter.fetch_add(*value * 2, Ordering::SeqCst);
        ///     }
        /// });
        ///
        /// let chained = bi_consumer1.and_then(bi_consumer2);
        /// chained.accept(&"test".to_string(), &3);
        /// // counter1 = 3, counter2 = 6
        /// ```
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
