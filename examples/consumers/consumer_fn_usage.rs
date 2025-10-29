/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Demonstrates how into_fn and to_fn are used with function parameters that accept closures

use prism3_function::{
    ArcConsumer,
    BoxConsumer,
    Consumer,
    RcConsumer,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

fn main() {
    println!("=== Consumer into_fn/to_fn Usage Examples ===\n");

    // Example 1: Using BoxConsumer::into_fn to pass to standard library's map
    println!("1. BoxConsumer::into_fn used with Iterator::for_each");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let consumer = BoxConsumer::new(move |x: &i32| {
        l.lock().unwrap().push(*x * 2);
    });

    // Convert consumer to closure and pass to for_each
    [1, 2, 3, 4, 5].iter().for_each(consumer.into_fn());
    println!("   Result: {:?}\n", *log.lock().unwrap());

    // Example 2: Using ArcConsumer::to_fn can be used multiple times
    println!("2. ArcConsumer::to_fn can be used multiple times");
    let log2 = Arc::new(Mutex::new(Vec::new()));
    let l2 = log2.clone();
    let consumer2 = ArcConsumer::new(move |x: &i32| {
        l2.lock().unwrap().push(*x + 10);
    });

    // to_fn doesn't consume consumer, can be called multiple times
    [1, 2, 3].iter().for_each(consumer2.to_fn());
    println!("   First time: {:?}", *log2.lock().unwrap());

    [4, 5].iter().for_each(consumer2.to_fn());
    println!("   Second time: {:?}\n", *log2.lock().unwrap());

    // Example 3: Using RcConsumer::to_fn
    println!("3. RcConsumer::to_fn used for single-threaded scenarios");
    let log3 = Rc::new(RefCell::new(Vec::new()));
    let l3 = log3.clone();
    let consumer3 = RcConsumer::new(move |x: &i32| {
        l3.borrow_mut().push(*x * 3);
    });

    [1, 2, 3, 4].iter().for_each(consumer3.to_fn());
    println!("   Result: {:?}\n", *log3.borrow());

    // Example 4: Using in custom functions
    println!("4. Using in custom functions");
    fn process_items<F>(items: Vec<i32>, consumer: F)
    where
        F: FnMut(&i32),
    {
        items.iter().for_each(consumer);
    }

    let log4 = Arc::new(Mutex::new(Vec::new()));
    let l4 = log4.clone();
    let consumer4 = BoxConsumer::new(move |x: &i32| {
        l4.lock().unwrap().push(*x * 5);
    });

    // Use into_fn to convert Consumer to closure and pass to function
    process_items(vec![1, 2, 3], consumer4.into_fn());
    println!("   Result: {:?}\n", *log4.lock().unwrap());

    // Example 5: Using into_fn after chained operations
    println!("5. Using into_fn after chained operations");
    let log5 = Arc::new(Mutex::new(Vec::new()));
    let l5 = log5.clone();
    let l6 = log5.clone();

    let chained = BoxConsumer::new(move |x: &i32| {
        l5.lock().unwrap().push(format!("A: {}", x));
    })
    .and_then(move |x: &i32| {
        l6.lock().unwrap().push(format!("B: {}", x));
    });

    [1, 2].iter().for_each(chained.into_fn());
    println!("   Result: {:?}\n", *log5.lock().unwrap());

    println!("=== Demo Complete ===");
}
