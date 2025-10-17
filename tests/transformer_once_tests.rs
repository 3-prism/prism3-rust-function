/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for TransformerOnce types

use prism3_function::{
    ArcTransformerOnce, BoxTransformerOnce, FunctionOnce, RcTransformerOnce, TransformerOnce,
};

// ============================================================================
// BoxTransformerOnce Tests
// ============================================================================

#[test]
fn test_box_transformer_once_basic() {
    let parse = BoxTransformerOnce::new(|s: String| s.trim().to_uppercase());

    assert_eq!(parse.transform("  hello  ".to_string()), "HELLO");
}

#[test]
fn test_box_transformer_once_identity() {
    let identity = BoxTransformerOnce::<i32>::identity();
    assert_eq!(identity.transform(42), 42);
}

#[test]
fn test_box_transformer_once_constant() {
    let constant = BoxTransformerOnce::constant(42);
    assert_eq!(constant.transform(123), 42);
}

#[test]
fn test_box_transformer_once_and_then() {
    let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
    let double = |x: i32| x * 2;
    let composed = add_one.and_then(double);
    assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_box_transformer_once_compose() {
    let double = BoxTransformerOnce::new(|x: i32| x * 2);
    let add_one = |x: i32| x + 1;
    let composed = double.compose(add_one);
    assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_box_transformer_once_with_move() {
    let value = String::from("hello");
    let to_upper = BoxTransformerOnce::new(|s: String| s.to_uppercase());
    assert_eq!(to_upper.transform(value), "HELLO");
    // value has been moved, can't use it anymore
}

#[test]
fn test_box_transformer_once_chain() {
    let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
    let double = |x: i32| x * 2;
    let subtract_three = |x: i32| x - 3;

    let composed = add_one.and_then(double).and_then(subtract_three);
    assert_eq!(composed.transform(5), 9); // ((5 + 1) * 2) - 3 = 9
}

#[test]
fn test_box_transformer_once_as_function_once() {
    let double = BoxTransformerOnce::new(|x: i32| x * 2);
    assert_eq!(double.apply(21), 42);
}

#[test]
fn test_box_transformer_once_into_fn() {
    let double = BoxTransformerOnce::new(|x: i32| x * 2);
    let mut closure = double.into_fn();
    assert_eq!(closure(21), 42);
}

// ============================================================================
// ArcTransformerOnce Tests
// ============================================================================

#[test]
fn test_arc_transformer_once_basic() {
    let parse = ArcTransformerOnce::new(|s: String| s.trim().to_uppercase());

    assert_eq!(parse.transform("  hello  ".to_string()), "HELLO");
}

#[test]
fn test_arc_transformer_once_clone() {
    let double = ArcTransformerOnce::new(|x: i32| x * 2);
    let cloned = double.clone();

    assert_eq!(double.transform(21), 42);
    assert_eq!(cloned.transform(21), 42);
}

#[test]
fn test_arc_transformer_once_identity() {
    let identity = ArcTransformerOnce::<i32>::identity();
    assert_eq!(identity.transform(42), 42);
}

#[test]
fn test_arc_transformer_once_constant() {
    let constant = ArcTransformerOnce::constant(42);
    assert_eq!(constant.transform(123), 42);
}

#[test]
fn test_arc_transformer_once_and_then() {
    let double = ArcTransformerOnce::new(|x: i32| x * 2);
    let add_one = ArcTransformerOnce::new(|x: i32| x + 1);
    let composed = double.and_then(&add_one);

    // Both original and composed can be used
    assert_eq!(double.transform(21), 42);
    assert_eq!(composed.transform(5), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_arc_transformer_once_compose() {
    let double = ArcTransformerOnce::new(|x: i32| x * 2);
    let add_one = ArcTransformerOnce::new(|x: i32| x + 1);
    let composed = double.compose(&add_one);

    assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_arc_transformer_once_chain() {
    let add_one = ArcTransformerOnce::new(|x: i32| x + 1);
    let double = ArcTransformerOnce::new(|x: i32| x * 2);
    let subtract_three = ArcTransformerOnce::new(|x: i32| x - 3);

    let composed = add_one.and_then(&double).and_then(&subtract_three);

    // Original transformers still usable
    assert_eq!(add_one.transform(5), 6);
    assert_eq!(composed.transform(5), 9); // ((5 + 1) * 2) - 3 = 9
}

#[test]
fn test_arc_transformer_once_with_string() {
    let to_upper = ArcTransformerOnce::new(|s: String| s.to_uppercase());
    let input = String::from("hello");
    assert_eq!(to_upper.transform(input), "HELLO");
}

#[test]
fn test_arc_transformer_once_thread_safety() {
    use std::sync::Arc as StdArc;
    use std::thread;

    let transformer = ArcTransformerOnce::new(|x: i32| x * 2);
    let transformer_arc = StdArc::new(transformer);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let t = StdArc::clone(&transformer_arc);
            thread::spawn(move || t.as_ref().clone().transform(i))
        })
        .collect();

    let results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(results, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
}

#[test]
fn test_arc_transformer_once_as_function_once() {
    let double = ArcTransformerOnce::new(|x: i32| x * 2);
    assert_eq!(double.apply(21), 42);
}

#[test]
fn test_arc_transformer_once_into_fn() {
    let double = ArcTransformerOnce::new(|x: i32| x * 2);
    let mut closure = double.into_fn();
    assert_eq!(closure(21), 42);
    assert_eq!(closure(10), 20);
}

// ============================================================================
// RcTransformerOnce Tests
// ============================================================================

#[test]
fn test_rc_transformer_once_basic() {
    let parse = RcTransformerOnce::new(|s: String| s.trim().to_uppercase());

    assert_eq!(parse.transform("  hello  ".to_string()), "HELLO");
}

#[test]
fn test_rc_transformer_once_clone() {
    let double = RcTransformerOnce::new(|x: i32| x * 2);
    let cloned = double.clone();

    assert_eq!(double.transform(21), 42);
    assert_eq!(cloned.transform(21), 42);
}

#[test]
fn test_rc_transformer_once_identity() {
    let identity = RcTransformerOnce::<i32>::identity();
    assert_eq!(identity.transform(42), 42);
}

#[test]
fn test_rc_transformer_once_constant() {
    let constant = RcTransformerOnce::constant(42);
    assert_eq!(constant.transform(123), 42);
}

#[test]
fn test_rc_transformer_once_and_then() {
    let double = RcTransformerOnce::new(|x: i32| x * 2);
    let add_one = RcTransformerOnce::new(|x: i32| x + 1);
    let composed = double.and_then(&add_one);

    // Both original and composed can be used
    assert_eq!(double.transform(21), 42);
    assert_eq!(composed.transform(5), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_rc_transformer_once_compose() {
    let double = RcTransformerOnce::new(|x: i32| x * 2);
    let add_one = RcTransformerOnce::new(|x: i32| x + 1);
    let composed = double.compose(&add_one);

    assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_rc_transformer_once_chain() {
    let add_one = RcTransformerOnce::new(|x: i32| x + 1);
    let double = RcTransformerOnce::new(|x: i32| x * 2);
    let subtract_three = RcTransformerOnce::new(|x: i32| x - 3);

    let composed = add_one.and_then(&double).and_then(&subtract_three);

    // Original transformers still usable
    assert_eq!(add_one.transform(5), 6);
    assert_eq!(composed.transform(5), 9); // ((5 + 1) * 2) - 3 = 9
}

#[test]
fn test_rc_transformer_once_with_string() {
    let to_upper = RcTransformerOnce::new(|s: String| s.to_uppercase());
    let input = String::from("hello");
    assert_eq!(to_upper.transform(input), "HELLO");
}

#[test]
fn test_rc_transformer_once_as_function_once() {
    let double = RcTransformerOnce::new(|x: i32| x * 2);
    assert_eq!(double.apply(21), 42);
}

#[test]
fn test_rc_transformer_once_into_fn() {
    let double = RcTransformerOnce::new(|x: i32| x * 2);
    let mut closure = double.into_fn();
    assert_eq!(closure(21), 42);
    assert_eq!(closure(10), 20);
}

// ============================================================================
// Blanket Implementation Tests
// ============================================================================

#[test]
fn test_closure_as_transformer_once() {
    let double = |x: i32| x * 2;
    assert_eq!(double.transform(21), 42);
}

#[test]
fn test_function_as_transformer_once() {
    fn double(x: i32) -> i32 {
        x * 2
    }
    assert_eq!(double.transform(21), 42);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_transformer_once_conversion() {
    let box_transformer = BoxTransformerOnce::new(|x: i32| x * 2);
    let _rc_function = box_transformer.into_rc();
    // Note: Can't test further because transformer is consumed
}

#[test]
fn test_transformer_once_with_complex_type() {
    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: i32,
    }

    let inc_age = BoxTransformerOnce::new(|mut p: Person| {
        p.age += 1;
        p
    });

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let result = inc_age.transform(person);
    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 31);
}

#[test]
fn test_transformer_once_with_captured_value() {
    let prefix = String::from("Hello, ");
    let add_prefix = BoxTransformerOnce::new(move |name: String| format!("{}{}", prefix, name));

    assert_eq!(add_prefix.transform("World".to_string()), "Hello, World");
}
