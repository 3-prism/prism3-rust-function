/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for Transformer types

use prism3_function::{ArcTransformer, BoxTransformer, Function, RcTransformer, Transformer};

// ============================================================================
// BoxTransformer Tests
// ============================================================================

#[test]
fn test_box_transformer_basic() {
    let double = BoxTransformer::new(|x: &i32| x * 2);
    assert_eq!(double.transform(&21), 42);
    assert_eq!(double.transform(&10), 20);
}

#[test]
fn test_box_transformer_identity() {
    let identity = BoxTransformer::<i32>::identity();
    assert_eq!(identity.transform(&42), 42);
    assert_eq!(identity.transform(&100), 100);
}

#[test]
fn test_box_transformer_constant() {
    let constant = BoxTransformer::constant(42);
    assert_eq!(constant.transform(&1), 42);
    assert_eq!(constant.transform(&100), 42);
}

#[test]
fn test_box_transformer_and_then() {
    let double = BoxTransformer::new(|x: &i32| x * 2);
    let add_one = BoxTransformer::new(|x: &i32| x + 1);
    let composed = double.and_then(add_one);
    assert_eq!(composed.transform(&5), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_box_transformer_compose() {
    let double = BoxTransformer::new(|x: &i32| x * 2);
    let add_one = BoxTransformer::new(|x: &i32| x + 1);
    let composed = double.compose(add_one);
    assert_eq!(composed.transform(&5), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_box_transformer_chain() {
    let add_one = BoxTransformer::new(|x: &i32| x + 1);
    let double = BoxTransformer::new(|x: &i32| x * 2);
    let subtract_three = BoxTransformer::new(|x: &i32| x - 3);

    let composed = add_one.and_then(double).and_then(subtract_three);
    assert_eq!(composed.transform(&5), 9); // ((5 + 1) * 2) - 3 = 9
}

#[test]
fn test_box_transformer_with_string() {
    let to_upper = BoxTransformer::new(|s: &String| s.to_uppercase());
    let input = String::from("hello");
    assert_eq!(to_upper.transform(&input), "HELLO");
}

#[test]
fn test_box_transformer_as_function() {
    let double = BoxTransformer::new(|x: &i32| x * 2);
    // Test using apply method from Function trait
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_box_transformer_into_fn() {
    let double = BoxTransformer::new(|x: &i32| x * 2);
    let mut closure = double.into_fn();
    assert_eq!(closure(&21), 42);
    assert_eq!(closure(&10), 20);
}

// ============================================================================
// ArcTransformer Tests
// ============================================================================

#[test]
fn test_arc_transformer_basic() {
    let double = ArcTransformer::new(|x: &i32| x * 2);
    assert_eq!(double.transform(&21), 42);
    assert_eq!(double.transform(&10), 20);
}

#[test]
fn test_arc_transformer_clone() {
    let double = ArcTransformer::new(|x: &i32| x * 2);
    let cloned = double.clone();

    assert_eq!(double.transform(&21), 42);
    assert_eq!(cloned.transform(&21), 42);
}

#[test]
fn test_arc_transformer_identity() {
    let identity = ArcTransformer::<i32>::identity();
    assert_eq!(identity.transform(&42), 42);
    assert_eq!(identity.transform(&100), 100);
}

#[test]
fn test_arc_transformer_constant() {
    let constant = ArcTransformer::constant(42);
    assert_eq!(constant.transform(&1), 42);
    assert_eq!(constant.transform(&100), 42);
}

#[test]
fn test_arc_transformer_and_then() {
    let double = ArcTransformer::new(|x: &i32| x * 2);
    let add_one = ArcTransformer::new(|x: &i32| x + 1);
    let composed = double.and_then(&add_one);

    // Original transformers still usable
    assert_eq!(double.transform(&21), 42);
    assert_eq!(add_one.transform(&41), 42);
    assert_eq!(composed.transform(&5), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_arc_transformer_compose() {
    let double = ArcTransformer::new(|x: &i32| x * 2);
    let add_one = ArcTransformer::new(|x: &i32| x + 1);
    let composed = double.compose(&add_one);

    assert_eq!(composed.transform(&5), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_arc_transformer_chain() {
    let add_one = ArcTransformer::new(|x: &i32| x + 1);
    let double = ArcTransformer::new(|x: &i32| x * 2);
    let subtract_three = ArcTransformer::new(|x: &i32| x - 3);

    let composed = add_one.and_then(&double).and_then(&subtract_three);

    // Original transformers still usable
    assert_eq!(add_one.transform(&5), 6);
    assert_eq!(composed.transform(&5), 9); // ((5 + 1) * 2) - 3 = 9
}

#[test]
fn test_arc_transformer_with_string() {
    let to_upper = ArcTransformer::new(|s: &String| s.to_uppercase());
    let input = String::from("hello");
    assert_eq!(to_upper.transform(&input), "HELLO");
}

#[test]
fn test_arc_transformer_thread_safety() {
    use std::sync::Arc as StdArc;
    use std::thread;

    let transformer = ArcTransformer::new(|x: &i32| x * 2);
    let transformer_arc = StdArc::new(transformer);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let t = StdArc::clone(&transformer_arc);
            thread::spawn(move || t.transform(&i))
        })
        .collect();

    let results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(results, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
}

#[test]
fn test_arc_transformer_as_function() {
    let double = ArcTransformer::new(|x: &i32| x * 2);
    // Test using apply method from Function trait
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_arc_transformer_into_fn() {
    let double = ArcTransformer::new(|x: &i32| x * 2);
    let mut closure = double.into_fn();
    assert_eq!(closure(&21), 42);
    assert_eq!(closure(&10), 20);
}

// ============================================================================
// RcTransformer Tests
// ============================================================================

#[test]
fn test_rc_transformer_basic() {
    let double = RcTransformer::new(|x: &i32| x * 2);
    assert_eq!(double.transform(&21), 42);
    assert_eq!(double.transform(&10), 20);
}

#[test]
fn test_rc_transformer_clone() {
    let double = RcTransformer::new(|x: &i32| x * 2);
    let cloned = double.clone();

    assert_eq!(double.transform(&21), 42);
    assert_eq!(cloned.transform(&21), 42);
}

#[test]
fn test_rc_transformer_identity() {
    let identity = RcTransformer::<i32>::identity();
    assert_eq!(identity.transform(&42), 42);
    assert_eq!(identity.transform(&100), 100);
}

#[test]
fn test_rc_transformer_constant() {
    let constant = RcTransformer::constant(42);
    assert_eq!(constant.transform(&1), 42);
    assert_eq!(constant.transform(&100), 42);
}

#[test]
fn test_rc_transformer_and_then() {
    let double = RcTransformer::new(|x: &i32| x * 2);
    let add_one = RcTransformer::new(|x: &i32| x + 1);
    let composed = double.and_then(&add_one);

    // Original transformers still usable
    assert_eq!(double.transform(&21), 42);
    assert_eq!(add_one.transform(&41), 42);
    assert_eq!(composed.transform(&5), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_rc_transformer_compose() {
    let double = RcTransformer::new(|x: &i32| x * 2);
    let add_one = RcTransformer::new(|x: &i32| x + 1);
    let composed = double.compose(&add_one);

    assert_eq!(composed.transform(&5), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_rc_transformer_chain() {
    let add_one = RcTransformer::new(|x: &i32| x + 1);
    let double = RcTransformer::new(|x: &i32| x * 2);
    let subtract_three = RcTransformer::new(|x: &i32| x - 3);

    let composed = add_one.and_then(&double).and_then(&subtract_three);

    // Original transformers still usable
    assert_eq!(add_one.transform(&5), 6);
    assert_eq!(composed.transform(&5), 9); // ((5 + 1) * 2) - 3 = 9
}

#[test]
fn test_rc_transformer_with_string() {
    let to_upper = RcTransformer::new(|s: &String| s.to_uppercase());
    let input = String::from("hello");
    assert_eq!(to_upper.transform(&input), "HELLO");
}

#[test]
fn test_rc_transformer_as_function() {
    let double = RcTransformer::new(|x: &i32| x * 2);
    // Test using apply method from Function trait
    assert_eq!(double.apply(&21), 42);
}

#[test]
fn test_rc_transformer_into_fn() {
    let double = RcTransformer::new(|x: &i32| x * 2);
    let mut closure = double.into_fn();
    assert_eq!(closure(&21), 42);
    assert_eq!(closure(&10), 20);
}

// ============================================================================
// Blanket Implementation Tests
// ============================================================================

#[test]
fn test_closure_as_transformer() {
    let double = |x: &i32| x * 2;
    assert_eq!(double.transform(&21), 42);
}

#[test]
fn test_function_as_transformer() {
    fn double(x: &i32) -> i32 {
        x * 2
    }
    assert_eq!(double.transform(&21), 42);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_transformer_conversion() {
    let box_transformer = BoxTransformer::new(|x: &i32| x * 2);
    let _rc_function = box_transformer.into_rc();
    // Note: Can't test further because transformer is consumed
}

#[test]
fn test_transformer_with_complex_type() {
    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: i32,
    }

    let inc_age = BoxTransformer::new(|p: &Person| Person {
        name: p.name.clone(),
        age: p.age + 1,
    });

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let result = inc_age.transform(&person);
    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 31);
}
