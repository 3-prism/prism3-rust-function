/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    ArcBiPredicate,
    ArcStatefulBiTransformer,
    ArcStatefulTransformer,
    BiPredicate,
    BiTransformerOnce,
    BoxBiPredicate,
    BoxStatefulBiTransformer,
    BoxStatefulTransformer,
    FnStatefulBiTransformerOps,
    RcBiPredicate,
    RcStatefulBiTransformer,
    RcStatefulTransformer,
    StatefulBiTransformer,
};

// ============================================================================
// BoxStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_box_stateful_bi_transformer_new() {
    // Test basic creation and usage with stateful transformation
    let mut counter = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31); // 10 + 20 + 1
    assert_eq!(transformer.apply(10, 20), 32); // 10 + 20 + 2
    assert_eq!(transformer.apply(10, 20), 33); // 10 + 20 + 3
}

#[test]
fn test_box_stateful_bi_transformer_constant() {
    // Test constant bi-transformer that ignores inputs
    let mut constant = BoxStatefulBiTransformer::constant("hello");
    assert_eq!(constant.apply(1, 2), "hello");
    assert_eq!(constant.apply(3, 4), "hello");
    assert_eq!(constant.apply(5, 6), "hello");
}

#[test]
fn test_box_stateful_bi_transformer_and_then() {
    // Test composition with and_then method
    let mut counter1 = 0;
    let bi_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter1 += 1;
        x + y + counter1
    });

    let mut counter2 = 0;
    let trans = BoxStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = bi_trans.and_then(trans);
    assert_eq!(composed.apply(10, 20), 31); // (10 + 20 + 1) * 1
    assert_eq!(composed.apply(10, 20), 64); // (10 + 20 + 2) * 2
    assert_eq!(composed.apply(10, 20), 99); // (10 + 20 + 3) * 3
}

#[test]
fn test_box_stateful_bi_transformer_and_then_with_closure() {
    // Test and_then with a closure
    let mut counter = 0;
    let bi_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut composed = bi_trans.and_then(|x: i32| x * 2);
    assert_eq!(composed.apply(5, 5), 22); // (5 + 5 + 1) * 2
    assert_eq!(composed.apply(5, 5), 24); // (5 + 5 + 2) * 2
}

#[test]
fn test_box_stateful_bi_transformer_when_or_else() {
    // Test conditional execution with when and or_else
    let mut then_count = 0;
    let mut else_count = 0;

    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(move |x, y| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    assert_eq!(transformer.apply(5, 3), "Then[1]: 8");
    assert_eq!(transformer.apply(-5, 3), "Else[1]: -15");
    assert_eq!(transformer.apply(10, 2), "Then[2]: 12");
    assert_eq!(transformer.apply(0, 5), "Else[2]: 0");
}

#[test]
fn test_box_stateful_bi_transformer_when_with_predicate() {
    // Test when with a predicate object
    let predicate = BoxBiPredicate::new(|x: &i32, y: &i32| *x >= 10 && *y >= 10);

    let mut transformer = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y)
        .when(predicate)
        .or_else(|x, y| x * y);

    assert_eq!(transformer.apply(15, 20), 35); // both >= 10, add
    assert_eq!(transformer.apply(5, 20), 100); // not both >= 10, multiply
}

#[test]
fn test_box_stateful_bi_transformer_into_box() {
    // Test into_box conversion (zero-cost)
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut boxed = transformer.into_box();
    assert_eq!(boxed.apply(10, 20), 31);
    assert_eq!(boxed.apply(10, 20), 32);
}

#[test]
fn test_box_stateful_bi_transformer_into_rc() {
    // Test into_rc conversion
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut rc_transformer = transformer.into_rc();
    assert_eq!(rc_transformer.apply(10, 20), 31);
    assert_eq!(rc_transformer.apply(10, 20), 32);
}

#[test]
fn test_box_stateful_bi_transformer_into_fn() {
    // Test into_fn conversion
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let mut closure = transformer.into_fn();
    assert_eq!(closure(10, 2), 20); // 10 * 2 * 1
    assert_eq!(closure(10, 2), 40); // 10 * 2 * 2
    assert_eq!(closure(10, 2), 60); // 10 * 2 * 3
}

#[test]
fn test_box_stateful_bi_transformer_with_string_types() {
    // Test with string input and output types
    let mut count = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |s1: String, s2: String| {
        count += 1;
        format!("[{}] {}{}", count, s1, s2)
    });

    assert_eq!(
        transformer.apply("hello".to_string(), "world".to_string()),
        "[1] helloworld"
    );
    assert_eq!(
        transformer.apply("foo".to_string(), "bar".to_string()),
        "[2] foobar"
    );
}

#[test]
fn test_box_stateful_bi_transformer_different_types() {
    // Test with different input and output types
    let mut counter = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |name: String, age: i32| {
        counter += 1;
        format!("#{} {} is {}", counter, name, age)
    });

    assert_eq!(transformer.apply("Alice".to_string(), 30), "#1 Alice is 30");
    assert_eq!(transformer.apply("Bob".to_string(), 25), "#2 Bob is 25");
}

#[test]
fn test_box_stateful_bi_transformer_accumulation() {
    // Test stateful accumulation
    let mut sum = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        sum += x + y;
        sum
    });

    assert_eq!(transformer.apply(10, 20), 30);
    assert_eq!(transformer.apply(5, 5), 40);
    assert_eq!(transformer.apply(3, 7), 50);
}

#[test]
fn test_box_stateful_bi_transformer_complex_state() {
    // Test with complex internal state
    let mut history = Vec::new();
    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        let sum = x + y;
        history.push(sum);
        (sum, history.len())
    });

    assert_eq!(transformer.apply(10, 20), (30, 1));
    assert_eq!(transformer.apply(5, 5), (10, 2));
    assert_eq!(transformer.apply(3, 7), (10, 3));
}

// ============================================================================
// ArcStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_arc_stateful_bi_transformer_new() {
    // Test basic creation and usage
    let mut counter = 0;
    let mut transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31);
    assert_eq!(transformer.apply(10, 20), 32);
    assert_eq!(transformer.apply(10, 20), 33);
}

#[test]
fn test_arc_stateful_bi_transformer_constant() {
    // Test constant bi-transformer
    let mut constant = ArcStatefulBiTransformer::constant("hello");
    assert_eq!(constant.apply(1, 2), "hello");
    assert_eq!(constant.apply(3, 4), "hello");
}

#[test]
fn test_arc_stateful_bi_transformer_clone() {
    // Test cloning and shared state
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut t1 = transformer.clone();
    let mut t2 = transformer.clone();

    assert_eq!(t1.apply(10, 20), 31); // counter = 1
    assert_eq!(t2.apply(10, 20), 32); // counter = 2 (shared state)
    assert_eq!(t1.apply(10, 20), 33); // counter = 3 (shared state)
}

