// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Demonstrates adapting consumer objects to APIs that accept closures.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::ArcConsumer;
use qubit_function::BoxConsumer;
use qubit_function::Consumer;
use qubit_function::RcConsumer;

fn main() {
    println!("=== Consumer Closure Interoperability Examples ===\n");

    // Example 1: Adapt BoxConsumer to Iterator::for_each.
    println!("1. BoxConsumer used with Iterator::for_each");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let consumer = BoxConsumer::new(move |x: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x * 2);
    });

    // Convert consumer to closure and pass to for_each
    [1, 2, 3, 4, 5].iter().for_each(move |value| consumer.accept(value));
    println!("   Result: {:?}\n", *log.lock().expect("mutex should not be poisoned"));

    // Example 2: A shared ArcConsumer can be reused.
    println!("2. ArcConsumer can be used multiple times");
    let log2 = Arc::new(Mutex::new(Vec::new()));
    let l2 = log2.clone();
    let consumer2 = ArcConsumer::new(move |x: &i32| {
        l2.lock().expect("mutex should not be poisoned").push(*x + 10);
    });

    // Calling through a borrowed ArcConsumer does not consume it.
    [1, 2, 3].iter().for_each(|value| consumer2.accept(value));
    println!(
        "   First time: {:?}",
        *log2.lock().expect("mutex should not be poisoned")
    );

    [4, 5].iter().for_each(|value| consumer2.accept(value));
    println!(
        "   Second time: {:?}\n",
        *log2.lock().expect("mutex should not be poisoned")
    );

    // Example 3: Adapt RcConsumer in a single-threaded scenario.
    println!("3. RcConsumer used for single-threaded scenarios");
    let log3 = Rc::new(RefCell::new(Vec::new()));
    let l3 = log3.clone();
    let consumer3 = RcConsumer::new(move |x: &i32| {
        l3.borrow_mut().push(*x * 3);
    });

    [1, 2, 3, 4].iter().for_each(|value| consumer3.accept(value));
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
        l4.lock().expect("mutex should not be poisoned").push(*x * 5);
    });

    // Adapt the consumer with a forwarding closure.
    process_items(vec![1, 2, 3], move |value| consumer4.accept(value));
    println!("   Result: {:?}\n", *log4.lock().expect("mutex should not be poisoned"));

    // Example 5: Adapt a composed consumer.
    println!("5. Using a forwarding closure after chained operations");
    let log5 = Arc::new(Mutex::new(Vec::new()));
    let l5 = log5.clone();
    let l6 = log5.clone();

    let chained = BoxConsumer::new(move |x: &i32| {
        l5.lock()
            .expect("mutex should not be poisoned")
            .push(format!("A: {}", x));
    })
    .and_then(move |x: &i32| {
        l6.lock()
            .expect("mutex should not be poisoned")
            .push(format!("B: {}", x));
    });

    [1, 2].iter().for_each(move |value| chained.accept(value));
    println!("   Result: {:?}\n", *log5.lock().expect("mutex should not be poisoned"));

    println!("=== Demo Complete ===");
}
