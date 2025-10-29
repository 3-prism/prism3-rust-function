/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Comprehensive tests for StatefulFunction trait and its implementations

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use prism3_function::{
    ArcPredicate,
    ArcStatefulFunction,
    BoxPredicate,
    BoxStatefulFunction,
    RcPredicate,
    RcStatefulFunction,
    StatefulFunction,
};

// ============================================================================
// StatefulFunction Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_stateful_function_trait_apply() {
    // Test that StatefulFunction trait's apply method works correctly
    let mut counter = 0;
    let mut func = |x: &i32| {
        counter += 1;
        x + counter
    };
    assert_eq!(func.apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
    assert_eq!(func.apply(&10), 13);
}

#[test]
fn test_stateful_function_trait_into_box() {
    // Test conversion from closure to BoxStatefulFunction
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut boxed = func.into_box();
    assert_eq!(boxed.apply(&10), 11);
    assert_eq!(boxed.apply(&10), 12);
}

#[test]
fn test_stateful_function_trait_into_rc() {
    // Test conversion from closure to RcStatefulFunction
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut rc = func.into_rc();
    assert_eq!(rc.apply(&10), 11);
    assert_eq!(rc.apply(&10), 12);
}

#[test]
fn test_stateful_function_trait_into_arc() {
    // Test conversion from closure to ArcStatefulFunction
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut arc = func.into_arc();
    assert_eq!(arc.apply(&10), 11);
    assert_eq!(arc.apply(&10), 12);
}

#[test]
fn test_stateful_function_trait_into_fn() {
    // Test conversion to closure
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut closure = func.into_fn();
    assert_eq!(closure(&10), 11);
    assert_eq!(closure(&10), 12);
}

#[test]
fn test_stateful_function_trait_to_box() {
    // Test non-consuming conversion to BoxStatefulFunction
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut boxed = func.to_box();
    assert_eq!(boxed.apply(&10), 11);
    assert_eq!(boxed.apply(&10), 12);
}

#[test]
fn test_stateful_function_trait_to_rc() {
    // Test non-consuming conversion to RcStatefulFunction
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut rc = func.to_rc();
    assert_eq!(rc.apply(&10), 11);
    assert_eq!(rc.apply(&10), 12);
}

#[test]
fn test_stateful_function_trait_to_arc() {
    // Test non-consuming conversion to ArcStatefulFunction
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut arc = func.to_arc();
    assert_eq!(arc.apply(&10), 11);
    assert_eq!(arc.apply(&10), 12);
}

#[test]
fn test_stateful_function_trait_to_fn() {
    // Test non-consuming conversion to closure
    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x + counter
    };
    let mut closure = func.to_fn();
    assert_eq!(closure(&10), 11);
    assert_eq!(closure(&10), 12);
}

// ============================================================================
// BoxStatefulFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_box_stateful_function_new() {
    // Test BoxStatefulFunction::new with simple closure
    let mut counter = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    assert_eq!(func.apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
    assert_eq!(func.apply(&10), 13);
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
    let mut counter = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&10), 20);
    assert_eq!(func.apply(&10), 30);
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

#[test]
fn test_box_stateful_function_compose() {
    // Test BoxStatefulFunction::compose reverse composition
    let mut counter = 0;
    let func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    });

    let mut composed = func.compose(|x: &i32| x + 1);
    assert_eq!(composed.apply(&10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(&10), 22); // (10 + 1) * 2
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
    let mut counter = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    })
    .when(BoxPredicate::new(|x: &i32| *x > 0))
    .or_else(|x: &i32| -(*x));

    assert_eq!(func.apply(&10), 10); // 10 > 0, apply * 1
    assert_eq!(func.apply(&-5), 5); // -5 <= 0, apply negate
}

// ============================================================================
// BoxStatefulFunction Tests - Type Conversions
// ============================================================================