#[test]
fn test_arc_stateful_bi_transformer_and_then() {
    // Test composition with and_then
    let mut counter1 = 0;
    let bi_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter1 += 1;
        x + y + counter1
    });

    let mut counter2 = 0;
    let trans = ArcStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = bi_trans.and_then(trans);
    assert_eq!(composed.apply(10, 20), 31); // (10 + 20 + 1) * 1
    assert_eq!(composed.apply(10, 20), 64); // (10 + 20 + 2) * 2
}

#[test]
fn test_arc_stateful_bi_transformer_and_then_preserves_original() {
    // Test that and_then uses &self and preserves original
    let mut counter = 0;
    let bi_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let trans = ArcStatefulTransformer::new(|x: i32| x * 2);
    let mut composed = bi_trans.and_then(trans);

    assert_eq!(composed.apply(5, 5), 22); // (5 + 5 + 1) * 2

    // Original bi_trans still usable
    let mut original = bi_trans.clone();
    assert_eq!(original.apply(10, 20), 32); // 10 + 20 + 2 (state continues)
}

#[test]
fn test_arc_stateful_bi_transformer_when_or_else() {
    // Test conditional execution
    let mut then_count = 0;
    let mut else_count = 0;

    let mut transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(move |x, y| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    assert_eq!(transformer.apply(5, 3), "Then[1]: 8");
    assert_eq!(transformer.apply(-5, 3), "Else[1]: -15");
}

#[test]
fn test_arc_stateful_bi_transformer_when_preserves_original() {
    // Test that when uses &self and preserves original
    let transformer = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = transformer.when(|x: &i32, _y: &i32| *x > 0);

    let mut result = conditional.or_else(|x, y| x * y);
    assert_eq!(result.apply(5, 3), 8);

    // Original transformer still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 20), 30);
}

#[test]
fn test_arc_stateful_bi_transformer_into_box() {
    // Test into_box conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut boxed = transformer.into_box();
    assert_eq!(boxed.apply(10, 20), 31);
    assert_eq!(boxed.apply(10, 20), 32);
}

#[test]
fn test_arc_stateful_bi_transformer_into_rc() {
    // Test into_rc conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut rc_transformer = transformer.into_rc();
    assert_eq!(rc_transformer.apply(10, 20), 31);
    assert_eq!(rc_transformer.apply(10, 20), 32);
}

#[test]
fn test_arc_stateful_bi_transformer_into_arc() {
    // Test into_arc conversion (zero-cost)
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut arc_transformer = transformer.into_arc();
    assert_eq!(arc_transformer.apply(10, 20), 31);
    assert_eq!(arc_transformer.apply(10, 20), 32);
}

#[test]
fn test_arc_stateful_bi_transformer_into_fn() {
    // Test into_fn conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let mut closure = transformer.into_fn();
    assert_eq!(closure(10, 2), 20); // 10 * 2 * 1
    assert_eq!(closure(10, 2), 40); // 10 * 2 * 2
}

#[test]
fn test_arc_stateful_bi_transformer_to_arc() {
    // Test non-consuming to_arc conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let arc1 = transformer.to_arc();
    let mut a1 = arc1.clone();
    assert_eq!(a1.apply(10, 20), 31);

    let arc2 = transformer.to_arc();
    let mut a2 = arc2.clone();
    assert_eq!(a2.apply(10, 20), 32); // shared state

    // Original still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 20), 33);
}

#[test]
fn test_arc_stateful_bi_transformer_thread_safe() {
    // Test thread safety
    use std::thread;

    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let t1 = transformer.clone();
    let t2 = transformer.clone();

    let handle1 = thread::spawn(move || {
        let mut t = t1.clone();
        t.apply(10, 20)
    });

    let handle2 = thread::spawn(move || {
        let mut t = t2.clone();
        t.apply(5, 5)
    });

    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();

    // Results depend on execution order, but both should be valid
    assert!((31..=32).contains(&result1));
    assert!((11..=12).contains(&result2));
}

// ============================================================================
// RcStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_rc_stateful_bi_transformer_new() {
    // Test basic creation and usage
    let mut counter = 0;
    let mut transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31);
    assert_eq!(transformer.apply(10, 20), 32);
    assert_eq!(transformer.apply(10, 20), 33);
}

#[test]
fn test_rc_stateful_bi_transformer_constant() {
    // Test constant bi-transformer
    let mut constant = RcStatefulBiTransformer::constant("hello");
    assert_eq!(constant.apply(1, 2), "hello");
    assert_eq!(constant.apply(3, 4), "hello");
}

#[test]
fn test_rc_stateful_bi_transformer_clone() {
    // Test cloning and shared state
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut t1 = transformer.clone();
    let mut t2 = transformer.clone();

    assert_eq!(t1.apply(10, 20), 31); // counter = 1
    assert_eq!(t2.apply(10, 20), 32); // counter = 2 (shared state)
    assert_eq!(t1.apply(10, 20), 33); // counter = 3 (shared state)
}

#[test]
fn test_rc_stateful_bi_transformer_and_then() {
    // Test composition with and_then
    let mut counter1 = 0;
    let bi_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter1 += 1;
        x + y + counter1
    });

    let mut counter2 = 0;
    let trans = RcStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = bi_trans.and_then(trans);
    assert_eq!(composed.apply(10, 20), 31); // (10 + 20 + 1) * 1
    assert_eq!(composed.apply(10, 20), 64); // (10 + 20 + 2) * 2
}

#[test]
fn test_rc_stateful_bi_transformer_and_then_preserves_original() {
    // Test that and_then uses &self and preserves original
    let mut counter = 0;
    let bi_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let trans = RcStatefulTransformer::new(|x: i32| x * 2);
    let mut composed = bi_trans.and_then(trans);

    assert_eq!(composed.apply(5, 5), 22); // (5 + 5 + 1) * 2

    // Original bi_trans still usable
    let mut original = bi_trans.clone();
    assert_eq!(original.apply(10, 20), 32); // 10 + 20 + 2 (state continues)
}

