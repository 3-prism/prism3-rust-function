//! # Consumer Macro Definitions
//!
//! Provides declarative macros to simplify Consumer implementations and
//! reduce code duplication.
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
        /// Creates a new $type_desc.
        ///
        /// Wraps the provided closure in the appropriate smart pointer
        /// type for this $type_desc implementation.
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
        /// Returns a new $type_desc instance wrapping the closure.
        pub fn new<F>($f: F) -> Self
        where
            F: $($fn_trait_with_bounds)+,
        {
            Self {
                function: $wrapper_expr,
                name: None,
            }
        }

        /// Creates a new named $type_desc.
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
        /// * `name` - The name for this $type_desc
        /// * `f` - The closure to wrap
        ///
        /// # Returns
        ///
        /// Returns a new named $type_desc instance wrapping the closure.
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
    (@name_methods) => {
        /// Gets the name of this consumer.
        ///
        /// # Returns
        ///
        /// Returns `Some(&str)` if a name was set, `None` otherwise.
        pub fn name(&self) -> Option<&str> {
            self.name.as_deref()
        }

        /// Sets the name of this consumer.
        ///
        /// # Parameters
        ///
        /// * `name` - The name to set for this consumer
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

        impl_consumer_common_methods!(@name_methods);

        /// Creates a no-operation consumer.
        ///
        /// Creates a consumer that does nothing when called. Useful for
        /// default values or placeholder implementations.
        ///
        /// # Returns
        ///
        /// Returns a new consumer instance that performs no operation.
        ///
        /// # Examples
        ///
        /// ```rust
        /// # use prism3_function::BoxConsumer;
        /// let noop = BoxConsumer::<i32>::noop();
        /// noop.accept(&42); // Does nothing
        /// ```
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

        impl_consumer_common_methods!(@name_methods);

        /// Creates a no-operation bi-consumer.
        ///
        /// Creates a bi-consumer that does nothing when called. Useful
        /// for default values or placeholder implementations.
        ///
        /// # Returns
        ///
        /// Returns a new bi-consumer instance that performs no
        /// operation.
        ///
        /// # Examples
        ///
        /// ```rust
        /// # use prism3_function::BoxBiConsumer;
        /// let noop = BoxBiConsumer::<i32, i32>::noop();
        /// noop.accept(&1, &2); // Does nothing
        /// ```
        pub fn noop() -> Self {
            Self::new(|_, _| {})
        }
    };
}

/// Generates Debug and Display trait implementations for Consumer structs
///
/// Generates standard Debug and Display trait implementations for Consumer
/// structs that have a `name: Option<String>` field.
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list (one or more type parameters)
///
/// # Examples
///
/// ```ignore
/// // For single type parameter
/// impl_consumer_debug_display!(BoxConsumer<T>);
///
/// // For two type parameters
/// impl_consumer_debug_display!(BoxBiConsumer<T, U>);
/// ```
macro_rules! impl_consumer_debug_display {
    // Single generic parameter
    ($struct_name:ident < $generic:ident >) => {
        impl<$generic> std::fmt::Debug for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("name", &self.name)
                    .field("function", &"<function>")
                    .finish()
            }
        }

        impl<$generic> std::fmt::Display for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.name {
                    Some(name) => write!(f, "{}({})", stringify!($struct_name), name),
                    None => write!(f, "{}", stringify!($struct_name)),
                }
            }
        }
    };
    // Two generic parameters
    ($struct_name:ident < $generic1:ident, $generic2:ident >) => {
        impl<$generic1, $generic2> std::fmt::Debug for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("name", &self.name)
                    .field("function", &"<function>")
                    .finish()
            }
        }

        impl<$generic1, $generic2> std::fmt::Display for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.name {
                    Some(name) => write!(f, "{}({})", stringify!($struct_name), name),
                    None => write!(f, "{}", stringify!($struct_name)),
                }
            }
        }
    };
}