#[test]
fn test_box_stateful_function_into_box() {
    // Test BoxStatefulFunction::into_box (should return itself)
    let mut counter = 0;
    let func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut boxed = func.into_box();
    assert_eq!(boxed.apply(&10), 11);
}

#[test]
fn test_box_stateful_function_into_rc() {
    // Test BoxStatefulFunction::into_rc conversion
    let mut counter = 0;
    let func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut rc = func.into_rc();
    assert_eq!(rc.apply(&10), 11);
}

#[test]
fn test_box_stateful_function_into_fn() {
    // Test BoxStatefulFunction::into_fn conversion
    let mut counter = 0;
    let func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut closure = func.into_fn();
    assert_eq!(closure(&10), 11);
}

// ============================================================================
// ArcStatefulFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_arc_stateful_function_new() {
    // Test ArcStatefulFunction::new with simple closure
    let mut counter = 0;
    let mut func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    assert_eq!(func.apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
    assert_eq!(func.apply(&10), 13);
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
    let mut counter = 0;
    let mut func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&10), 20);
}

#[test]
fn test_arc_stateful_function_clone() {
    // Test ArcStatefulFunction::clone
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut func_clone = func.clone();
    assert_eq!(func_clone.apply(&10), 11);
    assert_eq!(func_clone.apply(&10), 12);
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

#[test]
fn test_arc_stateful_function_compose() {
    // Test ArcStatefulFunction::compose reverse composition
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    });

    let mut composed = func.compose(|x: &i32| x + 1);
    assert_eq!(composed.apply(&10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(&10), 22); // (10 + 1) * 2
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
        *counter_clone.lock().unwrap() += 1;
        x * 2
    })
    .when(|x: &i32| *x > 10)
    .or_else(|x: &i32| x + 1);

    assert_eq!(func.apply(&15), 30); // 15 > 10, apply * 2
    assert_eq!(func.apply(&5), 6); // 5 <= 10, apply + 1
    assert_eq!(*counter.lock().unwrap(), 1); // Only the first call satisfies the condition
}

#[test]
fn test_arc_stateful_function_when_with_predicate() {
    // Test when with ArcPredicate
    let mut counter = 0;
    let mut func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    })
    .when(ArcPredicate::new(|x: &i32| *x > 0))
    .or_else(|x: &i32| -(*x));

    assert_eq!(func.apply(&10), 10); // 10 > 0, apply * 1
    assert_eq!(func.apply(&-5), 5); // -5 <= 0, apply negate
}

// ============================================================================
// ArcStatefulFunction Tests - Type Conversions
// ============================================================================

#[test]
fn test_arc_stateful_function_into_box() {
    // Test ArcStatefulFunction::into_box conversion
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut boxed = func.into_box();
    assert_eq!(boxed.apply(&10), 11);
}

#[test]
fn test_arc_stateful_function_into_rc() {
    // Test ArcStatefulFunction::into_rc conversion
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut rc = func.into_rc();
    assert_eq!(rc.apply(&10), 11);
}

#[test]
fn test_arc_stateful_function_into_arc() {
    // Test ArcStatefulFunction::into_arc (should return itself)
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut arc = func.into_arc();
    assert_eq!(arc.apply(&10), 11);
}

#[test]
fn test_arc_stateful_function_into_fn() {
    // Test ArcStatefulFunction::into_fn conversion
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut closure = func.into_fn();
    assert_eq!(closure(&10), 11);
}

#[test]
fn test_arc_stateful_function_to_box() {
    // Test non-consuming conversion to BoxStatefulFunction
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut boxed = func.to_box();
    assert_eq!(boxed.apply(&10), 11);
    assert_eq!(func.clone().apply(&10), 12);
}

#[test]
fn test_arc_stateful_function_to_rc() {
    // Test non-consuming conversion to RcStatefulFunction
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut rc = func.to_rc();
    assert_eq!(rc.apply(&10), 11);
    assert_eq!(func.clone().apply(&10), 12);
}

