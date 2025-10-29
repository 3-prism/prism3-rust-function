/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Comprehensive tests for Function trait and its implementations

use prism3_function::{
    ArcFunction,
    ArcPredicate,
    BoxFunction,
    BoxPredicate,
    Function,
    FunctionOnce,
    Predicate,
    RcFunction,
    RcPredicate,
};

// ============================================================================
// Function Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_function_trait_apply() {
    // Test that Function trait's apply method works correctly
    let double = |x: &i32| x * 2;
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-10), -20);
}

#[test]
fn test_function_trait_into_box() {
    // Test conversion from closure to BoxFunction
    let double = |x: &i32| x * 2;
    let boxed = double.into_box();
    assert_eq!(boxed.apply(&21), 42);
}

#[test]
fn test_function_trait_into_rc() {
    // Test conversion from closure to RcFunction
    let double = |x: &i32| x * 2;
    let rc = double.into_rc();
    assert_eq!(rc.apply(&21), 42);
}

#[test]
fn test_function_trait_into_arc() {
    // Test conversion from closure to ArcFunction
    let double = |x: &i32| x * 2;
    let arc = double.into_arc();
    assert_eq!(arc.apply(&21), 42);
}

#[test]
fn test_function_trait_into_fn() {
    // Test conversion to closure
    let double = |x: &i32| x * 2;
    let func = double.into_fn();
    assert_eq!(func(&21), 42);
}

#[test]
fn test_function_trait_to_box() {
    // Test non-consuming conversion to BoxFunction
    let double = |x: &i32| x * 2;
    let boxed = double.to_box();
    assert_eq!(boxed.apply(&21), 42);
    // Original closure still usable
    assert_eq!(double.apply(&10), 20);
}

#[test]
fn test_function_trait_to_rc() {
    // Test non-consuming conversion to RcFunction
    let double = |x: &i32| x * 2;
    let rc = double.to_rc();
    assert_eq!(rc.apply(&21), 42);
    // Original closure still usable
    assert_eq!(double.apply(&10), 20);
}

#[test]
fn test_function_trait_to_arc() {
    // Test non-consuming conversion to ArcFunction
    let double = |x: &i32| x * 2;
    let arc = double.to_arc();
    assert_eq!(arc.apply(&21), 42);
    // Original closure still usable
    assert_eq!(double.apply(&10), 20);
}

#[test]
fn test_function_trait_to_fn() {
    // Test non-consuming conversion to closure
    let double = |x: &i32| x * 2;
    let func = double.to_fn();
    assert_eq!(func(&21), 42);
    // Original closure still usable
    assert_eq!(double.apply(&10), 20);
}

// ============================================================================
// BoxFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_box_function_new() {
    // Test BoxFunction::new with simple closure
    let double = BoxFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-5), -10);
}

#[test]
fn test_box_function_identity() {
    // Test BoxFunction::identity
    let identity = BoxFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_box_function_constant() {
    // Test BoxFunction::constant
    let constant = BoxFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
    assert_eq!(constant.apply(&0), "hello");
}

#[test]
fn test_box_function_apply() {
    // Test Function trait implementation for BoxFunction
    let add_one = BoxFunction::new(|x: &i32| x + 1);
    assert_eq!(add_one.apply(&41), 42);
    assert_eq!(add_one.apply(&0), 1);
    assert_eq!(add_one.apply(&-1), 0);
}

// ============================================================================
// BoxFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_box_function_and_then() {
    // Test BoxFunction::and_then composition
    let double = BoxFunction::new(|x: &i32| x * 2);
    let to_string = BoxFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string);
    assert_eq!(composed.apply(&21), "42");
    assert_eq!(composed.apply(&0), "0");
    assert_eq!(composed.apply(&-5), "-10");
}

#[test]
fn test_box_function_and_then_with_closure() {
    // Test and_then with closure
    let double = BoxFunction::new(|x: &i32| x * 2);
    let composed = double.and_then(|x: &i32| x + 10);
    assert_eq!(composed.apply(&16), 42);
}

