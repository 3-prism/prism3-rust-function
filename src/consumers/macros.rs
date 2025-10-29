//! # Consumer Macro Definitions
//!
//! Provides declarative macros to simplify Consumer implementations and
//! reduce code duplication.
//!
//! # Author
//!
//! Haixing Hu

/// Generates common Consumer methods (name, set_name, noop)
///
/// Generates standard name management methods and noop constructor for
/// Consumer structs.
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list
/// * `$noop_fn` - Closure expression used by the noop method
///
/// # Generated Methods
///
/// * `name()` - Gets the name of the consumer
/// * `set_name()` - Sets the name of the consumer
/// * `noop()` - Creates a consumer that performs no operation
macro_rules! impl_consumer_methods {
    ($struct_name:ident < $($generic:ident),+ >, $noop_fn:expr) => {
        impl<$($generic),+> $struct_name<$($generic),+> {
            /// Get the name of this consumer
            ///
            /// # Return Value
            ///
            /// Returns `Some(&str)` if a name was set, `None` otherwise
            pub fn name(&self) -> Option<&str> {
                self.name.as_deref()
            }

            /// Set the name of this consumer
            ///
            /// # Parameters
            ///
            /// * `name` - The name to set
            pub fn set_name(&mut self, name: impl Into<String>) {
                self.name = Some(name.into());
            }

            /// Create a no-operation consumer
            ///
            /// Creates a consumer that does nothing when called.
            ///
            /// # Return Value
            ///
            /// Returns a new consumer instance that performs no operation
            pub fn noop() -> Self {
                Self::new($noop_fn)
            }
        }
    };
}

/// Generates Debug and Display trait implementations
///
/// Generates standard Debug and Display trait implementations for Consumer
/// structs.
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list
macro_rules! impl_debug_display {
    ($struct_name:ident < $($generic:ident),+ >) => {
        impl<$($generic),+> std::fmt::Debug for $struct_name<$($generic),+> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({:?})", stringify!($struct_name), self.name)
            }
        }

        impl<$($generic),+> std::fmt::Display for $struct_name<$($generic),+> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    self.name.as_deref().unwrap_or(stringify!($struct_name))
                )
            }
        }
    };
}

/// Generates when method implementation for Consumer trait
///
/// Generates conditional execution when method implementation for Consumer
/// trait.
///
/// # Parameters
///
/// * `$trait_name` - Trait name (Consumer, BiConsumer, ConsumerOnce,
///   BiConsumerOnce)
/// * `$box_type` - Box wrapper type
/// * `$params` - Method parameters (e.g., t: &T or t: &T, u: &U)
/// * `$args` - Call arguments (e.g., t or t, u)
macro_rules! impl_consumer_when {
    ($trait_name:ident, $box_type:ident, ($($params:tt)*), ($($args:tt)*)) => {
        fn when<P>(self, predicate: P) -> $box_type<T>
        where
            Self: Sized + 'static,
            T: 'static,
            P: crate::predicates::Predicate<T> + 'static,
        {
            let mut consumer = self;
            let mut pred = predicate;
            $box_type::new(move |$($params)*| {
                if pred.test($($args)*) {
                    consumer.accept($($args)*);
                }
            })
        }
    };
}

/// Generates when method implementation for BiConsumer trait
///
/// Generates conditional execution when method implementation for BiConsumer
/// trait.
///
/// # Parameters
///
/// * `$trait_name` - Trait name
/// * `$box_type` - Box wrapper type
/// * `$params` - Method parameters
/// * `$args` - Call arguments
macro_rules! impl_bi_consumer_when {
    ($trait_name:ident, $box_type:ident, ($($params:tt)*), ($($args:tt)*)) => {
        fn when<P>(self, predicate: P) -> $box_type<T, U>
        where
            Self: Sized + 'static,
            T: 'static,
            U: 'static,
            P: crate::predicates::BiPredicate<T, U> + 'static,
        {
            let mut consumer = self;
            let mut pred = predicate;
            $box_type::new(move |$($params)*| {
                if pred.test($($args)*) {
                    consumer.accept($($args)*);
                }
            })
        }
    };
}

/// Generates and_then method implementation for Consumer
///
/// Generates chaining and_then method implementation for Consumer.
///
/// # Parameters
///
/// * `$box_type` - Box wrapper type
/// * `$params` - Method parameters
/// * `$args` - Call arguments
macro_rules! impl_consumer_and_then {
    ($box_type:ident, ($($params:tt)*), ($($args:tt)*)) => {
        fn and_then<C>(self, after: C) -> $box_type<T>
        where
            Self: Sized + 'static,
            T: 'static,
            C: Consumer<T> + 'static,
        {
            let mut first = self;
            let mut second = after;
            $box_type::new(move |$($params)*| {
                first.accept($($args)*);
                second.accept($($args)*);
            })
        }
    };
}

