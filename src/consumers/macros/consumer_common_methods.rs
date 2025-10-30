/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Consumer Common Methods Macro
//!
//! Generates common Consumer methods (new, new_with_name, name,
//! set_name, noop)
//!
//! Generates constructor methods, name management methods and noop
//! constructor for Consumer structs. This macro should be called inside
//! an impl block.
//!
//! The macro automatically detects the number of generic parameters and
//! generates the appropriate implementations for single-parameter or
//! two-parameter consumers.
//!
//! # Parameters
//!
//! * `$struct_name<$generics>` - Struct name with generic parameters
//! * `$fn_trait_with_bounds` - Closure trait with complete bounds
//!   (e.g., `Fn(&T) + 'static`)
//! * `$wrapper_expr` - Wrapper expression (uses `f` for the closure)
//!
//! # Usage
//!
//! ```ignore
//! // Single generic parameter - Consumer
//! impl_consumer_common_methods!(
//!     BoxConsumer<T>,
//!     (Fn(&T) + 'static),
//!     |f| Box::new(f)
//! );
//!
//! // Single generic parameter - StatefulConsumer
//! impl_consumer_common_methods!(
//!     ArcStatefulConsumer<T>,
//!     (FnMut(&T) + Send + 'static),
//!     |f| Arc::new(Mutex::new(f))
//! );
//!
//! // Two generic parameters - BiConsumer
//! impl_consumer_common_methods!(
//!     BoxBiConsumer<T, U>,
//!     (Fn(&T, &U) + 'static),
//!     |f| Box::new(f)
//! );
//! ```
//!
//! # Generated Methods
//!
//! * `new()` - Creates a new consumer
//! * `new_with_name()` - Creates a named consumer
//! * `name()` - Gets the name of the consumer
//! * `set_name()` - Sets the name of the consumer
//! * `noop()` - Creates a consumer that performs no operation
//!
//! # Author
//!
//! Haixing Hu

