/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    ArcBinaryOperator, BiTransformer, BiTransformerOnce, BinaryOperator, BoxBinaryOperator,
    BoxBinaryOperatorOnce, RcBinaryOperator,
};
use std::thread;

// Test using BinaryOperator as a generic constraint
fn reduce<T, O>(values: Vec<T>, initial: T, op: &O) -> T
where
    O: BinaryOperator<T>,
    T: Clone,
{
    values
        .into_iter()
        .fold(initial, |acc, val| op.apply(acc, val))
}

#[test]
fn test_binary_operator_basic() {
    let sum = |a: i32, b: i32| a + b;
    assert_eq!(reduce(vec![1, 2, 3, 4], 0, &sum), 10);
}

#[test]
fn test_box_binary_operator_creation() {
    let add: BoxBinaryOperator<i32> = BoxBinaryOperator::new(|a, b| a + b);
    assert_eq!(add.apply(20, 22), 42);
}

#[test]
fn test_arc_binary_operator_thread_safety() {
    let multiply = ArcBinaryOperator::new(|a: i32, b: i32| a * b);
    let multiply_clone = multiply.clone();

    let handle = thread::spawn(move || multiply_clone.apply(6, 7));

    assert_eq!(multiply.apply(8, 5), 40);
    assert_eq!(handle.join().unwrap(), 42);
}

#[test]
fn test_rc_binary_operator_clone() {
    let max = RcBinaryOperator::new(|a: i32, b: i32| if a > b { a } else { b });
    let cloned = max.clone();

    assert_eq!(max.apply(10, 20), 20);
    assert_eq!(cloned.apply(30, 15), 30);
}

#[test]
fn test_box_binary_operator_once() {
    let add: BoxBinaryOperatorOnce<i32> = BoxBinaryOperatorOnce::new(|a, b| a + b);
    assert_eq!(add.apply(20, 22), 42);
}

#[test]
fn test_binary_operator_compatibility() {
    fn use_bi_transformer<T: BiTransformer<i32, i32, i32>>(t: T, a: i32, b: i32) -> i32 {
        t.apply(a, b)
    }

    let binary_op = BoxBinaryOperator::new(|a: i32, b: i32| a + b);
    assert_eq!(use_bi_transformer(binary_op, 5, 10), 15);
}
