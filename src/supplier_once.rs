/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # SupplierOnce Types
//!
//! Provides one-time supplier implementations that generate and
//! return values without taking any input parameters, consuming
//! themselves in the process.
//!
//! # Overview
//!
//! A **SupplierOnce** is a functional abstraction similar to
//! `Supplier`, but can only be called once. The `get()` method
//! consumes `self`, ensuring the supplier cannot be reused.
//!
//! # Key Characteristics
//!
//! - **Single use**: Can only call `get()` once
//! - **Consumes self**: The method takes ownership of `self`
//! - **Holds `FnOnce`**: Can capture and move non-cloneable values
//! - **Type-system guaranteed**: Prevents multiple calls at compile
//!   time
//!
//! # Use Cases
//!
//! 1. **Lazy initialization**: Delay expensive computation until
//!    needed
//! 2. **One-time resource consumption**: Generate value by consuming
//!    a resource
//! 3. **Move-only closures**: Hold closures that capture moved
//!    values
//!
//! # Examples
//!
//! ## Lazy Initialization
//!
//! ```rust
//! use prism3_function::{BoxSupplierOnce, SupplierOnce};
//!
//! let once = BoxSupplierOnce::new(|| {
//!     println!("Expensive initialization");
//!     42
//! });
//!
//! let value = once.get(); // Only initializes once
//! assert_eq!(value, 42);
//! ```
//!
//! ## Moving Captured Values
//!
//! ```rust
//! use prism3_function::{BoxSupplierOnce, SupplierOnce};
//!
//! let resource = String::from("data");
//! let once = BoxSupplierOnce::new(move || resource);
//!
//! let value = once.get();
//! assert_eq!(value, "data");
//! ```
//!
//! # Author
//!
//! Haixing Hu

// ==========================================================================
// SupplierOnce Trait
// ==========================================================================

/// One-time supplier trait: generates a value consuming self.
///
/// Similar to `Supplier`, but can only be called once. The `get()`
/// method consumes `self`, ensuring the supplier cannot be reused.
///
/// # Key Characteristics
///
/// - **Single use**: Can only call `get()` once
/// - **Consumes self**: The method takes ownership of `self`
/// - **Holds `FnOnce`**: Can capture and move non-cloneable values
/// - **Type-system guaranteed**: Prevents multiple calls at compile
///   time
///
/// # Use Cases
///
/// 1. **Lazy initialization**: Delay expensive computation until
///    needed
/// 2. **One-time resource consumption**: Generate value by consuming
///    a resource
/// 3. **Move-only closures**: Hold closures that capture moved
///    values
///
/// # Examples
///
/// ## Lazy Initialization
///
/// ```rust
/// use prism3_function::{BoxSupplierOnce, SupplierOnce};
///
/// let once = BoxSupplierOnce::new(|| {
///     println!("Expensive computation");
///     42
/// });
///
/// let value = once.get(); // Prints: Expensive computation
/// assert_eq!(value, 42);
/// // once is now consumed and cannot be used again
/// ```
///
/// ## Resource Consumption
///
/// ```rust
/// use prism3_function::{BoxSupplierOnce, SupplierOnce};
///
/// let resource = String::from("data");
/// let once = BoxSupplierOnce::new(move || {
///     resource // Move the resource
/// });
///
/// let value = once.get();
/// assert_eq!(value, "data");
/// ```
///
/// # Author
///
/// Haixing Hu
pub trait SupplierOnce<T> {
    /// Generates and returns the value, consuming self.
    ///
    /// This method can only be called once because it consumes
    /// `self`. This ensures type-system level guarantee that the
    /// supplier won't be called multiple times.
    ///
    /// # Returns
    ///
    /// The generated value of type `T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplierOnce, SupplierOnce};
    ///
    /// let once = BoxSupplierOnce::new(|| 42);
    /// assert_eq!(once.get(), 42);
    /// // once is consumed here
    /// ```
    fn get(self) -> T;

    /// Converts to `BoxSupplierOnce`.
    ///
    /// # Returns
    ///
    /// A new `BoxSupplierOnce<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::SupplierOnce;
    ///
    /// let closure = || 42;
    /// let boxed = closure.into_box_once();
    /// assert_eq!(boxed.get(), 42);
    /// ```
    fn into_box_once(self) -> BoxSupplierOnce<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxSupplierOnce::new(move || self.get())
    }
}

// ==========================================================================
// Implement SupplierOnce for Closures
// ==========================================================================

impl<T, F> SupplierOnce<T> for F
where
    F: FnOnce() -> T,
{
    fn get(self) -> T {
        self()
    }
}

// ==========================================================================
// BoxSupplierOnce - One-time Supplier Implementation
// ==========================================================================

/// Box-based one-time supplier.
///
/// Uses `Box<dyn FnOnce() -> T>` for one-time value generation.
/// Can only call `get()` once, consuming the supplier.
///
/// # Examples
///
/// ## Lazy Initialization
///
/// ```rust
/// use prism3_function::{BoxSupplierOnce, SupplierOnce};
///
/// let once = BoxSupplierOnce::new(|| {
///     println!("Expensive initialization");
///     42
/// });
///
/// let value = once.get(); // Prints: Expensive initialization
/// assert_eq!(value, 42);
/// ```
///
/// ## Moving Captured Values
///
/// ```rust
/// use prism3_function::{BoxSupplierOnce, SupplierOnce};
///
/// let resource = String::from("data");
/// let once = BoxSupplierOnce::new(move || resource);
///
/// let value = once.get();
/// assert_eq!(value, "data");
/// ```
///
/// # Author
///
/// Haixing Hu
pub struct BoxSupplierOnce<T> {
    func: Option<Box<dyn FnOnce() -> T>>,
}

impl<T> BoxSupplierOnce<T> {
    /// Creates a new `BoxSupplierOnce`.
    ///
    /// # Parameters
    ///
    /// * `f` - The closure to wrap
    ///
    /// # Returns
    ///
    /// A new `BoxSupplierOnce<T>` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prism3_function::{BoxSupplierOnce, SupplierOnce};
    ///
    /// let once = BoxSupplierOnce::new(|| 42);
    /// assert_eq!(once.get(), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        BoxSupplierOnce {
            func: Some(Box::new(f)),
        }
    }
}

impl<T> SupplierOnce<T> for BoxSupplierOnce<T> {
    fn get(mut self) -> T {
        (self.func.take().expect("Supplier already consumed"))()
    }
}
