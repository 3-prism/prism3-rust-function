// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the `BoxConsumer` public type.

use crate::BoxConditionalConsumer;
use crate::Consumer;
use crate::Predicate;
use crate::consumers::macros::impl_box_consumer_methods;
use crate::consumers::macros::impl_consumer_common_methods;
use crate::consumers::macros::impl_consumer_debug_display;

// ============================================================================
// 2. BoxConsumer - Single Ownership Implementation
// ============================================================================

/// BoxConsumer struct
///
/// Non-mutating consumer implementation based on `Box<dyn Fn(&T)>` for single
/// ownership scenarios.
///
/// # Features
///
/// - **Single Ownership**: Not cloneable, transfers ownership when used
/// - **Runtime cost**: One heap allocation and dynamic dispatch; no reference
///   counting or locking
/// - **Shared-reference API**: Invoked through `&self` and shared input
///   references
/// - **No Wrapper Interior Mutability**: No need for Mutex or RefCell in the
///   wrapper
///
/// # Use Cases
///
/// Choose `BoxConsumer` when:
/// - Non-mutating consumer is used once or in a linear flow
/// - No need to share consumer across contexts
/// - Pure observation operations, such as logging
///
/// # Examples
///
/// ```rust
/// use qubit_function::{Consumer, BoxConsumer};
///
/// let consumer = BoxConsumer::new(|x: &i32| {
///     println!("Observed value: {}", x);
/// });
/// consumer.accept(&5);
/// ```
#[must_use = "callback wrappers do nothing unless stored or invoked"]
pub struct BoxConsumer<T> {
    /// The wrapped callback implementation.
    pub(super) function: Box<dyn Fn(&T)>,
    /// Diagnostic metadata associated with this callback.
    pub(super) metadata: crate::internal::CallbackMetadata,
}

impl<T> BoxConsumer<T> {
    // Generates: new(), new_with_name(), name(), set_name(), noop()
    impl_consumer_common_methods!(BoxConsumer<T>, (Fn(&T) + 'static), |f| { Box::new(f) });

    // Generates: when() and and_then() methods that consume self
    impl_box_consumer_methods!(BoxConsumer<T>, BoxConditionalConsumer, Consumer);
}

impl<T> Consumer<T> for BoxConsumer<T> {
    #[inline(always)]
    fn accept(&self, value: &T) {
        (self.function)(value)
    }
}

// Use macro to generate Debug and Display implementations
impl_consumer_debug_display!(BoxConsumer<T>);