/// Generates Debug and Display trait implementations for Conditional Consumer structs
///
/// Generates standard Debug and Display trait implementations for Conditional
/// Consumer structs that have `consumer` and `predicate` fields but no `name` field.
///
/// # Parameters
///
/// * `$struct_name` - The struct name
/// * `$generic` - Generic parameter list (one or more type parameters)
///
/// # Examples
///
/// ```ignore
/// // For single type parameter
/// impl_conditional_consumer_debug_display!(BoxConditionalConsumer<T>);
///
/// // For two type parameters
/// impl_conditional_consumer_debug_display!(BoxConditionalBiConsumer<T, U>);
/// ```
macro_rules! impl_conditional_consumer_debug_display {
    // Single generic parameter
    ($struct_name:ident < $generic:ident >) => {
        impl<$generic> std::fmt::Debug for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("consumer", &self.consumer)
                    .field("predicate", &self.predicate)
                    .finish()
            }
        }

        impl<$generic> std::fmt::Display for $struct_name<$generic> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}({}, {})",
                    stringify!($struct_name),
                    self.consumer,
                    self.predicate
                )
            }
        }
    };
    // Two generic parameters
    ($struct_name:ident < $generic1:ident, $generic2:ident >) => {
        impl<$generic1, $generic2> std::fmt::Debug for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    .field("consumer", &self.consumer)
                    .field("predicate", &self.predicate)
                    .finish()
            }
        }

        impl<$generic1, $generic2> std::fmt::Display for $struct_name<$generic1, $generic2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}({}, {})",
                    stringify!($struct_name),
                    self.consumer,
                    self.predicate
                )
            }
        }
    };
}

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

