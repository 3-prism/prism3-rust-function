// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for StatefulFunction trait and its implementations

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::ArcPredicate;
use qubit_function::ArcStatefulFunction;
use qubit_function::BoxPredicate;
use qubit_function::BoxStatefulFunction;
use qubit_function::FunctionOnce;
use qubit_function::RcPredicate;
use qubit_function::RcStatefulFunction;
use qubit_function::StatefulFunction;

// ============================================================================
// StatefulFunction Trait Tests - Core Functionality
// ============================================================================

/// Custom struct for testing StatefulFunction trait default implementations
#[derive(Clone)]
struct CustomStatefulFunction {
    multiplier: i32,
}

impl StatefulFunction<i32, i32> for CustomStatefulFunction {
    fn apply(&mut self, input: &i32) -> i32 {
        self.multiplier += 1;
        input * self.multiplier
    }
}

// ============================================================================
// ArcConditionalStatefulFunction Clone Tests
// ============================================================================

#[test]
fn test_stateful_function_trait_apply() {
    // Test that StatefulFunction trait's apply method works correctly

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let func = move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x + current
    };
    assert_eq!(func.clone().apply(&10), 10);
    assert_eq!(func.clone().apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
}

// ============================================================================
// BoxStatefulFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_box_stateful_function_new() {
    // Test BoxStatefulFunction::new with simple closure

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x + current
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
}

#[test]
fn test_box_stateful_function_identity() {
    // Test BoxStatefulFunction::identity
    let mut identity = BoxStatefulFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_box_stateful_function_constant() {
    // Test BoxStatefulFunction::constant
    let mut constant = BoxStatefulFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
    assert_eq!(constant.apply(&0), "hello");
}

#[test]
fn test_box_stateful_function_apply() {
    // Test StatefulFunction trait implementation for BoxStatefulFunction

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x * current
    });
    assert_eq!(func.apply(&10), 0);
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&10), 20);
}

// ============================================================================
// BoxStatefulFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_box_stateful_function_and_then() {
    // Test BoxStatefulFunction::and_then composition
    let mut counter1 = 0;
    let func1 = BoxStatefulFunction::new(move |x: &i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let func2 = BoxStatefulFunction::new(move |x: &i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = func1.and_then(func2);
    assert_eq!(composed.apply(&10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(&10), 24); // (10 + 2) * 2
}

// ============================================================================
// BoxStatefulFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_box_stateful_function_when_or_else() {
    // Test conditional execution with when/or_else
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        *counter_clone.borrow_mut() += 1;
        x * 2
    })
    .when(|x: &i32| *x > 10)
    .or_else(|x: &i32| x + 1);

    assert_eq!(func.apply(&15), 30); // 15 > 10, apply * 2
    assert_eq!(func.apply(&5), 6); // 5 <= 10, apply + 1
    assert_eq!(*counter.borrow(), 1); // Only the first call satisfies the condition
}

#[test]
fn test_box_stateful_function_when_with_predicate() {
    // Test when with BoxPredicate

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x * current
    })
    .when(BoxPredicate::new(|x: &i32| *x > 0))
    .or_else(|x: &i32| -(*x));

    assert_eq!(func.apply(&10), 0); // 10 > 0, apply * 0
    assert_eq!(func.apply(&-5), 5); // -5 <= 0, apply negate
}

// ============================================================================
// BoxStatefulFunction Tests - Type Conversions
// ============================================================================