#[test]
fn test_box_function_compose() {
    // Test BoxFunction::compose reverse composition
    let double = BoxFunction::new(|x: &i32| x * 2);
    let add_one = BoxFunction::new(|x: &i32| x + 1);
    let composed = double.compose(add_one);
    assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
    assert_eq!(composed.apply(&0), 2); // (0 + 1) * 2
    assert_eq!(composed.apply(&-1), 0); // (-1 + 1) * 2
}

#[test]
fn test_box_function_compose_with_closure() {
    // Test compose with closure
    let double = BoxFunction::new(|x: &i32| x * 2);
    let composed = double.compose(|x: &i32| x + 1);
    assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
}

// ============================================================================
// BoxFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_box_function_when_or_else() {
    // Test conditional execution with when/or_else
    let double = BoxFunction::new(|x: &i32| x * 2);
    let identity = BoxFunction::<i32, i32>::identity();
    let conditional = double.when(|x: &i32| *x > 0).or_else(identity);

    assert_eq!(conditional.apply(&5), 10); // when branch
    assert_eq!(conditional.apply(&-5), -5); // or_else branch
    assert_eq!(conditional.apply(&0), 0); // or_else branch
}

#[test]
fn test_box_function_when_with_closure() {
    // Test when with closure predicate and or_else
    let double = BoxFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x >= 10).or_else(|x: &i32| *x);

    assert_eq!(conditional.apply(&15), 30); // when branch
    assert_eq!(conditional.apply(&5), 5); // or_else branch
}

#[test]
fn test_box_function_when_with_predicate() {
    // Test when with BoxPredicate
    let double = BoxFunction::new(|x: &i32| x * 2);
    let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    let conditional = double.when(is_positive).or_else(|x: &i32| -(*x));

    assert_eq!(conditional.apply(&5), 10); // when branch
    assert_eq!(conditional.apply(&-5), 5); // or_else branch
}

// ============================================================================
// BoxFunction Tests - Type Conversions (Function trait)
// ============================================================================

#[test]
fn test_box_function_into_box() {
    // Test BoxFunction::into_box (should return itself)
    let double = BoxFunction::new(|x: &i32| x * 2);
    let boxed = double.into_box();
    assert_eq!(boxed.apply(&21), 42);
}

#[test]
fn test_box_function_into_rc() {
    // Test BoxFunction::into_rc conversion
    let double = BoxFunction::new(|x: &i32| x * 2);
    let rc = double.into_rc();
    assert_eq!(rc.apply(&21), 42);
}

#[test]
fn test_box_function_into_fn() {
    // Test BoxFunction::into_fn conversion
    let double = BoxFunction::new(|x: &i32| x * 2);
    let func = double.into_fn();
    assert_eq!(func(&21), 42);
}

// ============================================================================
// BoxFunction Tests - FunctionOnce Implementation
// ============================================================================

#[test]
fn test_box_function_apply_once() {
    // Test FunctionOnce::apply_once for BoxFunction
    let double = BoxFunction::new(|x: &i32| x * 2);
    let result = double.apply_once(&21);
    assert_eq!(result, 42);
}

#[test]
fn test_box_function_into_box_once() {
    // Test FunctionOnce::into_box_once
    let double = BoxFunction::new(|x: &i32| x * 2);
    let boxed_once = double.into_box_once();
    assert_eq!(boxed_once.apply_once(&21), 42);
}

#[test]
fn test_box_function_into_fn_once() {
    // Test FunctionOnce::into_fn_once
    let double = BoxFunction::new(|x: &i32| x * 2);
    let func_once = double.into_fn_once();
    assert_eq!(func_once(&21), 42);
}

// Note: BoxFunction doesn't implement Clone, so to_box_once() and
// to_fn_once() are not available for BoxFunction. These tests are
// intentionally commented out.

// #[test]
// fn test_box_function_to_box_once() {
//     // BoxFunction doesn't implement Clone, so to_box_once() is not available
//     let double = BoxFunction::new(|x: &i32| x * 2);
//     let boxed_once = double.to_box_once();
//     assert_eq!(boxed_once.apply_once(&21), 42);
// }