#[test]
fn test_rc_stateful_bi_transformer_when_or_else() {
    // Test conditional execution
    let mut then_count = 0;
    let mut else_count = 0;

    let mut transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(move |x, y| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    assert_eq!(transformer.apply(5, 3), "Then[1]: 8");
    assert_eq!(transformer.apply(-5, 3), "Else[1]: -15");
}

#[test]
fn test_rc_stateful_bi_transformer_when_preserves_original() {
    // Test that when uses &self and preserves original
    let transformer = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = transformer.when(|x: &i32, _y: &i32| *x > 0);

    let mut result = conditional.or_else(|x, y| x * y);
    assert_eq!(result.apply(5, 3), 8);

    // Original transformer still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 20), 30);
}

#[test]
fn test_rc_stateful_bi_transformer_into_box() {
    // Test into_box conversion
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut boxed = transformer.into_box();
    assert_eq!(boxed.apply(10, 20), 31);
    assert_eq!(boxed.apply(10, 20), 32);
}

#[test]
fn test_rc_stateful_bi_transformer_into_rc() {
    // Test into_rc conversion (zero-cost)
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut rc_transformer = transformer.into_rc();
    assert_eq!(rc_transformer.apply(10, 20), 31);
    assert_eq!(rc_transformer.apply(10, 20), 32);
}

#[test]
fn test_rc_stateful_bi_transformer_into_fn() {
    // Test into_fn conversion
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let mut closure = transformer.into_fn();
    assert_eq!(closure(10, 2), 20); // 10 * 2 * 1
    assert_eq!(closure(10, 2), 40); // 10 * 2 * 2
}

#[test]
fn test_rc_stateful_bi_transformer_to_rc() {
    // Test non-consuming to_rc conversion
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let rc1 = transformer.to_rc();
    let mut r1 = rc1.clone();
    assert_eq!(r1.apply(10, 20), 31);

    let rc2 = transformer.to_rc();
    let mut r2 = rc2.clone();
    assert_eq!(r2.apply(10, 20), 32); // shared state

    // Original still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 20), 33);
}

// ============================================================================
// Closure StatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_closure_as_stateful_bi_transformer() {
    // Test that closures implement StatefulBiTransformer
    let mut counter = 0;
    let mut transformer = |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    assert_eq!(transformer.apply(10, 20), 31);
    assert_eq!(transformer.apply(10, 20), 32);
    assert_eq!(transformer.apply(10, 20), 33);
}

#[test]
fn test_closure_into_box() {
    // Test closure conversion to BoxStatefulBiTransformer
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    let mut boxed = transformer.into_box();
    assert_eq!(boxed.apply(10, 20), 31);
    assert_eq!(boxed.apply(10, 20), 32);
}

#[test]
fn test_closure_into_rc() {
    // Test closure conversion to RcStatefulBiTransformer
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    let mut rc_transformer = transformer.into_rc();
    assert_eq!(rc_transformer.apply(10, 20), 31);
    assert_eq!(rc_transformer.apply(10, 20), 32);
}

#[test]
fn test_closure_into_arc() {
    // Test closure conversion to ArcStatefulBiTransformer
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    let mut arc_transformer = transformer.into_arc();
    assert_eq!(arc_transformer.apply(10, 20), 31);
    assert_eq!(arc_transformer.apply(10, 20), 32);
}

#[test]
fn test_closure_into_fn() {
    // Test closure into_fn conversion
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    let mut closure = transformer.into_fn();
    assert_eq!(closure(10, 20), 31);
    assert_eq!(closure(10, 20), 32);
}

// ============================================================================
// FnStatefulBiTransformerOps Tests
// ============================================================================

#[test]
fn test_fn_stateful_bi_transformer_ops_and_then() {
    // Test and_then extension method for closures
    let mut counter1 = 0;
    let bi_trans = move |x: i32, y: i32| {
        counter1 += 1;
        x + y + counter1
    };

    let mut counter2 = 0;
    let trans = move |x: i32| {
        counter2 += 1;
        x * counter2
    };

    let mut composed = bi_trans.and_then(trans);
    assert_eq!(composed.apply(10, 20), 31); // (10 + 20 + 1) * 1
    assert_eq!(composed.apply(10, 20), 64); // (10 + 20 + 2) * 2
}

#[test]
fn test_fn_stateful_bi_transformer_ops_and_then_with_transformer() {
    // Test and_then with a stateful transformer object
    let mut counter = 0;
    let bi_trans = move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    let trans = BoxStatefulTransformer::new(|x: i32| x * 2);
    let mut composed = bi_trans.and_then(trans);

    assert_eq!(composed.apply(5, 5), 22); // (5 + 5 + 1) * 2
    assert_eq!(composed.apply(5, 5), 24); // (5 + 5 + 2) * 2
}

#[test]
fn test_fn_stateful_bi_transformer_ops_when() {
    // Test when extension method for closures
    let mut transformer = (|x: i32, y: i32| x + y)
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(|x, y| x * y);

    assert_eq!(transformer.apply(5, 3), 8);
    assert_eq!(transformer.apply(-5, 3), -15);
    assert_eq!(transformer.apply(0, 5), 0);
}

#[test]
fn test_fn_stateful_bi_transformer_ops_when_with_predicate() {
    // Test when with a predicate object
    let predicate = BoxBiPredicate::new(|x: &i32, _y: &i32| *x >= 10);

    let mut transformer = (|x: i32, y: i32| x + y)
        .when(predicate)
        .or_else(|x, y| x * y);

    assert_eq!(transformer.apply(15, 5), 20); // x >= 10, add
    assert_eq!(transformer.apply(5, 10), 50); // x < 10, multiply
}

// ============================================================================
// BoxConditionalStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_box_conditional_or_else_basic() {
    // Test basic or_else functionality
    let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply = BoxStatefulBiTransformer::new(|x: i32, y: i32| x * y);

    let mut conditional = add
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply);

    assert_eq!(conditional.apply(5, 3), 8); // both positive, add
    assert_eq!(conditional.apply(-5, 3), -15); // not both positive, multiply
    assert_eq!(conditional.apply(0, 5), 0); // zero, multiply
}

#[test]
fn test_box_conditional_or_else_with_closure() {
    // Test or_else with closure
    let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);

    let mut conditional = add.when(|x: &i32, _y: &i32| *x > 10).or_else(|x, y| x - y);

    assert_eq!(conditional.apply(15, 5), 20); // x > 10, add
    assert_eq!(conditional.apply(5, 3), 2); // x <= 10, subtract
}

#[test]
fn test_box_conditional_stateful_transformers() {
    // Test conditional with stateful transformers
    let mut then_count = 0;
    let mut else_count = 0;

    let then_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    });

    let else_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    let mut conditional = then_trans
        .when(|x: &i32, _y: &i32| *x > 0)
        .or_else(else_trans);

    assert_eq!(conditional.apply(5, 3), "Then[1]: 8");
    assert_eq!(conditional.apply(-5, 3), "Else[1]: -15");
    assert_eq!(conditional.apply(10, 2), "Then[2]: 12");
}

// ============================================================================
// ArcConditionalStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_arc_conditional_or_else_basic() {
    // Test basic or_else functionality
    let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply = ArcStatefulBiTransformer::new(|x: i32, y: i32| x * y);

    let mut conditional = add
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply);

    assert_eq!(conditional.apply(5, 3), 8);
    assert_eq!(conditional.apply(-5, 3), -15);
}

