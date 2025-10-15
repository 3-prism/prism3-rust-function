/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    ArcFunction, ArcFunctionMut, ArcFunctionOnce, BoxFunction, BoxFunctionMut,
    BoxFunctionOnce, Function, FunctionMut, FunctionOnce, RcFunction,
    RcFunctionMut, RcFunctionOnce,
};
use std::thread;

// ============================================================================
// BoxFunction Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod box_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = BoxFunction::new(|x: &i32| x * 2);
        let value = 21;
        assert_eq!(double.apply(&value), 42);
        assert_eq!(value, 21); // Value is still usable
    }

    #[test]
    fn test_multiple_calls() {
        let double = BoxFunction::new(|x: &i32| x * 2);
        assert_eq!(double.apply(&21), 42);
        assert_eq!(double.apply(&42), 84);
        assert_eq!(double.apply(&10), 20);
    }

    #[test]
    fn test_identity() {
        let identity = BoxFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(&42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxFunction::constant("hello");
        assert_eq!(constant.apply(&123), "hello");
        assert_eq!(constant.apply(&456), "hello");
    }

    #[test]
    fn test_with_string() {
        let len = BoxFunction::new(|s: &String| s.len());
        let text = "hello".to_string();
        assert_eq!(len.apply(&text), 5);
        assert_eq!(text, "hello"); // String is still usable
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let multiply = BoxFunction::new(move |x: &i32| x * multiplier);
        assert_eq!(multiply.apply(&7), 21);
    }
}

// ============================================================================
// ArcFunction Tests - Immutable, thread-safe
// ============================================================================

