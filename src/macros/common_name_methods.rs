// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! # Common Name Methods Macro
//!
//! Generates common name management methods for function-like structs.

/// Implements common name management methods for function-like structs.
///
/// This macro generates name-management methods for structs that have callback
/// metadata. These methods provide a standardized way to get, set, clear, and
/// chain names for debugging and logging purposes.
///
/// # Parameters
///
/// * `$type_desc:literal` - Description of the type (e.g., "consumer")
///
/// # Generated Methods
///
/// * `name(&self) -> Option<&str>` - Gets the current name if set
/// * `set_name(&mut self, name: &str)` - Sets a new name for the instance
/// * `clear_name(&mut self)` - Clears the current name
/// * `with_name(self, name: &str) -> Self` - Sets a name and returns the
///   instance
macro_rules! impl_common_name_methods {
    ($type_desc:literal) => {
        #[doc = concat!("Gets the name of this ", $type_desc, ".")]
        ///
        /// # Returns
        ///
        /// Returns `Some(&str)` if a name was set, `None` otherwise.
        #[inline(always)]
        pub fn name(&self) -> Option<&str> {
            self.metadata.name()
        }

        #[doc = concat!("Sets the name of this ", $type_desc, ".")]
        ///
        /// # Parameters
        #[doc = concat!("* `name` - The name to set for this ", $type_desc)]
        #[inline(always)]
        pub fn set_name(&mut self, name: &str) {
            self.metadata.set_name(name);
        }

        #[doc = concat!("Clears the name of this ", $type_desc, ".")]
        #[inline(always)]
        pub fn clear_name(&mut self) {
            self.metadata.clear_name();
        }

        #[doc = concat!("Sets the name of this ", $type_desc, " and returns it.")]
        ///
        /// # Parameters
        #[doc = concat!("* `name` - The name to set for this ", $type_desc)]
        ///
        /// # Returns
        #[doc = concat!("This ", $type_desc, " with the supplied name.")]
        #[inline(always)]
        pub fn with_name(mut self, name: &str) -> Self {
            self.metadata.set_name(name);
            self
        }
    };
}

pub(crate) use impl_common_name_methods;
