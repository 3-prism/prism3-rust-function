//! # Mutator Macro Definitions
//!
//! Provides declarative macros to simplify Mutator implementations and
//! reduce code duplication. These macros are for internal module use only
//! and are not exported outside the module.
//!
//! # Author
//!
//! Haixing Hu

////////////////////////////////////////////////////////////////////////////////
// Factory method macros
////////////////////////////////////////////////////////////////////////////////

/// Generates the `noop` method
///
/// # Parameters
///
/// * `$new_fn` - The constructor function name used to create instances
macro_rules! impl_mutator_noop {
    ($new_fn:ident) => {
        pub fn noop() -> Self
        where
            T: 'static,
        {
            Self::$new_fn(|_| {})
        }
    };
}

/// Generates the `constant` method
///
/// # Parameters
///
/// * `$new_fn` - The constructor function name used to create instances
macro_rules! impl_mutator_constant {
    ($new_fn:ident) => {
        pub fn constant(value: T) -> Self
        where
            T: Clone + 'static,
        {
            Self::$new_fn(move |x| *x = value.clone())
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
// when method macros
////////////////////////////////////////////////////////////////////////////////

/// Generates the `when` method (Box type, consumes self)
///
/// # Parameters
///
/// * `$conditional_type` - Name of the conditional type
/// * `$predicate_method` - Method name to convert Predicate to wrapper type
macro_rules! impl_box_mutator_when {
    ($conditional_type:ident, $predicate_method:ident) => {
        pub fn when<P>(self, predicate: P) -> $conditional_type<T>
        where
            P: Predicate<T> + 'static,
            T: 'static,
        {
            $conditional_type {
                mutator: self,
                predicate: predicate.$predicate_method(),
            }
        }
    };
}

/// Generates the `when` method (Rc/Arc type, borrows &self)
///
/// # Parameters
///
/// * `$conditional_type` - Name of the conditional type
/// * `$predicate_method` - Method name to convert Predicate to wrapper type
macro_rules! impl_shared_mutator_when {
    ($conditional_type:ident, $predicate_method:ident) => {
        pub fn when<P>(&self, predicate: P) -> $conditional_type<T>
        where
            P: Predicate<T> + 'static,
            T: 'static,
        {
            $conditional_type {
                mutator: self.clone(),
                predicate: predicate.$predicate_method(),
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
// Clone implementation macros
////////////////////////////////////////////////////////////////////////////////

/// Generates Clone implementation (Rc/Arc single-field struct)
///
/// # Parameters
///
/// * `$struct_name` - Struct name
/// * `$field_name` - Field name
macro_rules! impl_mutator_clone {
    ($struct_name:ident, $field_name:ident) => {
        impl<T> Clone for $struct_name<T> {
            fn clone(&self) -> Self {
                $struct_name {
                    $field_name: self.$field_name.clone(),
                }
            }
        }
    };
}

/// Generates Clone implementation for ConditionalMutator
///
/// # Parameters
///
/// * `$struct_name` - Conditional mutator struct name
macro_rules! impl_conditional_mutator_clone {
    ($struct_name:ident) => {
        impl<T> Clone for $struct_name<T> {
            fn clone(&self) -> Self {
                $struct_name {
                    mutator: self.mutator.clone(),
                    predicate: self.predicate.clone(),
                }
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
// Export macros for internal module use
////////////////////////////////////////////////////////////////////////////////

pub(crate) use impl_box_mutator_when;
pub(crate) use impl_conditional_mutator_clone;
pub(crate) use impl_mutator_clone;
pub(crate) use impl_mutator_constant;
pub(crate) use impl_mutator_noop;
pub(crate) use impl_shared_mutator_when;