/// Generates and_then method implementation for BiConsumer
///
/// Generates chaining and_then method implementation for BiConsumer.
///
/// # Parameters
///
/// * `$box_type` - Box wrapper type
/// * `$params` - Method parameters
/// * `$args` - Call arguments
macro_rules! impl_bi_consumer_and_then {
    ($box_type:ident, ($($params:tt)*), ($($args:tt)*)) => {
        fn and_then<C>(self, after: C) -> $box_type<T, U>
        where
            Self: Sized + 'static,
            T: 'static,
            U: 'static,
            C: BiConsumer<T, U> + 'static,
        {
            let mut first = self;
            let mut second = after;
            $box_type::new(move |$($params)*| {
                first.accept($($args)*);
                second.accept($($args)*);
            })
        }
    };
}

/// Generates smart pointer conversion methods (into_box, into_rc, into_arc)
///
/// Generates smart pointer conversion methods for types implementing the
/// Consumer trait.
///
/// # Parameters
///
/// * `$trait_name` - Trait name
/// * `$box_type` - Box wrapper type
/// * `$rc_type` - Rc wrapper type
/// * `$arc_type` - Arc wrapper type
/// * `$params` - accept method parameters
/// * `$args` - accept call arguments
macro_rules! impl_consumer_conversions {
    (
        $trait_name:ident,
        $box_type:ident,
        $rc_type:ident,
        $arc_type:ident,
        ($($params:tt)*),
        ($($args:tt)*)
    ) => {
        fn into_box(self) -> $box_type<T>
        where
            Self: Sized + 'static,
            T: 'static,
        {
            let mut consumer = self;
            $box_type::new(move |$($params)*| consumer.accept($($args)*))
        }

        fn into_rc(self) -> $rc_type<T>
        where
            Self: Sized + 'static,
            T: 'static,
        {
            let mut consumer = self;
            $rc_type::new(move |$($params)*| consumer.accept($($args)*))
        }

        fn into_arc(self) -> $arc_type<T>
        where
            Self: Sized + Send + 'static,
            T: 'static,
        {
            let mut consumer = self;
            $arc_type::new(move |$($params)*| consumer.accept($($args)*))
        }

        fn to_box(&self) -> $box_type<T>
        where
            Self: Sized + Clone + 'static,
            T: 'static,
        {
            self.clone().into_box()
        }

        fn to_rc(&self) -> $rc_type<T>
        where
            Self: Sized + Clone + 'static,
            T: 'static,
        {
            self.clone().into_rc()
        }

        fn to_arc(&self) -> $arc_type<T>
        where
            Self: Sized + Clone + Send + 'static,
            T: 'static,
        {
            self.clone().into_arc()
        }
    };
}

/// Generates smart pointer conversion methods for BiConsumer
///
/// Generates smart pointer conversion methods for types implementing the
/// BiConsumer trait.
///
/// # Parameters
///
/// * `$trait_name` - Trait name
/// * `$box_type` - Box wrapper type
/// * `$rc_type` - Rc wrapper type
/// * `$arc_type` - Arc wrapper type
/// * `$params` - accept method parameters
/// * `$args` - accept call arguments
macro_rules! impl_bi_consumer_conversions {
    (
        $trait_name:ident,
        $box_type:ident,
        $rc_type:ident,
        $arc_type:ident,
        ($($params:tt)*),
        ($($args:tt)*)
    ) => {
        fn into_box(self) -> $box_type<T, U>
        where
            Self: Sized + 'static,
            T: 'static,
            U: 'static,
        {
            let mut consumer = self;
            $box_type::new(move |$($params)*| consumer.accept($($args)*))
        }

        fn into_rc(self) -> $rc_type<T, U>
        where
            Self: Sized + 'static,
            T: 'static,
            U: 'static,
        {
            let mut consumer = self;
            $rc_type::new(move |$($params)*| consumer.accept($($args)*))
        }

        fn into_arc(self) -> $arc_type<T, U>
        where
            Self: Sized + Send + 'static,
            T: 'static,
            U: 'static,
        {
            let mut consumer = self;
            $arc_type::new(move |$($params)*| consumer.accept($($args)*))
        }

        fn to_box(&self) -> $box_type<T, U>
        where
            Self: Sized + Clone + 'static,
            T: 'static,
            U: 'static,
        {
            self.clone().into_box()
        }

        fn to_rc(&self) -> $rc_type<T, U>
        where
            Self: Sized + Clone + 'static,
            T: 'static,
            U: 'static,
        {
            self.clone().into_rc()
        }

        fn to_arc(&self) -> $arc_type<T, U>
        where
            Self: Sized + Clone + Send + 'static,
            T: 'static,
            U: 'static,
        {
            self.clone().into_arc()
        }
    };
}

// Export all macros for use within the module
pub(crate) use impl_bi_consumer_and_then;
pub(crate) use impl_bi_consumer_conversions;
pub(crate) use impl_bi_consumer_when;
pub(crate) use impl_consumer_and_then;
pub(crate) use impl_consumer_conversions;
pub(crate) use impl_consumer_methods;
pub(crate) use impl_consumer_when;
pub(crate) use impl_debug_display;