/// Generates when and and_then method implementations for Arc/Rc-based Consumer
///
/// Generates conditional execution when method and chaining and_then method
/// for Arc/Rc-based consumers that borrow &self (because Arc/Rc can be cloned).
///
/// This macro supports both single-parameter and two-parameter consumers through
/// pattern matching on the struct signature.
///
/// # Parameters
///
/// * `$struct_name<$generics>` - The struct name with its generic parameters
///   - Single parameter: `ArcConsumer<T>`
///   - Two parameters: `ArcBiConsumer<T, U>`
/// * `$return_type` - The return type for when (e.g., ArcConditionalConsumer)
/// * `$predicate_conversion` - Method to convert predicate (into_arc or into_rc)
/// * `$consumer_trait` - Consumer trait name (e.g., Consumer, BiConsumer)
/// * `$extra_bounds` - Extra trait bounds ('static for Rc, Send + Sync + 'static for Arc)
///
/// # All Macro Invocations
///
/// | Consumer Type | Struct Signature | `$return_type` | `$predicate_conversion` | `$consumer_trait` | `$extra_bounds` |
/// |---------------|-----------------|----------------|------------------------|------------------|----------------|
/// | **ArcConsumer** | `ArcConsumer<T>` | ArcConditionalConsumer | into_arc | Consumer | Send + Sync + 'static |
/// | **RcConsumer** | `RcConsumer<T>` | RcConditionalConsumer | into_rc | Consumer | 'static |
/// | **ArcStatefulConsumer** | `ArcStatefulConsumer<T>` | ArcConditionalStatefulConsumer | into_arc | StatefulConsumer | Send + Sync + 'static |
/// | **RcStatefulConsumer** | `RcStatefulConsumer<T>` | RcConditionalStatefulConsumer | into_rc | StatefulConsumer | 'static |
/// | **ArcBiConsumer** | `ArcBiConsumer<T, U>` | ArcConditionalBiConsumer | into_arc | BiConsumer | Send + Sync + 'static |
/// | **RcBiConsumer** | `RcBiConsumer<T, U>` | RcConditionalBiConsumer | into_rc | BiConsumer | 'static |
/// | **ArcStatefulBiConsumer** | `ArcStatefulBiConsumer<T, U>` | ArcConditionalStatefulBiConsumer | into_arc | StatefulBiConsumer | Send + Sync + 'static |
/// | **RcStatefulBiConsumer** | `RcStatefulBiConsumer<T, U>` | RcConditionalStatefulBiConsumer | into_rc | StatefulBiConsumer | 'static |
///
/// # Examples
///
/// ```ignore
/// // Single-parameter with Arc
/// impl_shared_consumer_methods!(
///     ArcConsumer<T>,
///     ArcConditionalConsumer,
///     into_arc,
///     Consumer,
///     Send + Sync + 'static
/// );
///
/// // Two-parameter with Rc
/// impl_shared_consumer_methods!(
///     RcBiConsumer<T, U>,
///     RcConditionalBiConsumer,
///     into_rc,
///     BiConsumer,
///     'static
/// );
/// ```
macro_rules! impl_shared_consumer_methods {
    // Single generic parameter
    ($struct_name:ident < $t:ident >, $return_type:ident, $predicate_conversion:ident, $consumer_trait:ident, $($extra_bounds:tt)+) => {
        pub fn when<P>(&self, predicate: P) -> $return_type<$t>
        where
            P: Predicate<$t> + $($extra_bounds)+,
        {
            $return_type {
                consumer: self.clone(),
                predicate: predicate.$predicate_conversion(),
            }
        }

        #[allow(unused_mut)]
        pub fn and_then<C>(&self, mut after: C) -> $struct_name<$t>
        where
            $t: 'static,
            C: $consumer_trait<$t> + $($extra_bounds)+,
        {
            let mut first = self.clone();
            $struct_name::new(move |t: &$t| {
                first.accept(t);
                after.accept(t);
            })
        }
    };
    // Two generic parameters
    ($struct_name:ident < $t:ident, $u:ident >, $return_type:ident, $predicate_conversion:ident, $consumer_trait:ident, $($extra_bounds:tt)+) => {
        pub fn when<P>(&self, predicate: P) -> $return_type<$t, $u>
        where
            P: BiPredicate<$t, $u> + $($extra_bounds)+,
        {
            $return_type {
                consumer: self.clone(),
                predicate: predicate.$predicate_conversion(),
            }
        }

        #[allow(unused_mut)]
        pub fn and_then<C>(&self, mut after: C) -> $struct_name<$t, $u>
        where
            $t: 'static,
            $u: 'static,
            C: $consumer_trait<$t, $u> + $($extra_bounds)+,
        {
            let mut first = self.clone();
            $struct_name::new(move |t: &$t, u: &$u| {
                first.accept(t, u);
                after.accept(t, u);
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
            pub fn and_then<C>(self, next: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + 'static,
            {
                let first = self;
                let second = next;
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
                let then_cons = self.consumer;
                $consumer_type::new(move |t| {
                    if pred.test(t) {
                        then_cons.accept(t);
                    } else {
                        else_consumer.accept(t);
                    }
                })
            }
        }

        impl<$t> $consumer_trait<$t> for $struct_name<$t>
        where
            $t: 'static,
        {
            fn accept(&self, value: &$t) {
                if self.predicate.test(value) {
                    self.consumer.accept(value);
                }
            }

            fn into_box(self) -> BoxConsumer<$t> {
                let pred = self.predicate;
                let consumer = self.consumer;
                BoxConsumer::new(move |t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                })
            }

            fn into_rc(self) -> RcConsumer<$t> {
                let pred = self.predicate.into_rc();
                let consumer = self.consumer.into_rc();
                RcConsumer::new(move |t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                })
            }

            // do NOT override into_arc() because box conditional consumer is not Send + Sync
            // and calling into_arc() will cause a compile error

            fn into_fn(self) -> impl Fn(&$t) {
                let pred = self.predicate;
                let consumer = self.consumer;
                move |t: &$t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                }
            }

            // do NOT override to_xxx() because box conditional consumer is not Clone
            // and calling to_xxx() will cause a compile error
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
                let first = self;
                let second = next;
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
            pub fn or_else<C>(self, else_consumer: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + 'static,
            {
                let pred = self.predicate;
                let then_cons = self.consumer;
                $consumer_type::new(move |t, u| {
                    if pred.test(t, u) {
                        then_cons.accept(t, u);
                    } else {
                        else_consumer.accept(t, u);
                    }
                })
            }
        }

        impl<$t, $u> $consumer_trait<$t, $u> for $struct_name<$t, $u>
        where
            $t: 'static,
            $u: 'static,
        {
            fn accept(&self, first: &$t, second: &$u) {
                if self.predicate.test(first, second) {
                    self.consumer.accept(first, second);
                }
            }

            fn into_box(self) -> BoxBiConsumer<$t, $u> {
                let pred = self.predicate;
                let consumer = self.consumer;
                BoxBiConsumer::new(move |t, u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                })
            }

            fn into_rc(self) -> RcBiConsumer<$t, $u> {
                let pred = self.predicate.into_rc();
                let consumer = self.consumer.into_rc();
                RcBiConsumer::new(move |t, u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                })
            }

            // do NOT override into_arc() because box conditional consumer is not Send + Sync
            // and calling into_arc() will cause a compile error

            fn into_fn(self) -> impl Fn(&$t, &$u)
            where
                $t: 'static,
                $u: 'static,
            {
                let pred = self.predicate;
                let consumer = self.consumer;
                move |t: &$t, u: &$u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                }
            }

            // do NOT override to_xxx() because box conditional consumer is not Clone
            // and calling to_xxx() will cause a compile error
        }
    };
}

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
        $struct_name:ident<$t:ident>,
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
            pub fn and_then<C>(&self, next: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + $($extra_bounds)+,
            {
                let first = self.clone();
                $consumer_type::new(move |t| {
                    first.accept(t);
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
            pub fn or_else<C>(&self, else_consumer: C) -> $consumer_type<$t>
            where
                C: $consumer_trait<$t> + $($extra_bounds)+,
            {
                let pred = self.predicate.clone();
                let then_cons = self.consumer.clone();
                $consumer_type::new(move |t| {
                    if pred.test(t) {
                        then_cons.accept(t);
                    } else {
                        else_consumer.accept(t);
                    }
                })
            }
        }

        impl<$t> $consumer_trait<$t> for $struct_name<$t>
        where
            $t: 'static,
        {
            fn accept(&self, value: &$t) {
                if self.predicate.test(value) {
                    self.consumer.accept(value);
                }
            }

            fn into_box(self) -> BoxConsumer<$t> {
                let pred = self.predicate;
                let consumer = self.consumer;
                BoxConsumer::new(move |t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                })
            }

            fn into_rc(self) -> RcConsumer<$t> {
                let pred = self.predicate;
                let consumer = self.consumer;
                RcConsumer::new(move |t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                })
            }

            // Use trait default implementation for into_arc()
            // Arc types will work because they satisfy Send + Sync constraints
            // Rc types will get compile error if trying to call into_arc() due to Send + Sync constraints

            fn into_fn(self) -> impl Fn(&$t) {
                let pred = self.predicate;
                let consumer = self.consumer;
                move |t: &$t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                }
            }

            fn to_box(&self) -> BoxConsumer<$t> {
                let pred = self.predicate.clone();
                let consumer = self.consumer.clone();
                BoxConsumer::new(move |t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                })
            }

            fn to_rc(&self) -> RcConsumer<$t> {
                let pred = self.predicate.clone();
                let consumer = self.consumer.clone();
                RcConsumer::new(move |t| {
                    if pred.test(t) {
                        consumer.accept(t);
                    }
                })
            }

            // Use trait default implementation for to_arc()
            // Arc types will work because they satisfy Clone + Send + Sync constraints
            // Rc types will get compile error if trying to call to_arc() due to Send + Sync constraints
        }
    };

    // Two generic parameters - BiConsumer
    (
        $struct_name:ident<$t:ident, $u:ident>,
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
            pub fn and_then<C>(&self, next: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + $($extra_bounds)+,
            {
                let first = self.clone();
                $consumer_type::new(move |t, u| {
                    first.accept(t, u);
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
            pub fn or_else<C>(&self, else_consumer: C) -> $consumer_type<$t, $u>
            where
                C: $consumer_trait<$t, $u> + $($extra_bounds)+,
            {
                let pred = self.predicate.clone();
                let then_cons = self.consumer.clone();
                $consumer_type::new(move |t, u| {
                    if pred.test(t, u) {
                        then_cons.accept(t, u);
                    } else {
                        else_consumer.accept(t, u);
                    }
                })
            }
        }

        impl<$t, $u> $consumer_trait<$t, $u> for $struct_name<$t, $u>
        where
            $t: 'static,
            $u: 'static,
        {
            fn accept(&self, first: &$t, second: &$u) {
                if self.predicate.test(first, second) {
                    self.consumer.accept(first, second);
                }
            }

            fn into_box(self) -> BoxBiConsumer<$t, $u> {
                let pred = self.predicate;
                let consumer = self.consumer;
                BoxBiConsumer::new(move |t, u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                })
            }

            fn into_rc(self) -> RcBiConsumer<$t, $u> {
                let pred = self.predicate;
                let consumer = self.consumer;
                RcBiConsumer::new(move |t, u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                })
            }

            // Use trait default implementation for into_arc()
            // Arc types will work because they satisfy Send + Sync constraints
            // Rc types will get compile error if trying to call into_arc() due to Send + Sync constraints

            fn into_fn(self) -> impl Fn(&$t, &$u)
            where
                $t: 'static,
                $u: 'static,
            {
                let pred = self.predicate;
                let consumer = self.consumer;
                move |t: &$t, u: &$u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                }
            }

            fn to_box(&self) -> BoxBiConsumer<$t, $u> {
                let pred = self.predicate.clone();
                let consumer = self.consumer.clone();
                BoxBiConsumer::new(move |t, u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                })
            }

            fn to_rc(&self) -> RcBiConsumer<$t, $u> {
                let pred = self.predicate.clone();
                let consumer = self.consumer.clone();
                RcBiConsumer::new(move |t, u| {
                    if pred.test(t, u) {
                        consumer.accept(t, u);
                    }
                })
            }

            // Use trait default implementation for to_arc()
            // Arc types will work because they satisfy Clone + Send + Sync constraints
            // Rc types will get compile error if trying to call to_arc() due to Send + Sync constraints
        }
    };
}