#[cfg(test)]
mod arc_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = ArcFunction::new(|x: &i32| x * 2);
        let value = 21;
        assert_eq!(double.apply(&value), 42);
    }

    #[test]
    fn test_clone() {
        let double = ArcFunction::new(|x: &i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(&21), 42);
        assert_eq!(cloned.apply(&21), 42);
    }

    #[test]
    fn test_thread_safe() {
        let double = ArcFunction::new(|x: &i32| x * 2);
        let cloned = double.clone();

        let handle = thread::spawn(move || cloned.apply(&21));

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(double.apply(&21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = ArcFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(&42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcFunction::constant("hello");
        assert_eq!(constant.apply(&123), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let square = ArcFunction::new(|x: &i32| x * x);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let sq = square.clone();
                thread::spawn(move || sq.apply(&i))
            })
            .collect();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        assert_eq!(results, vec![0, 1, 4, 9]);
    }
}

// ============================================================================
// RcFunction Tests - Immutable, single-threaded
// ============================================================================

#[cfg(test)]
mod rc_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = RcFunction::new(|x: &i32| x * 2);
        let value = 21;
        assert_eq!(double.apply(&value), 42);
    }

    #[test]
    fn test_clone() {
        let double = RcFunction::new(|x: &i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(&21), 42);
        assert_eq!(cloned.apply(&21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = RcFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(&42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcFunction::constant("hello");
        assert_eq!(constant.apply(&123), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let to_upper = RcFunction::new(|s: &String| s.to_uppercase());

        let text = "hello".to_string();
        let func1 = to_upper.clone();
        let func2 = to_upper.clone();

        assert_eq!(to_upper.apply(&text), "HELLO");
        assert_eq!(func1.apply(&text), "HELLO");
        assert_eq!(func2.apply(&text), "HELLO");
    }
}

// ============================================================================
// BoxFunctionMut Tests - Mutable, single ownership
// ============================================================================

#[cfg(test)]
mod box_function_mut_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let mut double_in_place = BoxFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(double_in_place.apply(&mut value), 42);
        assert_eq!(value, 42); // Value was modified
    }

    #[test]
    fn test_multiple_calls() {
        let mut increment = BoxFunctionMut::new(|x: &mut i32| {
            *x += 1;
            *x
        });

        let mut value = 0;
        assert_eq!(increment.apply(&mut value), 1);
        assert_eq!(increment.apply(&mut value), 2);
        assert_eq!(increment.apply(&mut value), 3);
        assert_eq!(value, 3);
    }

    #[test]
    fn test_stateful_function() {
        let mut count = 0;
        let mut counter = BoxFunctionMut::new(move |x: &mut i32| {
            count += 1;
            *x += count;
            (*x, count)
        });

        let mut value = 10;
        assert_eq!(counter.apply(&mut value), (11, 1));
        assert_eq!(counter.apply(&mut value), (13, 2));
        assert_eq!(counter.apply(&mut value), (16, 3));
    }

    #[test]
    fn test_with_string() {
        let mut append = BoxFunctionMut::new(|s: &mut String| {
            s.push_str("!");
            s.len()
        });

        let mut text = "hello".to_string();
        assert_eq!(append.apply(&mut text), 6);
        assert_eq!(text, "hello!");
    }
}

// ============================================================================
// ArcFunctionMut Tests - Mutable, thread-safe
// ============================================================================

#[cfg(test)]
mod arc_function_mut_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let mut double_in_place = ArcFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(double_in_place.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_clone() {
        let mut increment = ArcFunctionMut::new(|x: &mut i32| {
            *x += 1;
            *x
        });

        let mut cloned = increment.clone();

        let mut val1 = 0;
        let mut val2 = 10;

        assert_eq!(increment.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 11);
    }

    #[test]
    fn test_stateful_shared() {
        let mut count = 0;
        let mut counter = ArcFunctionMut::new(move |x: &mut i32| {
            count += 1;
            *x += count;
            count
        });

        let mut cloned = counter.clone();

        let mut val1 = 0;
        let mut val2 = 0;

        // Both share the same internal state via Mutex
        assert_eq!(counter.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 2);
        assert_eq!(counter.apply(&mut val1), 3);
    }
}

// ============================================================================
// RcFunctionMut Tests - Mutable, single-threaded
// ============================================================================

#[cfg(test)]
mod rc_function_mut_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let mut double_in_place = RcFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(double_in_place.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_clone() {
        let mut increment = RcFunctionMut::new(|x: &mut i32| {
            *x += 1;
            *x
        });

        let mut cloned = increment.clone();

        let mut val1 = 0;
        let mut val2 = 10;

        assert_eq!(increment.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 11);
    }

    #[test]
    fn test_stateful_shared() {
        let mut count = 0;
        let mut counter = RcFunctionMut::new(move |x: &mut i32| {
            count += 1;
            *x += count;
            count
        });

        let mut cloned = counter.clone();

        let mut val1 = 0;
        let mut val2 = 0;

        // Both share the same internal state via RefCell
        assert_eq!(counter.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 2);
        assert_eq!(counter.apply(&mut val1), 3);
    }
}

// ============================================================================
// BoxFunctionOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_function_once_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let parse = BoxFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxFunctionOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxFunctionOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxFunctionOnce::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
        let pipeline = add_one
            .and_then(|x| x * 2)
            .and_then(|x| x - 3);
        assert_eq!(pipeline.apply(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxFunctionOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.apply("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxFunctionOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.apply(42), "42_suffix");
    }
}

// ============================================================================
// ArcFunctionOnce Tests - Consuming, thread-safe, reusable
// ============================================================================

#[cfg(test)]
mod arc_function_once_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let parse = ArcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_clone_and_reuse() {
        let parse = ArcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        let cloned = parse.clone();

        // Can use both (but each consumes its input)
        assert_eq!(parse.apply("42".to_string()), 42);
        assert_eq!(cloned.apply("21".to_string()), 21);
    }

    #[test]
    fn test_identity() {
        let identity = ArcFunctionOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcFunctionOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_thread_safe() {
        let parse = ArcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        let cloned = parse.clone();

        let handle = thread::spawn(move || {
            cloned.apply("42".to_string())
        });

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(parse.apply("21".to_string()), 21);
    }
}

// ============================================================================
// RcFunctionOnce Tests - Consuming, single-threaded, reusable
// ============================================================================

#[cfg(test)]
mod rc_function_once_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let parse = RcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_clone_and_reuse() {
        let parse = RcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        let cloned = parse.clone();

        assert_eq!(parse.apply("42".to_string()), 42);
        assert_eq!(cloned.apply("21".to_string()), 21);
    }

    #[test]
    fn test_identity() {
        let identity = RcFunctionOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcFunctionOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let to_upper = RcFunctionOnce::new(|s: String| s.to_uppercase());

        let func1 = to_upper.clone();
        let func2 = to_upper.clone();

        assert_eq!(to_upper.apply("hello".to_string()), "HELLO");
        assert_eq!(func1.apply("world".to_string()), "WORLD");
        assert_eq!(func2.apply("rust".to_string()), "RUST");
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_function_trait() {
        fn apply_function<F: Function<i32, i32>>(f: &F, x: &i32) -> i32 {
            f.apply(x)
        }

        let double = BoxFunction::new(|x: &i32| x * 2);
        assert_eq!(apply_function(&double, &21), 42);
    }

    #[test]
    fn test_function_mut_trait() {
        fn apply_function_mut<F: FunctionMut<i32, i32>>(
            f: &mut F,
            x: &mut i32,
        ) -> i32 {
            f.apply(x)
        }

        let mut double = BoxFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(apply_function_mut(&mut double, &mut value), 42);
    }

    #[test]
    fn test_function_once_trait() {
        fn apply_function_once<F: FunctionOnce<i32, i32>>(
            f: F,
            x: i32,
        ) -> i32 {
            f.apply(x)
        }

        let double = BoxFunctionOnce::new(|x: i32| x * 2);
        assert_eq!(apply_function_once(double, 21), 42);
    }
}