// #[test]
// fn test_box_function_to_fn_once() {
//     // BoxFunction doesn't implement Clone, so to_fn_once() is not available
//     let double = BoxFunction::new(|x: &i32| x * 2);
//     let func_once = double.to_fn_once();
//     assert_eq!(func_once(&21), 42);
// }

// ============================================================================
// ArcFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_arc_function_new() {
    // Test ArcFunction::new with simple closure
    let double = ArcFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-5), -10);
}

#[test]
fn test_arc_function_identity() {
    // Test ArcFunction::identity
    let identity = ArcFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_arc_function_constant() {
    // Test ArcFunction::constant
    let constant = ArcFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
}

#[test]
fn test_arc_function_apply() {
    // Test Function trait implementation for ArcFunction
    let add_one = ArcFunction::new(|x: &i32| x + 1);
    assert_eq!(add_one.apply(&41), 42);
    assert_eq!(add_one.apply(&0), 1);
}

#[test]
fn test_arc_function_clone() {
    // Test ArcFunction::clone
    let double = ArcFunction::new(|x: &i32| x * 2);
    let cloned = double.clone();
    assert_eq!(double.apply(&21), 42);
    assert_eq!(cloned.apply(&21), 42);
}

// ============================================================================
// ArcFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_arc_function_and_then() {
    // Test ArcFunction::and_then composition
    let double = ArcFunction::new(|x: &i32| x * 2);
    let to_string = ArcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string);
    assert_eq!(composed.apply(&21), "42");
    // Original still usable
    assert_eq!(double.apply(&10), 20);
}

#[test]
fn test_arc_function_and_then_with_clone() {
    // Test and_then preserving original with clone
    let double = ArcFunction::new(|x: &i32| x * 2);
    let to_string = ArcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string.clone());
    assert_eq!(composed.apply(&21), "42");
    assert_eq!(to_string.apply(&5), "5");
}

#[test]
fn test_arc_function_compose() {
    // Test ArcFunction::compose reverse composition
    let double = ArcFunction::new(|x: &i32| x * 2);
    let add_one = ArcFunction::new(|x: &i32| x + 1);
    let composed = double.compose(add_one);
    assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
                                        // Original still usable
    assert_eq!(double.apply(&10), 20);
}

// ============================================================================
// ArcFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_arc_function_when_or_else() {
    // Test conditional execution with when/or_else
    let double = ArcFunction::new(|x: &i32| x * 2);
    let identity = ArcFunction::<i32, i32>::identity();
    let conditional = double.when(|x: &i32| *x > 0).or_else(identity);

    let conditional_clone = conditional.clone();
    assert_eq!(conditional.apply(&5), 10);
    assert_eq!(conditional_clone.apply(&-5), -5);
}

#[test]
fn test_arc_function_when_with_predicate() {
    // Test when with ArcPredicate
    let double = ArcFunction::new(|x: &i32| x * 2);
    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let conditional = double
        .when(is_positive.clone())
        .or_else(ArcFunction::identity());

    assert_eq!(conditional.apply(&5), 10);
    assert!(is_positive.test(&3));
}

// ============================================================================
// ArcFunction Tests - Type Conversions
// ============================================================================

#[test]
fn test_arc_function_into_box() {
    // Test ArcFunction::into_box conversion
    let double = ArcFunction::new(|x: &i32| x * 2);
    let boxed = double.into_box();
    assert_eq!(boxed.apply(&21), 42);
}

#[test]
fn test_arc_function_into_rc() {
    // Test ArcFunction::into_rc conversion
    let double = ArcFunction::new(|x: &i32| x * 2);
    let rc = double.into_rc();
    assert_eq!(rc.apply(&21), 42);
}

#[test]
fn test_arc_function_into_arc() {
    // Test ArcFunction::into_arc (should return itself)
    let double = ArcFunction::new(|x: &i32| x * 2);
    let arc = double.into_arc();
    assert_eq!(arc.apply(&21), 42);
}

