/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for TransformerMut types

use prism3_function::{
    ArcTransformerMut, BoxTransformerMut, FunctionMut, RcTransformerMut, TransformerMut,
};

// ============================================================================
// BoxTransformerMut Tests
// ============================================================================

#[test]
fn test_box_transformer_mut_basic() {
    let mut double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    assert_eq!(double.transform(&mut value), 42);
    assert_eq!(value, 42);
}

#[test]
fn test_box_transformer_mut_identity() {
    let mut identity = BoxTransformerMut::<i32>::identity();
    let mut value = 42;
    assert_eq!(identity.transform(&mut value), 42);
}

#[test]
fn test_box_transformer_mut_multiple_calls() {
    let mut double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value1 = 10;
    assert_eq!(double.transform(&mut value1), 20);

    let mut value2 = 5;
    assert_eq!(double.transform(&mut value2), 10);
}

#[test]
fn test_box_transformer_mut_and_then() {
    let double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = BoxTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed = double.and_then(add_one);

    let mut value = 5;
    assert_eq!(composed.transform(&mut value), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_box_transformer_mut_compose() {
    let double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = BoxTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed = double.compose(add_one);

    let mut value = 5;
    assert_eq!(composed.transform(&mut value), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_box_transformer_mut_with_state() {
    let mut counter = 0;
    let mut counting_double = BoxTransformerMut::new(move |x: &mut i32| {
        counter += 1;
        *x *= 2;
        *x + counter
    });

    let mut value1 = 10;
    assert_eq!(counting_double.transform(&mut value1), 21); // 10 * 2 + 1

    let mut value2 = 5;
    assert_eq!(counting_double.transform(&mut value2), 12); // 5 * 2 + 2
}

#[test]
fn test_box_transformer_mut_with_string() {
    let mut to_upper = BoxTransformerMut::new(|s: &mut String| {
        *s = s.to_uppercase();
        s.clone()
    });

    let mut input = String::from("hello");
    assert_eq!(to_upper.transform(&mut input), "HELLO");
    assert_eq!(input, "HELLO");
}

#[test]
fn test_box_transformer_mut_as_function_mut() {
    let mut double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    assert_eq!(double.apply(&mut value), 42);
}

#[test]
fn test_box_transformer_mut_into_fn() {
    let double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut closure = double.into_fn();
    let mut value1 = 21;
    assert_eq!(closure(&mut value1), 42);

    let mut value2 = 10;
    assert_eq!(closure(&mut value2), 20);
}

// ============================================================================
// ArcTransformerMut Tests
// ============================================================================

#[test]
fn test_arc_transformer_mut_basic() {
    let mut double = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    assert_eq!(double.transform(&mut value), 42);
    assert_eq!(value, 42);
}

#[test]
fn test_arc_transformer_mut_clone() {
    let double = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut cloned = double.clone();
    let mut value = 21;
    assert_eq!(cloned.transform(&mut value), 42);
}

#[test]
fn test_arc_transformer_mut_identity() {
    let mut identity = ArcTransformerMut::<i32>::identity();
    let mut value = 42;
    assert_eq!(identity.transform(&mut value), 42);
}

#[test]
fn test_arc_transformer_mut_and_then() {
    let double = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = ArcTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed = double.and_then(add_one);

    let mut value = 5;
    assert_eq!(composed.transform(&mut value), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_arc_transformer_mut_compose() {
    let double = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = ArcTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed = double.compose(add_one);

    let mut value = 5;
    assert_eq!(composed.transform(&mut value), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_arc_transformer_mut_with_state() {
    use std::sync::{Arc, Mutex};

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    let mut counting_double = ArcTransformerMut::new(move |x: &mut i32| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
        *x *= 2;
        *x + *count
    });

    let mut value1 = 10;
    assert_eq!(counting_double.transform(&mut value1), 21); // 10 * 2 + 1

    let mut value2 = 5;
    assert_eq!(counting_double.transform(&mut value2), 12); // 5 * 2 + 2
}

#[test]
fn test_arc_transformer_mut_thread_safety() {
    use std::sync::Arc as StdArc;
    use std::sync::Mutex;
    use std::thread;

    let transformer = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let transformer_mutex = StdArc::new(Mutex::new(transformer));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let t = StdArc::clone(&transformer_mutex);
            thread::spawn(move || {
                let mut transformer = t.lock().unwrap();
                let mut value = i;
                transformer.transform(&mut value)
            })
        })
        .collect();

    let results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(results, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
}

#[test]
fn test_arc_transformer_mut_as_function_mut() {
    let mut double = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    assert_eq!(double.apply(&mut value), 42);
}

#[test]
fn test_arc_transformer_mut_into_fn() {
    let double = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut closure = double.into_fn();
    let mut value1 = 21;
    assert_eq!(closure(&mut value1), 42);

    let mut value2 = 10;
    assert_eq!(closure(&mut value2), 20);
}

// ============================================================================
// RcTransformerMut Tests
// ============================================================================

#[test]
fn test_rc_transformer_mut_basic() {
    let mut double = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    assert_eq!(double.transform(&mut value), 42);
    assert_eq!(value, 42);
}

#[test]
fn test_rc_transformer_mut_clone() {
    let double = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut cloned = double.clone();
    let mut value = 21;
    assert_eq!(cloned.transform(&mut value), 42);
}

#[test]
fn test_rc_transformer_mut_identity() {
    let mut identity = RcTransformerMut::<i32>::identity();
    let mut value = 42;
    assert_eq!(identity.transform(&mut value), 42);
}

#[test]
fn test_rc_transformer_mut_and_then() {
    let double = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = RcTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed = double.and_then(add_one);

    let mut value = 5;
    assert_eq!(composed.transform(&mut value), 11); // 5 * 2 + 1 = 11
}

#[test]
fn test_rc_transformer_mut_compose() {
    let double = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = RcTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed = double.compose(add_one);

    let mut value = 5;
    assert_eq!(composed.transform(&mut value), 12); // (5 + 1) * 2 = 12
}

#[test]
fn test_rc_transformer_mut_with_state() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);

    let mut counting_double = RcTransformerMut::new(move |x: &mut i32| {
        let mut count = counter_clone.borrow_mut();
        *count += 1;
        *x *= 2;
        *x + *count
    });

    let mut value1 = 10;
    assert_eq!(counting_double.transform(&mut value1), 21); // 10 * 2 + 1

    let mut value2 = 5;
    assert_eq!(counting_double.transform(&mut value2), 12); // 5 * 2 + 2
}

#[test]
fn test_rc_transformer_mut_with_string() {
    let mut to_upper = RcTransformerMut::new(|s: &mut String| {
        *s = s.to_uppercase();
        s.clone()
    });

    let mut input = String::from("hello");
    assert_eq!(to_upper.transform(&mut input), "HELLO");
    assert_eq!(input, "HELLO");
}

#[test]
fn test_rc_transformer_mut_as_function_mut() {
    let mut double = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    assert_eq!(double.apply(&mut value), 42);
}

#[test]
fn test_rc_transformer_mut_into_fn() {
    let double = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut closure = double.into_fn();
    let mut value1 = 21;
    assert_eq!(closure(&mut value1), 42);

    let mut value2 = 10;
    assert_eq!(closure(&mut value2), 20);
}

// ============================================================================
// Blanket Implementation Tests
// ============================================================================

#[test]
fn test_closure_as_transformer_mut() {
    let mut double = |x: &mut i32| {
        *x *= 2;
        *x
    };

    let mut value = 21;
    assert_eq!(double.transform(&mut value), 42);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_transformer_mut_conversion() {
    let box_transformer = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let _rc_function = box_transformer.into_rc();
    // Note: Can't test further because transformer is consumed
}

#[test]
fn test_transformer_mut_with_complex_type() {
    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: i32,
    }

    let mut inc_age = BoxTransformerMut::new(|p: &mut Person| {
        p.age += 1;
        p.clone()
    });

    let mut person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let result = inc_age.transform(&mut person);
    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 31);
    assert_eq!(person.age, 31);
}