#[test]
fn test_arc_stateful_function_to_arc() {
    // Test non-consuming conversion to ArcStatefulFunction (clone)
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut arc = func.to_arc();
    assert_eq!(arc.apply(&10), 11);
    assert_eq!(func.clone().apply(&10), 12);
}

#[test]
fn test_arc_stateful_function_to_fn() {
    // Test non-consuming conversion to closure
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut closure = func.to_fn();
    assert_eq!(closure(&10), 11);
    assert_eq!(func.clone().apply(&10), 12);
}

// ============================================================================
// ArcStatefulFunction Tests - Thread Safety
// ============================================================================

#[test]
fn test_arc_stateful_function_thread_safety() {
    // Test that ArcStatefulFunction is Send + Sync
    let mut counter = 0;
    let func = ArcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut func_clone = func.clone();

    let handle = std::thread::spawn(move || func_clone.apply(&10));

    assert_eq!(handle.join().unwrap(), 11);
}

// ============================================================================
// RcStatefulFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_rc_stateful_function_new() {
    // Test RcStatefulFunction::new with simple closure
    let mut counter = 0;
    let mut func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    assert_eq!(func.apply(&10), 11);
    assert_eq!(func.apply(&10), 12);
    assert_eq!(func.apply(&10), 13);
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
    let mut counter = 0;
    let mut func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&10), 20);
}

#[test]
fn test_rc_stateful_function_clone() {
    // Test RcStatefulFunction::clone
    let mut counter = 0;
    let func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut func_clone = func.clone();
    assert_eq!(func_clone.apply(&10), 11);
    assert_eq!(func_clone.apply(&10), 12);
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

#[test]
fn test_rc_stateful_function_compose() {
    // Test RcStatefulFunction::compose reverse composition
    let mut counter = 0;
    let func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    });

    let mut composed = func.compose(|x: &i32| x + 1);
    assert_eq!(composed.apply(&10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(&10), 22); // (10 + 1) * 2
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

#[test]
fn test_rc_stateful_function_when_with_predicate() {
    // Test when with RcPredicate
    let mut counter = 0;
    let mut func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    })
    .when(RcPredicate::new(|x: &i32| *x > 0))
    .or_else(|x: &i32| -(*x));

    assert_eq!(func.apply(&10), 10); // 10 > 0, apply * 1
    assert_eq!(func.apply(&-5), 5); // -5 <= 0, apply negate
}

// ============================================================================
// RcStatefulFunction Tests - Type Conversions
// ============================================================================

#[test]
fn test_rc_stateful_function_into_box() {
    // Test RcStatefulFunction::into_box conversion
    let mut counter = 0;
    let func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut boxed = func.into_box();
    assert_eq!(boxed.apply(&10), 11);
}

#[test]
fn test_rc_stateful_function_into_rc() {
    // Test RcStatefulFunction::into_rc (should return itself)
    let mut counter = 0;
    let func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut rc = func.into_rc();
    assert_eq!(rc.apply(&10), 11);
}

#[test]
fn test_rc_stateful_function_into_fn() {
    // Test RcStatefulFunction::into_fn conversion
    let mut counter = 0;
    let func = RcStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    let mut closure = func.into_fn();
    assert_eq!(closure(&10), 11);
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_stateful_function_with_zero() {
    // Test stateful function with zero input
    let mut counter = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x + counter
    });
    assert_eq!(func.apply(&0), 1);
    assert_eq!(func.apply(&0), 2);
}

#[test]
fn test_stateful_function_with_negative() {
    // Test stateful function with negative input
    let mut counter = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        counter += 1;
        x * counter
    });
    assert_eq!(func.apply(&-10), -10);
    assert_eq!(func.apply(&-10), -20);
}

