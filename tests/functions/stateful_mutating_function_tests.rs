/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for StatefulMutatingFunction types (stateful FnMut(&mut T) ->
//! R)

use prism3_function::{
    ArcStatefulMutatingFunction,
    BoxStatefulMutatingFunction,
    FnMutStatefulMutatingFunctionOps,
    RcStatefulMutatingFunction,
    StatefulMutatingFunction,
};

// ============================================================================
// BoxStatefulMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_box_stateful_mutating_function {
    use super::*;

    #[test]
    fn test_new() {
        let mut counter = {
            let mut count = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x += 1;
                count
            })
        };
        let mut value = 5;
        assert_eq!(counter.apply(&mut value), 1);
        assert_eq!(value, 6);
        assert_eq!(counter.apply(&mut value), 2);
        assert_eq!(value, 7);
    }

    #[test]
    fn test_accumulator() {
        let mut accumulator = {
            let mut sum = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                *x *= 2;
                sum += *x;
                sum
            })
        };

        let mut value = 5;
        assert_eq!(accumulator.apply(&mut value), 10);
        assert_eq!(value, 10);

        let mut value2 = 3;
        assert_eq!(accumulator.apply(&mut value2), 16); // 10 + 6
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = {
            let mut count1 = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count1 += 1;
                *x *= 2;
                count1
            })
        };
        let second = {
            let mut count2 = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count2 += 1;
                *x += 10;
                count2
            })
        };

        let mut chained = first.and_then(second);
        let mut value = 5;
        assert_eq!(chained.apply(&mut value), 1);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_identity() {
        let mut identity = BoxStatefulMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = {
            let mut count = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut mapped = func.map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }
}

// ============================================================================
// RcStatefulMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_rc_stateful_mutating_function {
    use super::*;

    #[test]
    fn test_new() {
        let mut counter = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x += 1;
                count
            })
        };
        let mut value = 5;
        assert_eq!(counter.apply(&mut value), 1);
        assert_eq!(value, 6);
        assert_eq!(counter.apply(&mut value), 2);
        assert_eq!(value, 7);
    }

    #[test]
    fn test_clone() {
        let counter = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut clone = counter.clone();

        let mut value1 = 5;
        assert_eq!(clone.apply(&mut value1), 1);
        assert_eq!(value1, 10);

        // Shared state
        let mut value2 = 3;
        assert_eq!(clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = {
            let mut count1 = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count1 += 1;
                *x *= 2;
                count1
            })
        };
        let second = {
            let mut count2 = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count2 += 1;
                *x += 10;
                count2
            })
        };

        let mut combined = first.and_then(&second);
        let mut value = 5;
        assert_eq!(combined.apply(&mut value), 1);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_identity() {
        let mut identity = RcStatefulMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut mapped = func.map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }
}

// ============================================================================
// ArcStatefulMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_arc_stateful_mutating_function {
    use super::*;
    use std::thread;

    #[test]
    fn test_new() {
        let mut counter = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x += 1;
                count
            })
        };
        let mut value = 5;
        assert_eq!(counter.apply(&mut value), 1);
        assert_eq!(value, 6);
        assert_eq!(counter.apply(&mut value), 2);
        assert_eq!(value, 7);
    }

    #[test]
    fn test_clone() {
        let counter = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut clone = counter.clone();

        let mut value1 = 5;
        assert_eq!(clone.apply(&mut value1), 1);
        assert_eq!(value1, 10);

        // Shared state
        let mut value2 = 3;
        assert_eq!(clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_thread_safe() {
        let counter = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut counter_clone = counter.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            counter_clone.apply(&mut value)
        });

        let result = handle.join().unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_and_then() {
        let first = {
            let mut count1 = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count1 += 1;
                *x *= 2;
                count1
            })
        };
        let second = {
            let mut count2 = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count2 += 1;
                *x += 10;
                count2
            })
        };

        let mut combined = first.and_then(&second);
        let mut value = 5;
        assert_eq!(combined.apply(&mut value), 1);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_identity() {
        let mut identity = ArcStatefulMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut mapped = func.map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }
}

// ============================================================================
// Closure Tests
// ============================================================================

#[cfg(test)]
mod test_closure {
    use super::*;

    #[test]
    fn test_closure_implements_trait() {
        let mut count = 0;
        let mut closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };

        let mut value = 5;
        assert_eq!(closure.apply(&mut value), 1);
        assert_eq!(value, 10);
        assert_eq!(closure.apply(&mut value), 2);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_closure_and_then() {
        let mut count1 = 0;
        let mut count2 = 0;
        let mut chained = (move |x: &mut i32| {
            count1 += 1;
            *x *= 2;
            count1
        })
        .and_then(move |x: &mut i32| {
            count2 += 1;
            *x += 10;
            count2
        });

        let mut value = 5;
        assert_eq!(chained.apply(&mut value), 1);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_closure_map() {
        let mut count = 0;
        let mut mapped = (move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        })
        .map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_box() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut boxed = closure.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_rc() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut rc = closure.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_arc() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut arc = closure.into_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }
}