/// Generates conversion methods for Conditional Consumer implementations
///
/// This macro generates the conversion methods (`into_box`, `into_rc`, `into_fn`) for
/// conditional consumer types. It handles both immutable (Consumer) and mutable
/// (StatefulConsumer) cases using the `#[allow(unused_mut)]` annotation.
///
/// The macro works by always declaring variables as `mut`, which is necessary for
/// StatefulConsumer cases, while suppressing unused_mut warnings for Consumer cases
/// where the mutability is not needed.
///
/// # Parameters
///
/// * `$box_type<$t:ident>` - The box-based consumer type (e.g., `BoxConsumer<T>`)
/// * `$rc_type:ident` - The rc-based consumer type name (e.g., `RcConsumer`)
/// * `$fn_trait:ident` - The function trait (e.g., `Fn` or `FnMut`)
///
/// # Usage Examples
///
/// For Consumer (immutable):
/// ```ignore
/// impl<T> Consumer<T> for BoxConditionalConsumer<T>
/// where
///     T: 'static,
/// {
///     fn accept(&self, value: &T) {
///         if self.predicate.test(value) {
///             self.consumer.accept(value);
///         }
///     }
///
///     impl_conditional_consumer_conversions!(
///         BoxConsumer<T>,
///         RcConsumer,
///         Fn
///     );
/// }
/// ```
///
/// For StatefulConsumer (mutable):
/// ```ignore
/// impl<T> StatefulConsumer<T> for BoxConditionalStatefulConsumer<T>
/// where
///     T: 'static,
/// {
///     fn accept(&mut self, value: &T) {
///         if self.predicate.test(value) {
///             self.consumer.accept(value);
///         }
///     }
///
///     impl_conditional_consumer_conversions!(
///         BoxStatefulConsumer<T>,
///         RcStatefulConsumer,
///         FnMut
///     );
/// }
/// ```
///
/// # Implementation Details
///
/// - Uses `#[allow(unused_mut)]` to handle Consumer cases where `mut` is not needed
/// - The closures inside `into_box` and `into_rc` will automatically capture as `Fn`
///   or `FnMut` based on their internal operations
/// - The `into_fn` method uses the provided `$fn_trait` parameter to match the
///   intended trait type
macro_rules! impl_conditional_consumer_conversions {
    // Single generic parameter - Consumer
    (
        $box_type:ident < $t:ident >,
        $rc_type:ident,
        $fn_trait:ident
    ) => {
        #[allow(unused_mut)]
        fn into_box(self) -> $box_type<$t> {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            $box_type::new(move |t| {
                if pred.test(t) {
                    consumer.accept(t);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_rc(self) -> $rc_type<$t> {
            let pred = self.predicate.into_rc();
            let mut consumer = self.consumer.into_rc();
            let mut consumer_fn = consumer;
            $rc_type::new(move |t| {
                if pred.test(t) {
                    consumer_fn.accept(t);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_fn(self) -> impl $fn_trait(&$t) {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            move |t: &$t| {
                if pred.test(t) {
                    consumer.accept(t);
                }
            }
        }
    };

    // Two generic parameters - BiConsumer
    (
        $box_type:ident < $t:ident, $u:ident >,
        $rc_type:ident,
        $fn_trait:ident
    ) => {
        #[allow(unused_mut)]
        fn into_box(self) -> $box_type<$t, $u> {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            $box_type::new(move |t, u| {
                if pred.test(t, u) {
                    consumer.accept(t, u);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_rc(self) -> $rc_type<$t, $u> {
            let pred = self.predicate.into_rc();
            let mut consumer = self.consumer.into_rc();
            let mut consumer_fn = consumer;
            $rc_type::new(move |t, u| {
                if pred.test(t, u) {
                    consumer_fn.accept(t, u);
                }
            })
        }

        #[allow(unused_mut)]
        fn into_fn(self) -> impl $fn_trait(&$t, &$u) {
            let pred = self.predicate;
            let mut consumer = self.consumer;
            move |t: &$t, u: &$u| {
                if pred.test(t, u) {
                    consumer.accept(t, u);
                }
            }
        }
    };
}

// Export all macros for use within the module
pub(crate) use impl_bi_consumer_conversions;
pub(crate) use impl_box_conditional_consumer;
pub(crate) use impl_box_consumer_methods;
pub(crate) use impl_conditional_consumer_clone;
pub(crate) use impl_conditional_consumer_conversions;
pub(crate) use impl_conditional_consumer_debug_display;
pub(crate) use impl_consumer_clone;
pub(crate) use impl_consumer_common_methods;
pub(crate) use impl_consumer_conversions;
pub(crate) use impl_consumer_debug_display;
pub(crate) use impl_shared_conditional_consumer;
pub(crate) use impl_shared_consumer_methods;