#[test]
fn test_stateful_function_accumulator() {
    // Test stateful function as accumulator
    let mut sum = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        sum += *x;
        sum
    });
    assert_eq!(func.apply(&1), 1);
    assert_eq!(func.apply(&2), 3);
    assert_eq!(func.apply(&3), 6);
    assert_eq!(func.apply(&4), 10);
}

#[test]
fn test_stateful_function_with_string() {
    // Test stateful function with String type
    let mut buffer = String::new();
    let mut func = BoxStatefulFunction::new(move |s: &String| {
        buffer.push_str(s);
        buffer.clone()
    });
    assert_eq!(func.apply(&String::from("Hello")), "Hello");
    assert_eq!(func.apply(&String::from(" ")), "Hello ");
    assert_eq!(func.apply(&String::from("World")), "Hello World");
}

#[test]
fn test_stateful_function_with_vec() {
    // Test stateful function with Vec type
    let mut history = Vec::new();
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        history.push(*x);
        history.len()
    });
    assert_eq!(func.apply(&1), 1);
    assert_eq!(func.apply(&2), 2);
    assert_eq!(func.apply(&3), 3);
}

#[test]
fn test_stateful_function_counter() {
    // Test stateful function as simple counter
    let mut count = 0;
    let mut func = BoxStatefulFunction::new(move |_x: &i32| {
        count += 1;
        count
    });
    assert_eq!(func.apply(&0), 1);
    assert_eq!(func.apply(&0), 2);
    assert_eq!(func.apply(&0), 3);
}

#[test]
fn test_stateful_function_toggle() {
    // Test stateful function as toggle
    let mut toggle = false;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        toggle = !toggle;
        if toggle {
            *x
        } else {
            -*x
        }
    });
    assert_eq!(func.apply(&5), 5);
    assert_eq!(func.apply(&5), -5);
    assert_eq!(func.apply(&5), 5);
}

// ============================================================================
// FnStatefulFunctionOps Extension Trait Tests
// ============================================================================

#[test]
fn test_fn_stateful_function_ops_and_then() {
    // Test FnStatefulFunctionOps::and_then for closures
    use prism3_function::FnStatefulFunctionOps;

    let mut counter1 = 0;
    let func1 = move |x: &i32| {
        counter1 += 1;
        x + counter1
    };

    let mut counter2 = 0;
    let func2 = move |x: &i32| {
        counter2 += 1;
        x * counter2
    };

    let mut composed = func1.and_then(func2);
    assert_eq!(composed.apply(&10), 11);
}

#[test]
fn test_fn_stateful_function_ops_compose() {
    // Test FnStatefulFunctionOps::compose for closures
    use prism3_function::FnStatefulFunctionOps;

    let mut counter = 0;
    let func = move |x: &i32| {
        counter += 1;
        x * counter
    };

    let before = move |x: &i32| x + 1;

    let mut composed = func.compose(before);
    assert_eq!(composed.apply(&10), 11);
}

#[test]
fn test_fn_stateful_function_ops_when() {
    // Test FnStatefulFunctionOps::when for closures
    use prism3_function::FnStatefulFunctionOps;

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let func = move |x: &i32| {
        *counter_clone.borrow_mut() += 1;
        x * 2
    };

    let mut conditional = func.when(|x: &i32| *x > 0).or_else(|x: &i32| -(*x));
    assert_eq!(conditional.apply(&5), 10); // 5 > 0, apply * 2
    assert_eq!(conditional.apply(&-5), 5); // -5 <= 0, apply -(*x) = -(-5) = 5
    assert_eq!(*counter.borrow(), 1); // Only the first call satisfies the condition
}

// ============================================================================
// Complex State Management Tests
// ============================================================================

#[test]
fn test_stateful_function_with_multiple_state() {
    // Test stateful function with multiple state variables
    let mut count = 0;
    let mut sum = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        count += 1;
        sum += *x;
        (count, sum)
    });
    assert_eq!(func.apply(&10), (1, 10));
    assert_eq!(func.apply(&20), (2, 30));
    assert_eq!(func.apply(&30), (3, 60));
}