#[test]
fn test_arc_conditional_clone() {
    // Test cloning of conditional transformer
    let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = add.when(|x: &i32, _y: &i32| *x > 0);

    let conditional_clone = conditional.clone();

    let mut result1 = conditional.or_else(|x, y| x * y);
    let mut result2 = conditional_clone.or_else(|x, y| x * y);

    assert_eq!(result1.apply(5, 3), 8);
    assert_eq!(result2.apply(5, 3), 8);
    assert_eq!(result1.apply(-5, 3), -15);
    assert_eq!(result2.apply(-5, 3), -15);
}

#[test]
fn test_arc_conditional_stateful_transformers() {
    // Test conditional with stateful transformers
    let mut then_count = 0;
    let mut else_count = 0;

    let then_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    });

    let else_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    let mut conditional = then_trans
        .when(|x: &i32, _y: &i32| *x > 0)
        .or_else(else_trans);

    assert_eq!(conditional.apply(5, 3), "Then[1]: 8");
    assert_eq!(conditional.apply(-5, 3), "Else[1]: -15");
}

// ============================================================================
// RcConditionalStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_rc_conditional_or_else_basic() {
    // Test basic or_else functionality
    let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply = RcStatefulBiTransformer::new(|x: i32, y: i32| x * y);

    let mut conditional = add
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply);

    assert_eq!(conditional.apply(5, 3), 8);
    assert_eq!(conditional.apply(-5, 3), -15);
}

#[test]
fn test_rc_conditional_clone() {
    // Test cloning of conditional transformer
    let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = add.when(|x: &i32, _y: &i32| *x > 0);

    let conditional_clone = conditional.clone();

    let mut result1 = conditional.or_else(|x, y| x * y);
    let mut result2 = conditional_clone.or_else(|x, y| x * y);

    assert_eq!(result1.apply(5, 3), 8);
    assert_eq!(result2.apply(5, 3), 8);
    assert_eq!(result1.apply(-5, 3), -15);
    assert_eq!(result2.apply(-5, 3), -15);
}

#[test]
fn test_rc_conditional_stateful_transformers() {
    // Test conditional with stateful transformers
    let mut then_count = 0;
    let mut else_count = 0;

    let then_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    });

    let else_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    let mut conditional = then_trans
        .when(|x: &i32, _y: &i32| *x > 0)
        .or_else(else_trans);

    assert_eq!(conditional.apply(5, 3), "Then[1]: 8");
    assert_eq!(conditional.apply(-5, 3), "Else[1]: -15");
}

// ============================================================================
// BiTransformerOnce Implementation Tests
// ============================================================================

#[test]
fn test_box_stateful_bi_transformer_apply_once() {
    // Test apply_once consuming the transformer
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply_once(10, 20), 31);
    // transformer is now consumed
}

#[test]
fn test_box_stateful_bi_transformer_into_box_once() {
    // Test into_box_once conversion
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let once_transformer = transformer.into_box_once();
    assert_eq!(once_transformer.apply_once(10, 2), 20); // 10 * 2 * 1
}

#[test]
fn test_box_stateful_bi_transformer_into_fn_once() {
    // Test into_fn_once conversion
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let fn_once = transformer.into_fn_once();
    assert_eq!(fn_once(10, 2), 20); // 10 * 2 * 1
}

#[test]
fn test_arc_stateful_bi_transformer_apply_once() {
    // Test apply_once for ArcStatefulBiTransformer
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply_once(10, 20), 31);
}

#[test]
fn test_arc_stateful_bi_transformer_into_box_once() {
    // Test into_box_once conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let once_transformer = transformer.into_box_once();
    assert_eq!(once_transformer.apply_once(10, 2), 20);
}

#[test]
fn test_arc_stateful_bi_transformer_into_fn_once() {
    // Test into_fn_once conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let fn_once = transformer.into_fn_once();
    assert_eq!(fn_once(10, 2), 20);
}

#[test]
fn test_arc_stateful_bi_transformer_to_box_once() {
    // Test non-consuming to_box_once conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let once_transformer = transformer.to_box_once();
    assert_eq!(once_transformer.apply_once(10, 2), 20); // 10 * 2 * 1

    // Original still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 2), 40); // 10 * 2 * 2 (state continues)
}

#[test]
fn test_arc_stateful_bi_transformer_to_fn_once() {
    // Test non-consuming to_fn_once conversion
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let fn_once = transformer.to_fn_once();
    assert_eq!(fn_once(10, 2), 20); // 10 * 2 * 1

    // Original still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 2), 40); // 10 * 2 * 2 (state continues)
}

#[test]
fn test_rc_stateful_bi_transformer_apply_once() {
    // Test apply_once for RcStatefulBiTransformer
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply_once(10, 20), 31);
}

#[test]
fn test_rc_stateful_bi_transformer_into_box_once() {
    // Test into_box_once conversion
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let once_transformer = transformer.into_box_once();
    assert_eq!(once_transformer.apply_once(10, 2), 20);
}

#[test]
fn test_rc_stateful_bi_transformer_into_fn_once() {
    // Test into_fn_once conversion
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let fn_once = transformer.into_fn_once();
    assert_eq!(fn_once(10, 2), 20);
}

#[test]
fn test_rc_stateful_bi_transformer_to_box_once() {
    // Test non-consuming to_box_once conversion
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let once_transformer = transformer.to_box_once();
    assert_eq!(once_transformer.apply_once(10, 2), 20); // 10 * 2 * 1

    // Original still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 2), 40); // 10 * 2 * 2 (state continues)
}

#[test]
fn test_rc_stateful_bi_transformer_to_fn_once() {
    // Test non-consuming to_fn_once conversion
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    });

    let fn_once = transformer.to_fn_once();
    assert_eq!(fn_once(10, 2), 20); // 10 * 2 * 1

    // Original still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 2), 40); // 10 * 2 * 2 (state continues)
}

// ============================================================================
// Custom StatefulBiTransformer Implementation Tests
// ============================================================================

/// Custom stateful bi-transformer for testing default trait methods
#[derive(Clone)]
struct CustomStatefulBiTransformer {
    multiplier: i32,
}

impl StatefulBiTransformer<i32, i32, i32> for CustomStatefulBiTransformer {
    fn apply(&mut self, first: i32, second: i32) -> i32 {
        self.multiplier += 1;
        (first + second) * self.multiplier
    }
}

