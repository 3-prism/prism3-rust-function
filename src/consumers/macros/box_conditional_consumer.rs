/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Box Conditional Consumer Macro
//!
//! Generates Box-based Conditional Consumer implementations
//!
//! For Box-based conditional consumers, generates `and_then` and `or_else` methods,
//! as well as complete Consumer/BiConsumer trait implementations.
//!
//! Box type characteristics:
//! - `and_then` and `or_else` consume self (because Box cannot Clone)
//! - Does not implement `into_arc()` (because Box types are not Send + Sync)
//! - Does not implement `to_xxx()` methods (because Box types cannot Clone)
//!
//! # Parameters
//!
//! * `$struct_name<$generics>` - Struct name with generic parameters
//! * `$consumer_type` - Consumer wrapper type name
//! * `$consumer_trait` - Consumer trait name
//!
//! # Usage Examples
//!
//! ```ignore
//! // Single-parameter Consumer
//! impl_box_conditional_consumer!(
//!     BoxConditionalConsumer<T>,
//!     BoxConsumer,
//!     Consumer
//! );
//!
//! // Two-parameter BiConsumer
//! impl_box_conditional_consumer!(
//!     BoxConditionalBiConsumer<T, U>,
//!     BoxBiConsumer,
//!     BiConsumer
//! );
//! ```
//!
//! # Author
//!
//! Haixing Hu

/// Generates Box-based Conditional Consumer implementations
///
/// For Box-based conditional consumers, generates `and_then` and `or_else` methods,
/// as well as complete Consumer/BiConsumer trait implementations.
///
/// Box type characteristics:
/// - `and_then` and `or_else` consume self (because Box cannot Clone)
/// - Does not implement `into_arc()` (because Box types are not Send + Sync)
/// - Does not implement `to_xxx()` methods (because Box types cannot Clone)
///
/// # Parameters
///
/// * `$struct_name<$generics>` - Struct name with generic parameters
/// * `$consumer_type` - Consumer wrapper type name
/// * `$consumer_trait` - Consumer trait name
///
/// # Usage Examples
///
/// ```ignore
/// // Single-parameter Consumer
/// impl_box_conditional_consumer!(
///     BoxConditionalConsumer<T>,
///     BoxConsumer,
///     Consumer
/// );
///
/// // Two-parameter BiConsumer
/// impl_box_conditional_consumer!(
///     BoxConditionalBiConsumer<T, U>,
///     BoxBiConsumer,
///     BiConsumer
/// );
/// ```
macro_rules! impl_box_conditional_consumer {
    // Single generic parameter - Consumer
    (
        $struct_name:ident<$t:ident>,
        $consumer_type:ident,
        $consumer_trait:ident
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
            pub fn and_then<C>(self, next: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + 'static,
            {
                let mut first = self;
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
            pub fn or_else<C>(self, else_consumer: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + 'static,
            {
                let pred = self.predicate;
                #[allow(unused_mut)]
                let mut then_cons = self.consumer;
                #[allow(unused_mut)]
                let mut else_consumer = else_consumer;
                $consumer_type::new(move |t| {
                    if pred.test(t) {
                        then_cons.accept(t);
                    } else {
                        else_consumer.accept(t);
                    }
                })
            }
        }
    };

    // Two generic parameters - BiConsumer
    (
        $struct_name:ident<$t:ident, $u:ident>,
        $consumer_type:ident,
        $consumer_trait:ident
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
            pub fn and_then<C>(self, next: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + 'static,
            {
                #[allow(unused_mut)]
                let mut first = self;
                #[allow(unused_mut)]
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
            pub fn or_else<C>(self, else_consumer: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + 'static,
            {
                let pred = self.predicate;
                let mut then_consumer = self.consumer;
                let mut else_consumer = else_consumer;
                $consumer_type::new(move |t, u| {
                    if pred.test(t, u) {
                        then_consumer.accept(t, u);
                    } else {
                        else_consumer.accept(t, u);
                    }
                })
            }
        }
    };
}

pub(crate) use impl_box_conditional_consumer;
