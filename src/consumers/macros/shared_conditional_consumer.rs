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
            /// Combines the current conditional consumer with another consumer into a new
            /// consumer. The current conditional consumer executes first, followed by the
            /// next consumer.
            ///
            /// # Parameters
            ///
            /// * `next` - The next consumer to execute
            ///
            /// # Returns
            ///
            /// Returns a new combined consumer
            #[allow(unused_mut)]
            pub fn and_then<C>(&self, next: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + $($extra_bounds)+,
            {
                let mut first = self.consumer.clone();
                let mut second = next;
                $consumer_type::new(move |t| {
                    first.accept(t);
                    second.accept(t);
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
            pub fn or_else<C>(&self, else_consumer: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + $($extra_bounds)+,
            {
                let pred = self.predicate.clone();
                let mut then_cons = self.consumer.clone();
                let mut else_cons = else_consumer;
                $consumer_type::new(move |t| {
                    if pred.test(t) {
                        then_cons.accept(t);
                    } else {
                        else_cons.accept(t);
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
            /// Combines the current conditional bi-consumer with another bi-consumer into a new
            /// bi-consumer. The current conditional bi-consumer executes first, followed by the
            /// next bi-consumer.
            ///
            /// # Parameters
            ///
            /// * `next` - The next bi-consumer to execute
            ///
            /// # Returns
            ///
            /// Returns a new combined bi-consumer
            #[allow(unused_mut)]
            pub fn and_then<C>(&self, next: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + $($extra_bounds)+,
            {
                let mut first = self.consumer.clone();
                let mut second = next;
                $consumer_type::new(move |t, u| {
                    first.accept(t, u);
                    second.accept(t, u);
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
            pub fn or_else<C>(&self, else_consumer: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + $($extra_bounds)+,
            {
                let pred = self.predicate.clone();
                let mut then_cons = self.consumer.clone();
                let mut else_cons = else_consumer;
                $consumer_type::new(move |t, u| {
                    if pred.test(t, u) {
                        then_cons.accept(t, u);
                    } else {
                        else_cons.accept(t, u);
                    }
                })
            }
        }
    };
}

pub(crate) use impl_shared_conditional_consumer;