#[test]
fn test_custom_stateful_bi_transformer_into_box() {
    // Test custom transformer into_box conversion
    let transformer = CustomStatefulBiTransformer { multiplier: 0 };
    let mut boxed = transformer.into_box();

    assert_eq!(boxed.apply(10, 20), 30); // (10 + 20) * 1
    assert_eq!(boxed.apply(10, 20), 60); // (10 + 20) * 2
    assert_eq!(boxed.apply(10, 20), 90); // (10 + 20) * 3
}

#[test]
fn test_custom_stateful_bi_transformer_into_rc() {
    // Test custom transformer into_rc conversion
    let transformer = CustomStatefulBiTransformer { multiplier: 0 };
    let mut rc_transformer = transformer.into_rc();

    assert_eq!(rc_transformer.apply(10, 20), 30);
    assert_eq!(rc_transformer.apply(10, 20), 60);
}

#[test]
fn test_custom_stateful_bi_transformer_into_rc_clone() {
    // Test custom transformer into_rc with cloning
    let transformer = CustomStatefulBiTransformer { multiplier: 0 };
    let rc_transformer = transformer.into_rc();

    let mut t1 = rc_transformer.clone();
    let mut t2 = rc_transformer.clone();

    // Shared state
    assert_eq!(t1.apply(10, 20), 30); // (10 + 20) * 1
    assert_eq!(t2.apply(10, 20), 60); // (10 + 20) * 2
    assert_eq!(t1.apply(10, 20), 90); // (10 + 20) * 3
}

/// Custom Send stateful bi-transformer for testing Arc conversion
#[derive(Clone)]
struct CustomSendStatefulBiTransformer {
    multiplier: i32,
}

impl StatefulBiTransformer<i32, i32, i32> for CustomSendStatefulBiTransformer {
    fn apply(&mut self, first: i32, second: i32) -> i32 {
        self.multiplier += 1;
        (first + second) * self.multiplier
    }
}

unsafe impl Send for CustomSendStatefulBiTransformer {}
unsafe impl Sync for CustomSendStatefulBiTransformer {}

#[test]
fn test_custom_send_stateful_bi_transformer_into_arc() {
    // Test custom Send transformer into_arc conversion
    let transformer = CustomSendStatefulBiTransformer { multiplier: 0 };
    let mut arc_transformer = transformer.into_arc();

    assert_eq!(arc_transformer.apply(10, 20), 30);
    assert_eq!(arc_transformer.apply(10, 20), 60);
}

#[test]
fn test_custom_send_stateful_bi_transformer_into_arc_clone() {
    // Test custom Send transformer into_arc with cloning
    let transformer = CustomSendStatefulBiTransformer { multiplier: 0 };
    let arc_transformer = transformer.into_arc();

    let mut t1 = arc_transformer.clone();
    let mut t2 = arc_transformer.clone();

    // Shared state
    assert_eq!(t1.apply(10, 20), 30); // (10 + 20) * 1
    assert_eq!(t2.apply(10, 20), 60); // (10 + 20) * 2
    assert_eq!(t1.apply(10, 20), 90); // (10 + 20) * 3
}

// ============================================================================
// Complex Composition Tests
// ============================================================================

#[test]
fn test_complex_pipeline() {
    // Test complex pipeline with multiple transformations
    let mut counter1 = 0;
    let step1 = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter1 += 1;
        format!("Step1[{}]: {}", counter1, x + y)
    });

    let mut counter2 = 0;
    let step2 = BoxStatefulTransformer::new(move |s: String| {
        counter2 += 1;
        format!("{} -> Step2[{}]", s, counter2)
    });

    let mut counter3 = 0;
    let step3 = BoxStatefulTransformer::new(move |s: String| {
        counter3 += 1;
        format!("{} -> Step3[{}]", s, counter3)
    });

    let mut pipeline = step1.and_then(step2).and_then(step3);

    assert_eq!(
        pipeline.apply(10, 20),
        "Step1[1]: 30 -> Step2[1] -> Step3[1]"
    );
    assert_eq!(pipeline.apply(5, 5), "Step1[2]: 10 -> Step2[2] -> Step3[2]");
}

#[test]
fn test_nested_conditional() {
    // Test nested conditional transformations
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut error_count = 0;

    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        valid_count += 1;
        format!("Valid[{}]: {}", valid_count, x + y)
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(move |x: i32, y: i32| {
        let mut sub_transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
            invalid_count += 1;
            format!("Invalid[{}]: {}", invalid_count, x * y)
        })
        .when(move |x: &i32, y: &i32| *x < 0 || *y < 0)
        .or_else(move |x: i32, y: i32| {
            error_count += 1;
            format!("Error[{}]: {} {}", error_count, x, y)
        });
        sub_transformer.apply(x, y)
    });

    assert_eq!(transformer.apply(5, 3), "Valid[1]: 8");
    assert_eq!(transformer.apply(-5, 3), "Invalid[1]: -15");
    assert_eq!(transformer.apply(0, 0), "Error[1]: 0 0");
    assert_eq!(transformer.apply(10, 20), "Valid[2]: 30");
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_zero_values() {
    // Test with zero values
    let mut transformer = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    assert_eq!(transformer.apply(0, 0), 0);
    assert_eq!(transformer.apply(0, 5), 5);
    assert_eq!(transformer.apply(5, 0), 5);
}

#[test]
fn test_negative_values() {
    // Test with negative values
    let mut transformer = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    assert_eq!(transformer.apply(-5, -3), -8);
    assert_eq!(transformer.apply(-5, 3), -2);
    assert_eq!(transformer.apply(5, -3), 2);
}

#[test]
fn test_max_min_values() {
    // Test with max and min values
    let mut transformer = BoxStatefulBiTransformer::new(|x: i32, y: i32| if x > y { x } else { y });

    assert_eq!(transformer.apply(i32::MAX, 0), i32::MAX);
    assert_eq!(transformer.apply(i32::MIN, 0), 0);
    assert_eq!(transformer.apply(i32::MAX, i32::MIN), i32::MAX);
}

#[test]
fn test_empty_string() {
    // Test with empty strings
    let mut transformer =
        BoxStatefulBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));

    assert_eq!(transformer.apply("".to_string(), "".to_string()), "");
    assert_eq!(
        transformer.apply("hello".to_string(), "".to_string()),
        "hello"
    );
    assert_eq!(
        transformer.apply("".to_string(), "world".to_string()),
        "world"
    );
}

#[test]
fn test_unicode_strings() {
    // Test with Unicode strings
    let mut transformer =
        BoxStatefulBiTransformer::new(|s1: String, s2: String| format!("{} {}", s1, s2));

    assert_eq!(
        transformer.apply("Hello".to_string(), "World".to_string()),
        "Hello World"
    );
    assert_eq!(
        transformer.apply("🦀".to_string(), "Rust".to_string()),
        "🦀 Rust"
    );
}

