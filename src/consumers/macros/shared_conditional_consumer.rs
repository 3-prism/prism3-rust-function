/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Shared Conditional Consumer Macro
//!
//! Generates Arc/Rc-based Conditional Consumer implementations
//!
//! For Arc/Rc-based conditional consumers, generates `and_then` and `or_else` methods,
//! as well as complete Consumer/BiConsumer trait implementations.
//!
//! Arc/Rc type characteristics:
//! - `and_then` and `or_else` borrow &self (because Arc/Rc can Clone)
//! - Uses trait default implementations for `into_arc()` and `to_arc()`
//! - Arc types will work with `into_arc()` and `to_arc()` (satisfy Send + Sync constraints)
//! - Rc types will get compile errors if trying to use `into_arc()` or `to_arc()` (don't satisfy Send + Sync)
//! - Implement complete `to_xxx()` methods (because they can Clone)
//!
//! # Parameters
//!
//! * `$struct_name<$generics>` - Struct name with generic parameters
//! * `$consumer_type` - Consumer wrapper type name
//! * `$consumer_trait` - Consumer trait name
//! * `$predicate_conversion` - Predicate conversion method (into_arc or into_rc)
//! * `$extra_bounds` - Extra trait bounds
//!
//! # Usage Examples
//!
//! ```ignore
//! // Arc single-parameter Consumer
//! impl_shared_conditional_consumer!(
//!     ArcConditionalConsumer<T>,
//!     ArcConsumer,
//!     Consumer,
//!     into_arc,
//!     Send + Sync + 'static
//! );
//!
//! // Rc single-parameter Consumer
//! impl_shared_conditional_consumer!(
//!     RcConditionalConsumer<T>,
//!     RcConsumer,
//!     Consumer,
//!     into_rc,
//!     'static
//! );
//!
//! // Arc two-parameter BiConsumer
//! impl_shared_conditional_consumer!(
//!     ArcConditionalBiConsumer<T, U>,
//!     ArcBiConsumer,
//!     BiConsumer,
//!     into_arc,
//!     Send + Sync + 'static
//! );
//!
//! // Rc two-parameter BiConsumer
//! impl_shared_conditional_consumer!(
//!     RcConditionalBiConsumer<T, U>,
//!     RcBiConsumer,
//!     BiConsumer,
//!     into_rc,
//!     'static
//! );
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates Arc/Rc-based Conditional Consumer implementations
///
/// For Arc/Rc-based conditional consumers, generates `and_then` and `or_else` methods,
/// as well as complete Consumer/BiConsumer trait implementations.
///
/// Arc/Rc type characteristics:
/// - `and_then` and `or_else` borrow &self (because Arc/Rc can Clone)
/// - Uses trait default implementations for `into_arc()` and `to_arc()`
/// - Arc types will work with `into_arc()` and `to_arc()` (satisfy Send + Sync constraints)
/// - Rc types will get compile errors if trying to use `into_arc()` or `to_arc()` (don't satisfy Send + Sync)
/// - Implement complete `to_xxx()` methods (because they can Clone)
///
/// # Parameters
///
/// * `$struct_name<$generics>` - Struct name with generic parameters
/// * `$consumer_type` - Consumer wrapper type name
/// * `$consumer_trait` - Consumer trait name
/// * `$predicate_conversion` - Predicate conversion method (into_arc or into_rc)
/// * `$extra_bounds` - Extra trait bounds
///
/// # Usage Examples
///
/// ```ignore
/// // Arc single-parameter Consumer
/// impl_shared_conditional_consumer!(
///     ArcConditionalConsumer<T>,
///     ArcConsumer,
///     Consumer,
///     into_arc,
///     Send + Sync + 'static
/// );
///
/// // Rc single-parameter Consumer
/// impl_shared_conditional_consumer!(
///     RcConditionalConsumer<T>,
///     RcConsumer,
///     Consumer,
///     into_rc,
///     'static
/// );
///
/// // Arc two-parameter BiConsumer
/// impl_shared_conditional_consumer!(
///     ArcConditionalBiConsumer<T, U>,
///     ArcBiConsumer,
///     BiConsumer,
///     into_arc,
///     Send + Sync + 'static
/// );
///
/// // Rc two-parameter BiConsumer
/// impl_shared_conditional_consumer!(
///     RcConditionalBiConsumer<T, U>,
///     RcBiConsumer,
///     BiConsumer,
///     into_rc,
///     'static
/// );
/// ```
macro_rules! impl_shared_conditional_consumer {
    // Single generic parameter - Consumer
    (
        $struct_name:ident < $t:ident >,
        $consumer_type:ident,
        $consumer_trait:ident,
        $predicate_conversion:ident,
        $($extra_bounds:tt)+
    ) => {
        impl<$t> $struct_name<$t>
        where
            $t: 'static,
        {
            /// Chains another consumer in sequence
            ///
            /// Combines the current conditional consumer with another consumer
            /// into a new consumer that implements the following semantics:
            ///
            /// When the returned consumer is called with an argument:
            /// 1. First, it checks the predicate of this conditional consumer
            /// 2. If the predicate is satisfied, it executes the internal
            ///    consumer of this conditional consumer
            /// 3. Then, **regardless of whether the predicate was satisfied**,
            ///    it unconditionally executes the `next` consumer
            ///
            /// In other words, this creates a consumer that conditionally
            /// executes the first action (based on the predicate), and then
            /// always executes the second action.
            ///
            /// # Parameters
            ///
            /// * `next` - The next consumer to execute (always executed)
            ///
            /// # Returns
            ///
            /// Returns a new combined consumer
            ///
            /// # Examples
            ///
            /// ```ignore
            /// use std::sync::atomic::{AtomicI32, Ordering};
            ///
            /// let result = AtomicI32::new(0);
            ///
            /// let consumer1 = ArcConsumer::new(|x: &i32| {
            ///     result.fetch_add(*x, Ordering::SeqCst);
            /// });
            ///
            /// let consumer2 = ArcConsumer::new(|x: &i32| {
            ///     result.fetch_add(2 * (*x), Ordering::SeqCst);
            /// });
            ///
            /// let conditional = consumer1.when(|x| *x > 0);
            /// let chained = conditional.and_then(consumer2);
            ///
            /// chained.accept(&5);  // result = 5 + (2*5) = 15
            /// result.store(0, Ordering::SeqCst);  // reset
            /// chained.accept(&-5); // result = 0 + (2*-5) = -10 (not -15!)
            /// ```
            #[allow(unused_mut)]
            pub fn and_then<C>(&self, mut next: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + $($extra_bounds)+,
            {
                let first_predicate = self.predicate.clone();
                let mut first_consumer = self.consumer.clone();
                $consumer_type::new(move |t| {
                    if first_predicate.test(t) {
                        first_consumer.accept(t);
                    }
                    next.accept(t);
                })
            }

            /// Adds an else branch
            ///
            /// Executes the original consumer when the condition is satisfied, otherwise
            /// executes else_consumer.
            ///
            /// # Parameters
            ///
            /// * `else_consumer` - The consumer for the else branch
            ///
            /// # Returns
            ///
            /// Returns a new consumer with if-then-else logic
            #[allow(unused_mut)]
            pub fn or_else<C>(&self, mut else_consumer: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + $($extra_bounds)+,
            {
                let predicate = self.predicate.clone();
                let mut then_consumer = self.consumer.clone();
                $consumer_type::new(move |t| {
                    if predicate.test(t) {
                        then_consumer.accept(t);
                    } else {
                        else_consumer.accept(t);
                    }
                })
            }
        }
    };

    // Two generic parameters - BiConsumer
    (
        $struct_name:ident < $t:ident, $u:ident >,
        $consumer_type:ident,
        $consumer_trait:ident,
        $predicate_conversion:ident,
        $($extra_bounds:tt)+
    ) => {
        impl<$t, $u> $struct_name<$t, $u>
        where
            $t: 'static,
            $u: 'static,
        {
            /// Chains another bi-consumer in sequence
            ///
            /// Combines the current conditional bi-consumer with another
            /// bi-consumer into a new bi-consumer that implements the
            /// following semantics:
            ///
            /// When the returned bi-consumer is called with two arguments:
            /// 1. First, it checks the predicate of this conditional
            ///    bi-consumer
            /// 2. If the predicate is satisfied, it executes the internal
            ///    bi-consumer of this conditional bi-consumer
            /// 3. Then, **regardless of whether the predicate was
            ///    satisfied**, it unconditionally executes the `next`
            ///    bi-consumer
            ///
            /// In other words, this creates a bi-consumer that conditionally
            /// executes the first action (based on the predicate), and then
            /// always executes the second action.
            ///
            /// # Parameters
            ///
            /// * `next` - The next bi-consumer to execute (always executed)
            ///
            /// # Returns
            ///
            /// Returns a new combined bi-consumer
            ///
            /// # Examples
            ///
            /// ```ignore
            /// use std::sync::atomic::{AtomicI32, Ordering};
            ///
            /// let result = AtomicI32::new(0);
            ///
            /// let consumer1 = ArcBiConsumer::new(|x: &i32, y: &i32| {
            ///     result.fetch_add(x + y, Ordering::SeqCst);
            /// });
            ///
            /// let consumer2 = ArcBiConsumer::new(|x: &i32, y: &i32| {
            ///     result.fetch_add(2 * (x + y), Ordering::SeqCst);
            /// });
            ///
            /// let conditional = consumer1.when(|x, y| *x > 0 && *y > 0);
            /// let chained = conditional.and_then(consumer2);
            ///
            /// chained.accept(&5, &3);  // result = (5+3) + 2*(5+3) = 24
            /// result.store(0, Ordering::SeqCst);  // reset
            /// chained.accept(&-5, &3); // result = 0 + 2*(-5+3) = -4 (not -8!)
            /// ```
            #[allow(unused_mut)]
            pub fn and_then<C>(&self, mut next: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + $($extra_bounds)+,
            {
                let first_predicate = self.predicate.clone();
                let mut first_consumer = self.consumer.clone();
                $consumer_type::new(move |t, u| {
                    if first_predicate.test(t, u) {
                        first_consumer.accept(t, u);
                    }
                    next.accept(t, u);
                })
            }

            /// Adds an else branch
            ///
            /// Executes the original bi-consumer when the condition is satisfied, otherwise
            /// executes else_consumer.
            ///
            /// # Parameters
            ///
            /// * `else_consumer` - The bi-consumer for the else branch
            ///
            /// # Returns
            ///
            /// Returns a new bi-consumer with if-then-else logic
            #[allow(unused_mut)]
            pub fn or_else<C>(&self, mut else_consumer: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + $($extra_bounds)+,
            {
                let predicate = self.predicate.clone();
                let mut then_consumer = self.consumer.clone();
                $consumer_type::new(move |t, u| {
                    if predicate.test(t, u) {
                        then_consumer.accept(t, u);
                    } else {
                        else_consumer.accept(t, u);
                    }
                })
            }
        }
    };
}

pub(crate) use impl_shared_conditional_consumer;