#[test]
fn test_stateful_function_with_option_state() {
    // Test stateful function with Option state
    let mut last_value: Option<i32> = None;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        let result = last_value.unwrap_or(0) + *x;
        last_value = Some(*x);
        result
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&20), 30);
    assert_eq!(func.apply(&30), 50);
}

// ============================================================================
// Custom Struct Tests - StatefulFunction Default Implementation
// ============================================================================

/// Custom struct for testing StatefulFunction trait default implementations
#[derive(Clone)]
struct CustomStatefulFunction {
    multiplier: i32,
}

// Implement Send and Sync for CustomStatefulFunction to support Arc
unsafe impl Send for CustomStatefulFunction {}
unsafe impl Sync for CustomStatefulFunction {}

impl StatefulFunction<i32, i32> for CustomStatefulFunction {
    fn apply(&mut self, input: &i32) -> i32 {
        self.multiplier += 1;
        input * self.multiplier
    }
}

#[test]
fn test_custom_stateful_function_into_box() {
    // Test default implementation of into_box() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut boxed = custom.into_box();
    assert_eq!(boxed.apply(&10), 10); // 10 * 1
    assert_eq!(boxed.apply(&10), 20); // 10 * 2
    assert_eq!(boxed.apply(&10), 30); // 10 * 3
}

#[test]
fn test_custom_stateful_function_into_rc() {
    // Test default implementation of into_rc() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut rc = custom.into_rc();
    assert_eq!(rc.apply(&10), 10); // 10 * 1
    assert_eq!(rc.apply(&10), 20); // 10 * 2
    assert_eq!(rc.apply(&10), 30); // 10 * 3
}

#[test]
fn test_custom_stateful_function_into_arc() {
    // Test default implementation of into_arc() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut arc = custom.into_arc();
    assert_eq!(arc.apply(&10), 10); // 10 * 1
    assert_eq!(arc.apply(&10), 20); // 10 * 2
    assert_eq!(arc.apply(&10), 30); // 10 * 3
}

#[test]
fn test_custom_stateful_function_into_fn() {
    // Test default implementation of into_fn() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut func = custom.into_fn();
    assert_eq!(func(&10), 10); // 10 * 1
    assert_eq!(func(&10), 20); // 10 * 2
    assert_eq!(func(&10), 30); // 10 * 3
}

#[test]
fn test_custom_stateful_function_to_box() {
    // Test default implementation of to_box() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut boxed = custom.to_box();
    assert_eq!(boxed.apply(&10), 10); // 10 * 1
    assert_eq!(boxed.apply(&10), 20); // 10 * 2
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    assert_eq!(custom_clone.apply(&10), 10); // 10 * 1 (independent state)
}

#[test]
fn test_custom_stateful_function_to_rc() {
    // Test default implementation of to_rc() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut rc = custom.to_rc();
    assert_eq!(rc.apply(&10), 10); // 10 * 1
    assert_eq!(rc.apply(&10), 20); // 10 * 2
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    assert_eq!(custom_clone.apply(&10), 10); // 10 * 1 (independent state)
}

#[test]
fn test_custom_stateful_function_to_arc() {
    // Test default implementation of to_arc() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut arc = custom.to_arc();
    assert_eq!(arc.apply(&10), 10); // 10 * 1
    assert_eq!(arc.apply(&10), 20); // 10 * 2
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    assert_eq!(custom_clone.apply(&10), 10); // 10 * 1 (independent state)
}

#[test]
fn test_custom_stateful_function_to_fn() {
    // Test default implementation of to_fn() for custom struct
    let custom = CustomStatefulFunction { multiplier: 0 };
    let mut func = custom.to_fn();
    assert_eq!(func(&10), 10); // 10 * 1
    assert_eq!(func(&10), 20); // 10 * 2
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    assert_eq!(custom_clone.apply(&10), 10); // 10 * 1 (independent state)
}