#[test]
fn test_arc_function_into_fn() {
    // Test ArcFunction::into_fn conversion
    let double = ArcFunction::new(|x: &i32| x * 2);
    let func = double.into_fn();
    assert_eq!(func(&21), 42);
}

#[test]
fn test_arc_function_to_box() {
    // Test non-consuming conversion to BoxFunction
    let double = ArcFunction::new(|x: &i32| x * 2);
    let boxed = double.to_box();
    assert_eq!(boxed.apply(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_arc_function_to_rc() {
    // Test non-consuming conversion to RcFunction
    let double = ArcFunction::new(|x: &i32| x * 2);
    let rc = double.to_rc();
    assert_eq!(rc.apply(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_arc_function_to_arc() {
    // Test non-consuming conversion to ArcFunction (clone)
    let double = ArcFunction::new(|x: &i32| x * 2);
    let arc = double.to_arc();
    assert_eq!(arc.apply(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_arc_function_to_fn() {
    // Test non-consuming conversion to closure
    let double = ArcFunction::new(|x: &i32| x * 2);
    let func = double.to_fn();
    assert_eq!(func(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

// ============================================================================
// ArcFunction Tests - FunctionOnce Implementation
// ============================================================================

#[test]
fn test_arc_function_apply_once() {
    // Test FunctionOnce::apply_once for ArcFunction
    let double = ArcFunction::new(|x: &i32| x * 2);
    let result = double.apply_once(&21);
    assert_eq!(result, 42);
}

#[test]
fn test_arc_function_into_box_once() {
    // Test FunctionOnce::into_box_once
    let double = ArcFunction::new(|x: &i32| x * 2);
    let boxed_once = double.into_box_once();
    assert_eq!(boxed_once.apply_once(&21), 42);
}

#[test]
fn test_arc_function_into_fn_once() {
    // Test FunctionOnce::into_fn_once
    let double = ArcFunction::new(|x: &i32| x * 2);
    let func_once = double.into_fn_once();
    assert_eq!(func_once(&21), 42);
}

#[test]
fn test_arc_function_to_box_once() {
    // Test FunctionOnce::to_box_once (non-consuming)
    let double = ArcFunction::new(|x: &i32| x * 2);
    let boxed_once = double.to_box_once();
    assert_eq!(boxed_once.apply_once(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_arc_function_to_fn_once() {
    // Test FunctionOnce::to_fn_once (non-consuming)
    let double = ArcFunction::new(|x: &i32| x * 2);
    let func_once = double.to_fn_once();
    assert_eq!(func_once(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

// ============================================================================
// ArcFunction Tests - Thread Safety
// ============================================================================

#[test]
fn test_arc_function_send_sync() {
    // Test that ArcFunction is Send + Sync
    let double = ArcFunction::new(|x: &i32| x * 2);
    let double_clone = double.clone();

    let handle = std::thread::spawn(move || double_clone.apply(&21));

    assert_eq!(handle.join().unwrap(), 42);
    assert_eq!(double.apply(&10), 20);
}

// ============================================================================
// RcFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_rc_function_new() {
    // Test RcFunction::new with simple closure
    let double = RcFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-5), -10);
}

#[test]
fn test_rc_function_identity() {
    // Test RcFunction::identity
    let identity = RcFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_rc_function_constant() {
    // Test RcFunction::constant
    let constant = RcFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
}

#[test]
fn test_rc_function_apply() {
    // Test Function trait implementation for RcFunction
    let add_one = RcFunction::new(|x: &i32| x + 1);
    assert_eq!(add_one.apply(&41), 42);
    assert_eq!(add_one.apply(&0), 1);
}

#[test]
fn test_rc_function_clone() {
    // Test RcFunction::clone
    let double = RcFunction::new(|x: &i32| x * 2);
    let cloned = double.clone();
    assert_eq!(double.apply(&21), 42);
    assert_eq!(cloned.apply(&21), 42);
}

// ============================================================================
// RcFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_rc_function_and_then() {
    // Test RcFunction::and_then composition
    let double = RcFunction::new(|x: &i32| x * 2);
    let to_string = RcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string);
    assert_eq!(composed.apply(&21), "42");
    // Original still usable
    assert_eq!(double.apply(&10), 20);
}

#[test]
fn test_rc_function_and_then_with_clone() {
    // Test and_then preserving original with clone
    let double = RcFunction::new(|x: &i32| x * 2);
    let to_string = RcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string.clone());
    assert_eq!(composed.apply(&21), "42");
    assert_eq!(to_string.apply(&5), "5");
}

#[test]
fn test_rc_function_compose() {
    // Test RcFunction::compose reverse composition
    let double = RcFunction::new(|x: &i32| x * 2);
    let add_one = RcFunction::new(|x: &i32| x + 1);
    let composed = double.compose(add_one);
    assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
                                        // Original still usable
    assert_eq!(double.apply(&10), 20);
}

// ============================================================================
// RcFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_rc_function_when_or_else() {
    // Test conditional execution with when/or_else
    let double = RcFunction::new(|x: &i32| x * 2);
    let identity = RcFunction::<i32, i32>::identity();
    let conditional = double.when(|x: &i32| *x > 0).or_else(identity);

    let conditional_clone = conditional.clone();
    assert_eq!(conditional.apply(&5), 10);
    assert_eq!(conditional_clone.apply(&-5), -5);
}

#[test]
fn test_rc_function_when_with_predicate() {
    // Test when with RcPredicate
    let double = RcFunction::new(|x: &i32| x * 2);
    let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    let conditional = double
        .when(is_positive.clone())
        .or_else(RcFunction::<i32, i32>::identity());

    assert_eq!(conditional.apply(&5), 10);
    assert!(is_positive.test(&3));
}

// ============================================================================
// RcFunction Tests - Type Conversions
// ============================================================================

#[test]
fn test_rc_function_into_box() {
    // Test RcFunction::into_box conversion
    let double = RcFunction::new(|x: &i32| x * 2);
    let boxed = double.into_box();
    assert_eq!(boxed.apply(&21), 42);
}

#[test]
fn test_rc_function_into_rc() {
    // Test RcFunction::into_rc (should return itself)
    let double = RcFunction::new(|x: &i32| x * 2);
    let rc = double.into_rc();
    assert_eq!(rc.apply(&21), 42);
}

#[test]
fn test_rc_function_into_fn() {
    // Test RcFunction::into_fn conversion
    let double = RcFunction::new(|x: &i32| x * 2);
    let func = double.into_fn();
    assert_eq!(func(&21), 42);
}

#[test]
fn test_rc_function_to_box() {
    // Test non-consuming conversion to BoxFunction
    let double = RcFunction::new(|x: &i32| x * 2);
    let boxed = double.to_box();
    assert_eq!(boxed.apply(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_rc_function_to_rc() {
    // Test non-consuming conversion to RcFunction (clone)
    let double = RcFunction::new(|x: &i32| x * 2);
    let rc = double.to_rc();
    assert_eq!(rc.apply(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_rc_function_to_fn() {
    // Test non-consuming conversion to closure
    let double = RcFunction::new(|x: &i32| x * 2);
    let func = double.to_fn();
    assert_eq!(func(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

// ============================================================================
// RcFunction Tests - FunctionOnce Implementation
// ============================================================================

#[test]
fn test_rc_function_apply_once() {
    // Test FunctionOnce::apply_once for RcFunction
    let double = RcFunction::new(|x: &i32| x * 2);
    let result = double.apply_once(&21);
    assert_eq!(result, 42);
}

#[test]
fn test_rc_function_into_box_once() {
    // Test FunctionOnce::into_box_once
    let double = RcFunction::new(|x: &i32| x * 2);
    let boxed_once = double.into_box_once();
    assert_eq!(boxed_once.apply_once(&21), 42);
}

#[test]
fn test_rc_function_into_fn_once() {
    // Test FunctionOnce::into_fn_once
    let double = RcFunction::new(|x: &i32| x * 2);
    let func_once = double.into_fn_once();
    assert_eq!(func_once(&21), 42);
}

#[test]
fn test_rc_function_to_box_once() {
    // Test FunctionOnce::to_box_once (non-consuming)
    let double = RcFunction::new(|x: &i32| x * 2);
    let boxed_once = double.to_box_once();
    assert_eq!(boxed_once.apply_once(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_rc_function_to_fn_once() {
    // Test FunctionOnce::to_fn_once (non-consuming)
    let double = RcFunction::new(|x: &i32| x * 2);
    let func_once = double.to_fn_once();
    assert_eq!(func_once(&21), 42);
    assert_eq!(double.apply(&21), 42);
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_function_with_zero() {
    // Test functions with zero input
    let double = BoxFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&0), 0);
}

#[test]
fn test_function_with_negative() {
    // Test functions with negative input
    let double = BoxFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&-42), -84);
}

#[test]
fn test_function_with_max_value() {
    // Test functions with maximum value
    let identity = BoxFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&i32::MAX), i32::MAX);
}

#[test]
fn test_function_with_min_value() {
    // Test functions with minimum value
    let identity = BoxFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&i32::MIN), i32::MIN);
}

#[test]
fn test_function_chain_multiple() {
    // Test chaining multiple functions
    let add_one = BoxFunction::new(|x: &i32| x + 1);
    let double = BoxFunction::new(|x: &i32| x * 2);
    let add_ten = BoxFunction::new(|x: &i32| x + 10);

    let composed = add_one.and_then(double).and_then(add_ten);
    assert_eq!(composed.apply(&5), 22); // ((5 + 1) * 2) + 10
}

#[test]
fn test_function_with_string() {
    // Test functions with String type
    let to_upper = BoxFunction::new(|s: &String| s.to_uppercase());
    let input = String::from("hello");
    assert_eq!(to_upper.apply(&input), "HELLO");
}

#[test]
fn test_function_with_vec() {
    // Test functions with Vec type
    let get_len = BoxFunction::new(|v: &Vec<i32>| v.len());
    let vec = vec![1, 2, 3, 4, 5];
    assert_eq!(get_len.apply(&vec), 5);
}

#[test]
fn test_function_with_option() {
    // Test functions with Option type
    let unwrap_or_zero = BoxFunction::new(|opt: &Option<i32>| opt.unwrap_or(0));
    assert_eq!(unwrap_or_zero.apply(&Some(42)), 42);
    assert_eq!(unwrap_or_zero.apply(&None), 0);
}

#[test]
fn test_conditional_function_edge_cases() {
    // Test conditional function with boundary values
    let double = BoxFunction::new(|x: &i32| x * 2);
    let negate = BoxFunction::new(|x: &i32| -(*x));
    let conditional = double.when(|x: &i32| *x >= 0).or_else(negate);

    assert_eq!(conditional.apply(&0), 0); // Boundary: zero
    assert_eq!(conditional.apply(&1), 2); // Positive
    assert_eq!(conditional.apply(&-1), 1); // Negative
}

// ============================================================================
// FnFunctionOps Extension Trait Tests
// ============================================================================

#[test]
fn test_fn_function_ops_and_then() {
    // Test FnFunctionOps::and_then for closures
    use prism3_function::FnFunctionOps;

    let double = |x: &i32| x * 2;
    let to_string = |x: &i32| x.to_string();
    let composed = double.and_then(to_string);
    assert_eq!(composed.apply(&21), "42");
}

#[test]
fn test_fn_function_ops_compose() {
    // Test FnFunctionOps::compose for closures
    use prism3_function::FnFunctionOps;

    let double = |x: &i32| x * 2;
    let add_one = |x: &i32| x + 1;
    let composed = double.compose(add_one);
    assert_eq!(composed.apply(&5), 12); // (5 + 1) * 2
}

#[test]
fn test_fn_function_ops_when() {
    // Test FnFunctionOps::when for closures
    use prism3_function::FnFunctionOps;

    let double = |x: &i32| x * 2;
    let conditional = double.when(|x: &i32| *x > 0).or_else(|x: &i32| -(*x));
    assert_eq!(conditional.apply(&5), 10);
    assert_eq!(conditional.apply(&-5), 5);
}

// ============================================================================
// Function Trait Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod function_default_impl_tests {
    use prism3_function::{
        ArcFunction,
        BoxFunction,
        Function,
    };

    /// Custom struct that only implements the core apply method of Function trait
    /// All into_xxx() and to_xxx() methods use default implementation
    struct CustomFunction {
        multiplier: i32,
    }

    impl Function<i32, i32> for CustomFunction {
        fn apply(&self, input: &i32) -> i32 {
            input * self.multiplier
        }
        // 不覆盖任何 into_xxx() 或 to_xxx() 方法，测试默认实现
    }

    /// 可克隆的自定义函数，用于测试 to_xxx() 方法
    #[derive(Clone)]
    struct CloneableCustomFunction {
        multiplier: i32,
    }

    impl Function<i32, i32> for CloneableCustomFunction {
        fn apply(&self, input: &i32) -> i32 {
            input * self.multiplier
        }
        // 不覆盖任何 into_xxx() 或 to_xxx() 方法，测试默认实现
    }

    #[test]
    fn test_custom_into_box() {
        let custom = CustomFunction { multiplier: 3 };
        let boxed = custom.into_box();

        assert_eq!(boxed.apply(&7), 21);
        assert_eq!(boxed.apply(&10), 30);
    }

    #[test]
    fn test_custom_into_rc() {
        let custom = CustomFunction { multiplier: 5 };
        let rc = custom.into_rc();

        assert_eq!(rc.apply(&4), 20);
        assert_eq!(rc.apply(&6), 30);

        // 测试克隆
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(&2), 10);
    }

    #[test]
    fn test_custom_into_arc() {
        let custom = CustomFunction { multiplier: 7 };
        let arc = custom.into_arc();

        assert_eq!(arc.apply(&3), 21);
        assert_eq!(arc.apply(&5), 35);

        // 测试克隆
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.apply(&2), 14);
    }

    #[test]
    fn test_custom_into_fn() {
        let custom = CustomFunction { multiplier: 4 };
        let func = custom.into_fn();

        assert_eq!(func(&5), 20);
        assert_eq!(func(&10), 40);
    }

    #[test]
    fn test_cloneable_to_box() {
        let custom = CloneableCustomFunction { multiplier: 3 };
        let boxed = custom.to_box();

        assert_eq!(boxed.apply(&7), 21);

        // 原始函数仍然可用
        assert_eq!(custom.apply(&10), 30);
    }

    #[test]
    fn test_cloneable_to_rc() {
        let custom = CloneableCustomFunction { multiplier: 5 };
        let rc = custom.to_rc();

        assert_eq!(rc.apply(&4), 20);

        // 原始函数仍然可用
        assert_eq!(custom.apply(&6), 30);
    }

    #[test]
    fn test_cloneable_to_arc() {
        let custom = CloneableCustomFunction { multiplier: 7 };
        let arc = custom.to_arc();

        assert_eq!(arc.apply(&3), 21);

        // 原始函数仍然可用
        assert_eq!(custom.apply(&5), 35);
    }

    #[test]
    fn test_cloneable_to_fn() {
        let custom = CloneableCustomFunction { multiplier: 4 };
        let func = custom.to_fn();

        assert_eq!(func(&5), 20);

        // 原始函数仍然可用
        assert_eq!(custom.apply(&10), 40);
    }

    #[test]
    fn test_custom_chained_conversions() {
        let custom1 = CustomFunction { multiplier: 2 };
        let custom2 = CustomFunction { multiplier: 3 };

        // 测试 into_box -> into_rc 链式转换
        let boxed: BoxFunction<i32, i32> = custom1.into_box();
        let rc = boxed.into_rc();
        assert_eq!(rc.apply(&21), 42);

        // 测试 into_arc 直接转换
        let arc: ArcFunction<i32, i32> = custom2.into_arc();
        assert_eq!(arc.apply(&14), 42);
    }

    #[test]
    fn test_custom_composition() {
        let custom1 = CloneableCustomFunction { multiplier: 2 };
        let custom2 = CloneableCustomFunction { multiplier: 3 };

        let composed = custom1.to_box().and_then(custom2.to_box());
        assert_eq!(composed.apply(&7), 42); // 7 * 2 = 14, 14 * 3 = 42
    }
}

// ============================================================================
// ArcConditionalFunction Clone Tests
// ============================================================================

#[test]
fn test_arc_conditional_function_clone() {
    let double = ArcFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    // 克隆条件函数
    let conditional_clone = conditional.clone();

    // 两个克隆的条件函数都能正常工作
    let result1 = conditional.or_else(|x: &i32| -(*x));
    let result2 = conditional_clone.or_else(|x: &i32| x + 100);

    assert_eq!(result1.apply(&5), 10); // 条件满足: 5 * 2
    assert_eq!(result1.apply(&-5), 5); // 条件不满足: -(-5)
    assert_eq!(result2.apply(&5), 10); // 条件满足: 5 * 2
    assert_eq!(result2.apply(&-5), 95); // 条件不满足: -5 + 100
}

#[test]
fn test_arc_conditional_function_clone_multiple() {
    let triple = ArcFunction::new(|x: &i32| x * 3);
    let conditional = triple.when(|x: &i32| *x % 2 == 0);

    // 创建多个克隆
    let clone1 = conditional.clone();
    let clone2 = conditional.clone();
    let clone3 = conditional.clone();

    let result1 = conditional.or_else(|x: &i32| *x);
    let result2 = clone1.or_else(|x: &i32| *x);
    let result3 = clone2.or_else(|x: &i32| *x);
    let result4 = clone3.or_else(|x: &i32| *x);

    // 所有克隆都能正常工作
    assert_eq!(result1.apply(&4), 12); // 偶数: 4 * 3
    assert_eq!(result2.apply(&4), 12);
    assert_eq!(result3.apply(&4), 12);
    assert_eq!(result4.apply(&4), 12);

    assert_eq!(result1.apply(&5), 5); // 奇数: 5
    assert_eq!(result2.apply(&5), 5);
    assert_eq!(result3.apply(&5), 5);
    assert_eq!(result4.apply(&5), 5);
}

// ============================================================================
// RcConditionalFunction Clone Tests
// ============================================================================

#[test]
fn test_rc_conditional_function_clone() {
    let double = RcFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    // 克隆条件函数
    let conditional_clone = conditional.clone();

    // 两个克隆的条件函数都能正常工作
    let result1 = conditional.or_else(|x: &i32| -(*x));
    let result2 = conditional_clone.or_else(|x: &i32| x + 100);

    assert_eq!(result1.apply(&5), 10); // 条件满足: 5 * 2
    assert_eq!(result1.apply(&-5), 5); // 条件不满足: -(-5)
    assert_eq!(result2.apply(&5), 10); // 条件满足: 5 * 2
    assert_eq!(result2.apply(&-5), 95); // 条件不满足: -5 + 100
}

#[test]
fn test_rc_conditional_function_clone_multiple() {
    let triple = RcFunction::new(|x: &i32| x * 3);
    let conditional = triple.when(|x: &i32| *x % 2 == 0);

    // 创建多个克隆
    let clone1 = conditional.clone();
    let clone2 = conditional.clone();
    let clone3 = conditional.clone();

    let result1 = conditional.or_else(|x: &i32| *x);
    let result2 = clone1.or_else(|x: &i32| *x);
    let result3 = clone2.or_else(|x: &i32| *x);
    let result4 = clone3.or_else(|x: &i32| *x);

    // 所有克隆都能正常工作
    assert_eq!(result1.apply(&4), 12); // 偶数: 4 * 3
    assert_eq!(result2.apply(&4), 12);
    assert_eq!(result3.apply(&4), 12);
    assert_eq!(result4.apply(&4), 12);

    assert_eq!(result1.apply(&5), 5); // 奇数: 5
    assert_eq!(result2.apply(&5), 5);
    assert_eq!(result3.apply(&5), 5);
    assert_eq!(result4.apply(&5), 5);
}