#[test]
fn test_option_types() {
    // Test with Option types
    let mut transformer =
        BoxStatefulBiTransformer::new(|x: Option<i32>, y: Option<i32>| match (x, y) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        });

    assert_eq!(transformer.apply(Some(5), Some(3)), Some(8));
    assert_eq!(transformer.apply(Some(5), None), None);
    assert_eq!(transformer.apply(None, Some(3)), None);
    assert_eq!(transformer.apply(None, None), None);
}

#[test]
fn test_result_types() {
    // Test with Result types
    let mut transformer =
        BoxStatefulBiTransformer::new(|x: Result<i32, String>, y: Result<i32, String>| {
            match (x, y) {
                (Ok(a), Ok(b)) => Ok(a + b),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        });

    assert_eq!(transformer.apply(Ok(5), Ok(3)), Ok(8));
    assert_eq!(
        transformer.apply(Err("error1".to_string()), Ok(3)),
        Err("error1".to_string())
    );
    assert_eq!(
        transformer.apply(Ok(5), Err("error2".to_string())),
        Err("error2".to_string())
    );
}

// ============================================================================
// Predicate Integration Tests
// ============================================================================

#[test]
fn test_with_arc_bi_predicate() {
    // Test integration with ArcBiPredicate
    let predicate = ArcBiPredicate::new(|x: &i32, y: &i32| *x > 0 && *y > 0);

    let mut transformer = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y)
        .when(predicate.clone())
        .or_else(|x, y| x * y);

    assert_eq!(transformer.apply(5, 3), 8);
    assert_eq!(transformer.apply(-5, 3), -15);

    // Predicate still usable
    assert!(predicate.test(&10, &20));
    assert!(!predicate.test(&-10, &20));
}

#[test]
fn test_with_rc_bi_predicate() {
    // Test integration with RcBiPredicate
    let predicate = RcBiPredicate::new(|x: &i32, y: &i32| *x > 0 && *y > 0);

    let mut transformer = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y)
        .when(predicate.clone())
        .or_else(|x, y| x * y);

    assert_eq!(transformer.apply(5, 3), 8);
    assert_eq!(transformer.apply(-5, 3), -15);

    // Predicate still usable
    assert!(predicate.test(&10, &20));
    assert!(!predicate.test(&-10, &20));
}

// ============================================================================
// State Sharing Tests
// ============================================================================

#[test]
fn test_arc_shared_state_multiple_clones() {
    // Test shared state across multiple clones
    use std::sync::{
        Arc,
        Mutex,
    };

    let shared_counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&shared_counter);

    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
        (x + y) * *count
    });

    let mut t1 = transformer.clone();
    let mut t2 = transformer.clone();

    assert_eq!(t1.apply(10, 20), 30); // (10 + 20) * 1
    assert_eq!(t2.apply(10, 20), 60); // (10 + 20) * 2
    assert_eq!(t1.apply(10, 20), 90); // (10 + 20) * 3

    // Verify shared counter
    assert_eq!(*shared_counter.lock().unwrap(), 3);
}

#[test]
fn test_rc_shared_state_multiple_clones() {
    // Test shared state across multiple clones for Rc
    use std::cell::RefCell;
    use std::rc::Rc;

    let shared_counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&shared_counter);

    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        let mut count = counter_clone.borrow_mut();
        *count += 1;
        (x + y) * *count
    });

    let mut t1 = transformer.clone();
    let mut t2 = transformer.clone();

    assert_eq!(t1.apply(10, 20), 30); // (10 + 20) * 1
    assert_eq!(t2.apply(10, 20), 60); // (10 + 20) * 2
    assert_eq!(t1.apply(10, 20), 90); // (10 + 20) * 3

    // Verify shared counter
    assert_eq!(*shared_counter.borrow(), 3);
}

// ============================================================================
// Cloneable Custom Transformer Tests
// ============================================================================

#[derive(Clone)]
struct CloneableStatefulBiTransformer {
    counter: i32,
}

impl StatefulBiTransformer<i32, i32, i32> for CloneableStatefulBiTransformer {
    fn apply(&mut self, first: i32, second: i32) -> i32 {
        self.counter += 1;
        (first + second) + self.counter
    }
}

#[test]
fn test_cloneable_transformer_to_box() {
    // Test to_box with cloneable transformer
    let transformer = CloneableStatefulBiTransformer { counter: 0 };

    let mut b = transformer.to_box();
    assert_eq!(b.apply(10, 20), 31); // (10 + 20) + 1
    assert_eq!(b.apply(10, 20), 32); // (10 + 20) + 2
}

#[test]
fn test_cloneable_transformer_to_rc() {
    // Test to_rc with cloneable transformer
    let transformer = CloneableStatefulBiTransformer { counter: 0 };

    let rc = transformer.to_rc();
    let mut r1 = rc.clone();
    let mut r2 = rc.clone();

    assert_eq!(r1.apply(10, 20), 31); // (10 + 20) + 1
    assert_eq!(r2.apply(10, 20), 32); // (10 + 20) + 2 (shared state)
}

#[derive(Clone)]
struct CloneableSendStatefulBiTransformer {
    counter: i32,
}

impl StatefulBiTransformer<i32, i32, i32> for CloneableSendStatefulBiTransformer {
    fn apply(&mut self, first: i32, second: i32) -> i32 {
        self.counter += 1;
        (first * second) * self.counter
    }
}

unsafe impl Send for CloneableSendStatefulBiTransformer {}
unsafe impl Sync for CloneableSendStatefulBiTransformer {}

#[test]
fn test_cloneable_send_transformer_to_arc() {
    // Test to_arc with cloneable Send transformer
    let transformer = CloneableSendStatefulBiTransformer { counter: 0 };

    let mut a = transformer.to_arc();
    assert_eq!(a.apply(3, 5), 15); // (3 * 5) * 1
    assert_eq!(a.apply(3, 5), 30); // (3 * 5) * 2
}

// ============================================================================
// Closure Non-Consuming Conversion Tests
// ============================================================================

#[test]
fn test_closure_to_box() {
    // Test non-consuming to_box conversion for closures
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    let mut boxed = transformer.to_box();
    assert_eq!(boxed.apply(10, 20), 31);
    assert_eq!(boxed.apply(10, 20), 32);

    // Can call to_box() multiple times
    let mut boxed2 = transformer.to_box();
    assert_eq!(boxed2.apply(20, 30), 51); // new state

    // Original boxed still maintains its state
    assert_eq!(boxed.apply(10, 20), 33);
}