/// Generates common Consumer methods (new, new_with_name, name,
/// set_name, noop)
///
/// Generates constructor methods, name management methods and noop
/// constructor for Consumer structs. This macro should be called inside
/// an impl block.
///
/// The macro automatically detects the number of generic parameters and
/// generates the appropriate implementations for single-parameter or
/// two-parameter consumers.
///
/// # Parameters
///
/// * `$struct_name<$generics>` - Struct name with generic parameters
/// * `$fn_trait_with_bounds` - Closure trait with complete bounds
///   (e.g., `Fn(&T) + 'static`)
/// * `$wrapper_expr` - Wrapper expression (uses `f` for the closure)
///
/// # Usage
///
/// ```ignore
/// // Single generic parameter - Consumer
/// impl_consumer_common_methods!(
///     BoxConsumer<T>,
///     (Fn(&T) + 'static),
///     |f| Box::new(f)
/// );
///
/// // Single generic parameter - StatefulConsumer
/// impl_consumer_common_methods!(
///     ArcStatefulConsumer<T>,
///     (FnMut(&T) + Send + 'static),
///     |f| Arc::new(Mutex::new(f))
/// );
///
/// // Two generic parameters - BiConsumer
/// impl_consumer_common_methods!(
///     BoxBiConsumer<T, U>,
///     (Fn(&T, &U) + 'static),
///     |f| Box::new(f)
/// );
/// ```
///
/// # Generated Methods
///
/// * `new()` - Creates a new consumer
/// * `new_with_name()` - Creates a named consumer
/// * `name()` - Gets the name of the consumer
/// * `set_name()` - Sets the name of the consumer
/// * `noop()` - Creates a consumer that performs no operation
macro_rules! impl_consumer_common_methods {
    // Internal rule: generates new and new_with_name methods
    // Parameters:
    //   $fn_trait_with_bounds - Function trait bounds
    //   $f - Closure parameter name
    //   $wrapper_expr - Wrapper expression
    //   $type_desc - Type description for docs (e.g., "consumer" or "bi-consumer")
    (@new_methods
        ($($fn_trait_with_bounds:tt)+),
        |$f:ident| $wrapper_expr:expr,
        $type_desc:literal
    ) => {
        #[doc = concat!("Creates a new ", $type_desc, ".")]
        ///
        #[doc = concat!("Wraps the provided closure in the appropriate smart pointer type for this ", $type_desc, " implementation.")]
        ///
        /// # Type Parameters
        ///
        /// * `F` - The closure type
        ///
        /// # Parameters
        ///
        /// * `f` - The closure to wrap
        ///
        /// # Returns
        ///
        #[doc = concat!("Returns a new ", $type_desc, " instance wrapping the closure.")]
        pub fn new<F>($f: F) -> Self
        where
            F: $($fn_trait_with_bounds)+,
        {
            Self {
                function: $wrapper_expr,
                name: None,
            }
        }

        #[doc = concat!("Creates a new named ", $type_desc, ".")]
        ///
        /// Wraps the provided closure and assigns it a name, which is
        /// useful for debugging and logging purposes.
        ///
        /// # Type Parameters
        ///
        /// * `F` - The closure type
        ///
        /// # Parameters
        ///
        #[doc = concat!("* `name` - The name for this ", $type_desc)]
        /// * `f` - The closure to wrap
        ///
        /// # Returns
        ///
        #[doc = concat!("Returns a new named ", $type_desc, " instance wrapping the closure.")]
        pub fn new_with_name<F>(name: &str, $f: F) -> Self
        where
            F: $($fn_trait_with_bounds)+,
        {
            Self {
                function: $wrapper_expr,
                name: Some(name.to_string()),
            }
        }
    };

    // Internal rule: generates name and set_name methods
    (@name_methods $type_desc:literal) => {
        #[doc = concat!("Gets the name of this ", $type_desc, ".")]
        ///
        /// # Returns
        ///
        /// Returns `Some(&str)` if a name was set, `None` otherwise.
        pub fn name(&self) -> Option<&str> {
            self.name.as_deref()
        }

        #[doc = concat!("Sets the name of this ", $type_desc, ".")]
        ///
        /// # Parameters
        ///
        #[doc = concat!("* `name` - The name to set for this ", $type_desc)]
        pub fn set_name(&mut self, name: impl Into<String>) {
            self.name = Some(name.into());
        }
    };

    // Single generic parameter - Consumer types
    (
        $struct_name:ident < $t:ident >,
        ($($fn_trait_with_bounds:tt)+),
        |$f:ident| $wrapper_expr:expr
    ) => {
        impl_consumer_common_methods!(@new_methods
            ($($fn_trait_with_bounds)+),
            |$f| $wrapper_expr,
            "consumer"
        );

        impl_consumer_common_methods!(@name_methods "consumer");

        /// Creates a no-operation consumer.
        ///
        /// Creates a consumer that does nothing when called. Useful for
        /// default values or placeholder implementations.
        ///
        /// # Returns
        ///
        /// Returns a new consumer instance that performs no operation.
        pub fn noop() -> Self {
            Self::new(|_| {})
        }
    };

    // Two generic parameters - BiConsumer types
    (
        $struct_name:ident < $t:ident, $u:ident >,
        ($($fn_trait_with_bounds:tt)+),
        |$f:ident| $wrapper_expr:expr
    ) => {
        impl_consumer_common_methods!(@new_methods
            ($($fn_trait_with_bounds)+),
            |$f| $wrapper_expr,
            "bi-consumer"
        );

        impl_consumer_common_methods!(@name_methods "bi-consumer");

        /// Creates a no-operation bi-consumer.
        ///
        /// Creates a bi-consumer that does nothing when called. Useful
        /// for default values or placeholder implementations.
        ///
        /// # Returns
        ///
        /// Returns a new bi-consumer instance that performs no operation.
        pub fn noop() -> Self {
            Self::new(|_, _| {})
        }
    };
}

pub(crate) use impl_consumer_common_methods;
