/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    ArcUnaryOperator, BoxUnaryOperator, BoxUnaryOperatorOnce, RcUnaryOperator, Transformer,
    TransformerOnce, UnaryOperator,
};
use std::thread;

// Test using UnaryOperator as a generic constraint
fn apply_twice<T, O>(value: T, op: &O) -> T
where
    O: UnaryOperator<T>,
    T: Clone,
{
    let result = op.apply(value.clone());
    op.apply(result)
}

#[test]
fn test_unary_operator_basic() {
    let increment = |x: i32| x + 1;
    assert_eq!(apply_twice(5, &increment), 7);
}

#[test]
fn test_box_unary_operator_creation() {
    let increment: BoxUnaryOperator<i32> = BoxUnaryOperator::new(|x| x + 1);
    assert_eq!(increment.apply(41), 42);
}

#[test]
fn test_arc_unary_operator_thread_safety() {
    let square = ArcUnaryOperator::new(|x: i32| x * x);
    let square_clone = square.clone();

    let handle = thread::spawn(move || square_clone.apply(5));

    assert_eq!(square.apply(3), 9);
    assert_eq!(handle.join().unwrap(), 25);
}

#[test]
fn test_rc_unary_operator_clone() {
    let negate: RcUnaryOperator<i32> = RcUnaryOperator::new(|x: i32| -x);
    let cloned = negate.clone();

    assert_eq!(negate.apply(42), -42);
    assert_eq!(cloned.apply(-20), 20);
}

#[test]
fn test_box_unary_operator_once() {
    let double: BoxUnaryOperatorOnce<i32> = BoxUnaryOperatorOnce::new(|x| x * 2);
    assert_eq!(double.apply_once(21), 42);
}