#[test]
fn test_closure_to_rc() {
    // Test non-consuming to_rc conversion for closures
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x * y * counter
    };

    let rc_transformer = transformer.to_rc();
    let mut r1 = rc_transformer.clone();
    let mut r2 = rc_transformer.clone();

    // Shared state
    assert_eq!(r1.apply(10, 2), 20); // 10 * 2 * 1
    assert_eq!(r2.apply(10, 2), 40); // 10 * 2 * 2
    assert_eq!(r1.apply(10, 2), 60); // 10 * 2 * 3

    // Can call to_rc() again, creating new independent state
    let rc_transformer2 = transformer.to_rc();
    let mut r3 = rc_transformer2.clone();
    assert_eq!(r3.apply(10, 2), 20); // 10 * 2 * 1 (new state)
}

#[test]
fn test_closure_to_arc() {
    // Test non-consuming to_arc conversion for closures
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x - y - counter
    };

    let arc_transformer = transformer.to_arc();
    let mut a1 = arc_transformer.clone();
    let mut a2 = arc_transformer.clone();

    // Shared state
    assert_eq!(a1.apply(10, 2), 7); // 10 - 2 - 1
    assert_eq!(a2.apply(10, 2), 6); // 10 - 2 - 2
    assert_eq!(a1.apply(10, 2), 5); // 10 - 2 - 3

    // Can call to_arc() again, creating new independent state
    let arc_transformer2 = transformer.to_arc();
    let mut a3 = arc_transformer2.clone();
    assert_eq!(a3.apply(10, 2), 7); // 10 - 2 - 1 (new state)
}

#[test]
fn test_closure_to_fn() {
    // Test non-consuming to_fn conversion for closures
    let mut counter = 0;
    let transformer = move |x: i32, y: i32| {
        counter += 1;
        x + y + counter * 10
    };

    let mut closure = transformer.to_fn();
    assert_eq!(closure(5, 5), 20); // 5 + 5 + 1 * 10
    assert_eq!(closure(5, 5), 30); // 5 + 5 + 2 * 10

    // Can call to_fn() again, creating new independent state
    let mut closure2 = transformer.to_fn();
    assert_eq!(closure2(5, 5), 20); // 5 + 5 + 1 * 10 (new state)

    // Original closure still maintains its state
    assert_eq!(closure(5, 5), 40); // 5 + 5 + 3 * 10
}

// ============================================================================
// StatefulBiTransformer Trait Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod stateful_bi_transformer_default_impl_tests {
    use prism3_function::{
        ArcStatefulBiTransformer,
        BoxStatefulBiTransformer,
        StatefulBiTransformer,
    };

    /// 自定义结构体，只实现 StatefulBiTransformer trait 的核心 apply 方法
    /// 所有 into_xxx() 和 to_xxx() 方法都使用默认实现
    struct CustomStatefulBiTransformer {
        multiplier: i32,
        counter: i32,
    }

    impl StatefulBiTransformer<i32, i32, i32> for CustomStatefulBiTransformer {
        fn apply(&mut self, first: i32, second: i32) -> i32 {
            self.counter += 1;
            (first + second) * self.multiplier + self.counter
        }
        // 不覆盖任何 into_xxx() 或 to_xxx() 方法，测试默认实现
    }

    /// 可克隆的自定义有状态双参数转换器，用于测试 to_xxx() 方法
    #[derive(Clone)]
    struct CloneableCustomStatefulBiTransformer {
        multiplier: i32,
        counter: i32,
    }

    impl StatefulBiTransformer<i32, i32, i32> for CloneableCustomStatefulBiTransformer {
        fn apply(&mut self, first: i32, second: i32) -> i32 {
            self.counter += 1;
            (first + second) * self.multiplier + self.counter
        }
        // 不覆盖任何 into_xxx() 或 to_xxx() 方法，测试默认实现
    }

    #[test]
    fn test_custom_into_box() {
        let custom = CustomStatefulBiTransformer {
            multiplier: 2,
            counter: 0,
        };
        let mut boxed = custom.into_box();

        assert_eq!(boxed.apply(5, 10), 31); // (5 + 10) * 2 + 1
        assert_eq!(boxed.apply(5, 10), 32); // (5 + 10) * 2 + 2
    }

    #[test]
    fn test_custom_into_rc() {
        let custom = CustomStatefulBiTransformer {
            multiplier: 3,
            counter: 0,
        };
        let mut rc = custom.into_rc();

        assert_eq!(rc.apply(2, 3), 16); // (2 + 3) * 3 + 1
        assert_eq!(rc.apply(2, 3), 17); // (2 + 3) * 3 + 2

        // 测试克隆（共享状态）
        let mut rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(2, 3), 18); // (2 + 3) * 3 + 3
        assert_eq!(rc.apply(2, 3), 19); // (2 + 3) * 3 + 4
    }

    #[test]
    fn test_custom_into_arc() {
        let custom = CustomStatefulBiTransformer {
            multiplier: 4,
            counter: 0,
        };
        let mut arc = custom.into_arc();

        assert_eq!(arc.apply(1, 2), 13); // (1 + 2) * 4 + 1
        assert_eq!(arc.apply(1, 2), 14); // (1 + 2) * 4 + 2

        // 测试克隆（共享状态）
        let mut arc_clone = arc.clone();
        assert_eq!(arc_clone.apply(1, 2), 15); // (1 + 2) * 4 + 3
        assert_eq!(arc.apply(1, 2), 16); // (1 + 2) * 4 + 4
    }

    #[test]
    fn test_custom_into_fn() {
        let custom = CustomStatefulBiTransformer {
            multiplier: 5,
            counter: 0,
        };
        let mut func = custom.into_fn();

        assert_eq!(func(2, 2), 21); // (2 + 2) * 5 + 1
        assert_eq!(func(2, 2), 22); // (2 + 2) * 5 + 2
    }

    #[test]
    fn test_cloneable_to_box() {
        let custom = CloneableCustomStatefulBiTransformer {
            multiplier: 2,
            counter: 0,
        };
        let mut boxed = custom.to_box();

        assert_eq!(boxed.apply(5, 10), 31); // (5 + 10) * 2 + 1

        // 原始转换器仍然可用
        let mut custom_copy = custom.clone();
        assert_eq!(custom_copy.apply(3, 7), 21); // (3 + 7) * 2 + 1
    }

    #[test]
    fn test_cloneable_to_rc() {
        let custom = CloneableCustomStatefulBiTransformer {
            multiplier: 3,
            counter: 0,
        };
        let mut rc = custom.to_rc();

        assert_eq!(rc.apply(2, 3), 16); // (2 + 3) * 3 + 1

        // 原始转换器仍然可用
        let mut custom_copy = custom.clone();
        assert_eq!(custom_copy.apply(1, 1), 7); // (1 + 1) * 3 + 1
    }

    #[test]
    fn test_cloneable_to_arc() {
        let custom = CloneableCustomStatefulBiTransformer {
            multiplier: 4,
            counter: 0,
        };
        let mut arc = custom.to_arc();

        assert_eq!(arc.apply(1, 2), 13); // (1 + 2) * 4 + 1

        // 原始转换器仍然可用
        let mut custom_copy = custom.clone();
        assert_eq!(custom_copy.apply(2, 2), 17); // (2 + 2) * 4 + 1
    }

    #[test]
    fn test_cloneable_to_fn() {
        let custom = CloneableCustomStatefulBiTransformer {
            multiplier: 5,
            counter: 0,
        };
        let mut func = custom.to_fn();

        assert_eq!(func(2, 2), 21); // (2 + 2) * 5 + 1

        // 原始转换器仍然可用
        let mut custom_copy = custom.clone();
        assert_eq!(custom_copy.apply(1, 1), 11); // (1 + 1) * 5 + 1
    }

    #[test]
    fn test_custom_chained_conversions() {
        let custom1 = CustomStatefulBiTransformer {
            multiplier: 2,
            counter: 0,
        };
        let custom2 = CustomStatefulBiTransformer {
            multiplier: 3,
            counter: 0,
        };

        // 测试 into_box -> into_rc 链式转换
        let boxed: BoxStatefulBiTransformer<i32, i32, i32> = custom1.into_box();
        let mut rc = boxed.into_rc();
        assert_eq!(rc.apply(10, 11), 43); // (10 + 11) * 2 + 1

        // 测试 into_arc 直接转换
        let mut arc: ArcStatefulBiTransformer<i32, i32, i32> = custom2.into_arc();
        assert_eq!(arc.apply(5, 9), 43); // (5 + 9) * 3 + 1
    }

    #[test]
    fn test_custom_stateful_behavior() {
        let custom = CloneableCustomStatefulBiTransformer {
            multiplier: 3,
            counter: 0,
        };
        let mut boxed = custom.to_box();

        // 验证状态在多次调用中保持
        assert_eq!(boxed.apply(1, 1), 7); // (1 + 1) * 3 + 1
        assert_eq!(boxed.apply(1, 1), 8); // (1 + 1) * 3 + 2
        assert_eq!(boxed.apply(1, 1), 9); // (1 + 1) * 3 + 3
        assert_eq!(boxed.apply(2, 2), 16); // (2 + 2) * 3 + 4 (counter 已经是 4)
    }

    #[test]
    fn test_custom_composition() {
        use prism3_function::BoxStatefulTransformer;

        let custom1 = CloneableCustomStatefulBiTransformer {
            multiplier: 2,
            counter: 0,
        };

        // 创建一个单参数转换器用于组合
        let mut counter = 0;
        let single_transformer = BoxStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });

        let mut composed = custom1.to_box().and_then(single_transformer);
        // 第一次: (3 + 4) * 2 + 1 = 15, 然后 15 + 1 = 16
        assert_eq!(composed.apply(3, 4), 16);
        // 第二次: (3 + 4) * 2 + 2 = 16, 然后 16 + 2 = 18
        assert_eq!(composed.apply(3, 4), 18);
    }
}

