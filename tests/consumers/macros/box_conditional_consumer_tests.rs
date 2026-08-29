// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::Cell;
use std::rc::Rc;

use qubit_function::BoxConsumer;
use qubit_function::Consumer;

#[test]
fn test_box_conditional_consumer_macro_behavior() {
    let observed = Rc::new(Cell::new(0));
    let captured = Rc::clone(&observed);
    let consumer = BoxConsumer::new(move |value: &i32| captured.set(*value)).when(|value: &i32| *value > 0);
    consumer.accept(&42);
    consumer.accept(&-1);
    assert_eq!(observed.get(), 42);
}