// ============================================================================
// ArcStatefulFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_arc_stateful_function_new() {
    // Test ArcStatefulFunction::new with simple closure

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    let mut func = ArcStatefulFunction::new(move |x: &i32| {
        let mut current = counter_clone.lock().expect("mutex should not be poisoned");
        let result = x + *current;
        *current += 1;
        result
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
}

#[test]
fn test_arc_stateful_function_identity() {
    // Test ArcStatefulFunction::identity
    let mut identity = ArcStatefulFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_arc_stateful_function_constant() {
    // Test ArcStatefulFunction::constant
    let mut constant = ArcStatefulFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
}

#[test]
fn test_arc_stateful_function_apply() {
    // Test StatefulFunction trait implementation for ArcStatefulFunction

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    let mut func = ArcStatefulFunction::new(move |x: &i32| {
        let mut current = counter_clone.lock().expect("mutex should not be poisoned");
        let result = x * *current;
        *current += 1;
        result
    });
    assert_eq!(func.apply(&10), 0);
    assert_eq!(func.apply(&10), 10);
}

#[test]
fn test_arc_stateful_function_clone() {
    // Test ArcStatefulFunction::clone

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    let func = ArcStatefulFunction::new(move |x: &i32| {
        let mut current = counter_clone.lock().expect("mutex should not be poisoned");
        let result = x + *current;
        *current += 1;
        result
    });
    let mut func_clone = func.clone();
    assert_eq!(func_clone.apply(&10), 10);
    assert_eq!(func_clone.apply(&10), 11);
}

// ============================================================================
// ArcStatefulFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_arc_stateful_function_and_then() {
    // Test ArcStatefulFunction::and_then composition
    let mut counter1 = 0;
    let func1 = ArcStatefulFunction::new(move |x: &i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let func2 = ArcStatefulFunction::new(move |x: &i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = func1.and_then(func2);
    assert_eq!(composed.apply(&10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(&10), 24); // (10 + 2) * 2
}

// ============================================================================
// ArcStatefulFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_arc_stateful_function_when_or_else() {
    // Test conditional execution with when/or_else
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    let mut func = ArcStatefulFunction::new(move |x: &i32| {
        *counter_clone.lock().expect("mutex should not be poisoned") += 1;
        x * 2
    })
    .when(|x: &i32| *x > 10)
    .or_else(|x: &i32| x + 1);

    assert_eq!(func.apply(&15), 30); // 15 > 10, apply * 2
    assert_eq!(func.apply(&5), 6); // 5 <= 10, apply + 1
    assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 1); // Only the first call satisfies the condition
}

#[test]
fn test_arc_stateful_function_when_with_predicate() {
    // Test when with ArcPredicate

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    let mut func = ArcStatefulFunction::new(move |x: &i32| {
        let mut current = counter_clone.lock().expect("mutex should not be poisoned");
        let result = x * *current;
        *current += 1;
        result
    })
    .when(ArcPredicate::new(|x: &i32| *x > 0))
    .or_else(|x: &i32| -(*x));

    assert_eq!(func.apply(&10), 0); // 10 > 0, apply * 0
    assert_eq!(func.apply(&-5), 5); // -5 <= 0, apply negate
}

// ============================================================================
// ArcStatefulFunction Tests - Type Conversions
// ============================================================================

// ============================================================================
// ArcStatefulFunction Tests - Thread Safety
// ============================================================================

#[test]
fn test_arc_stateful_function_thread_safety() {
    // Test that ArcStatefulFunction is Send + Sync

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    let func = ArcStatefulFunction::new(move |x: &i32| {
        let mut current = counter_clone.lock().expect("mutex should not be poisoned");
        let result = x + *current;
        *current += 1;
        result
    });
    let mut func_clone = func.clone();

    let handle = std::thread::spawn(move || func_clone.apply(&10));

    assert_eq!(handle.join().expect("thread should not panic"), 10);
}

// ============================================================================
// RcStatefulFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_rc_stateful_function_new() {
    // Test RcStatefulFunction::new with simple closure

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = RcStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x + current
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
}

#[test]
fn test_rc_stateful_function_identity() {
    // Test RcStatefulFunction::identity
    let mut identity = RcStatefulFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_rc_stateful_function_constant() {
    // Test RcStatefulFunction::constant
    let mut constant = RcStatefulFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
}

#[test]
fn test_rc_stateful_function_apply() {
    // Test StatefulFunction trait implementation for RcStatefulFunction

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = RcStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x * current
    });
    assert_eq!(func.apply(&10), 0);
    assert_eq!(func.apply(&10), 10);
}

#[test]
fn test_rc_stateful_function_clone() {
    // Test RcStatefulFunction::clone

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let func = RcStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x + current
    });
    let mut func_clone = func.clone();
    assert_eq!(func_clone.apply(&10), 10);
    assert_eq!(func_clone.apply(&10), 11);
}

// ============================================================================
// RcStatefulFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_rc_stateful_function_and_then() {
    // Test RcStatefulFunction::and_then composition
    let mut counter1 = 0;
    let func1 = RcStatefulFunction::new(move |x: &i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let func2 = RcStatefulFunction::new(move |x: &i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = func1.and_then(func2);
    assert_eq!(composed.apply(&10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(&10), 24); // (10 + 2) * 2
}

// ============================================================================
// RcStatefulFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_rc_stateful_function_when_or_else() {
    // Test conditional execution with when/or_else
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = RcStatefulFunction::new(move |x: &i32| {
        *counter_clone.borrow_mut() += 1;
        x * 2
    })
    .when(|x: &i32| *x > 10)
    .or_else(|x: &i32| x + 1);

    assert_eq!(func.apply(&15), 30); // 15 > 10, apply * 2
    assert_eq!(func.apply(&5), 6); // 5 <= 10, apply + 1
    assert_eq!(*counter.borrow(), 1); // Only the first call satisfies the condition
}