// ============================================================================
// Custom Struct Tests - StatefulBiTransformer Default Implementation to_xxx()
// ============================================================================

#[test]
fn test_custom_stateful_bi_transformer_to_box() {
    let transformer = CustomStatefulBiTransformer { multiplier: 1 };
    let mut boxed = transformer.to_box();
    assert_eq!(boxed.apply(3, 4), 14); // (3 + 4) * 2
    assert_eq!(boxed.apply(5, 2), 21); // (5 + 2) * 3
                                       // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 10); // (2 + 3) * 2 (independent state)
}

#[test]
fn test_custom_stateful_bi_transformer_to_rc() {
    let transformer = CustomStatefulBiTransformer { multiplier: 1 };
    let mut rc = transformer.to_rc();
    assert_eq!(rc.apply(3, 4), 14); // (3 + 4) * 2
    assert_eq!(rc.apply(5, 2), 21); // (5 + 2) * 3
                                    // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 10); // (2 + 3) * 2 (independent state)
}

#[test]
fn test_custom_send_stateful_bi_transformer_to_arc() {
    let transformer = CustomSendStatefulBiTransformer { multiplier: 1 };
    let mut arc = transformer.to_arc();
    assert_eq!(arc.apply(3, 4), 14); // (3 + 4) * 2
    assert_eq!(arc.apply(5, 2), 21); // (5 + 2) * 3
                                     // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 10); // (2 + 3) * 2 (independent state)
}

#[test]
fn test_custom_stateful_bi_transformer_to_fn() {
    let transformer = CustomStatefulBiTransformer { multiplier: 1 };
    let mut closure = transformer.to_fn();
    assert_eq!(closure(3, 4), 14); // (3 + 4) * 2
    assert_eq!(closure(5, 2), 21); // (5 + 2) * 3
                                   // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 10); // (2 + 3) * 2 (independent state)
}

#[test]
fn test_cloneable_stateful_bi_transformer_to_box() {
    let transformer = CloneableStatefulBiTransformer { counter: 0 };
    let mut boxed = transformer.to_box();
    assert_eq!(boxed.apply(3, 4), 8); // (3 + 4) + 1
    assert_eq!(boxed.apply(5, 2), 9); // (5 + 2) + 2
                                      // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 6); // (2 + 3) + 1 (independent state)
}

#[test]
fn test_cloneable_stateful_bi_transformer_to_rc() {
    let transformer = CloneableStatefulBiTransformer { counter: 0 };
    let mut rc = transformer.to_rc();
    assert_eq!(rc.apply(3, 4), 8); // (3 + 4) + 1
    assert_eq!(rc.apply(5, 2), 9); // (5 + 2) + 2
                                   // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 6); // (2 + 3) + 1 (independent state)
}

#[test]
fn test_cloneable_send_stateful_bi_transformer_to_arc() {
    let transformer = CloneableSendStatefulBiTransformer { counter: 0 };
    let mut arc = transformer.to_arc();
    assert_eq!(arc.apply(3, 4), 12); // (3 * 4) * 1
    assert_eq!(arc.apply(5, 2), 20); // (5 * 2) * 2
                                     // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 6); // (2 * 3) * 1 (independent state)
}

#[test]
fn test_cloneable_stateful_bi_transformer_to_fn() {
    let transformer = CloneableStatefulBiTransformer { counter: 0 };
    let mut closure = transformer.to_fn();
    assert_eq!(closure(3, 4), 8); // (3 + 4) + 1
    assert_eq!(closure(5, 2), 9); // (5 + 2) + 2
                                  // Original transformer is still usable (was cloned)
    let mut transformer_clone = transformer.clone();
    assert_eq!(transformer_clone.apply(2, 3), 6); // (2 + 3) + 1 (independent state)
}
