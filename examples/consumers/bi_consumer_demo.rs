// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! BiConsumer demonstration
//!
//! This example demonstrates the usage of BiConsumer types after
//! refactoring to use &T, &U semantics (not modifying input values).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use qubit_function::ArcBiConsumer;
use qubit_function::BiConsumer;
use qubit_function::BoxBiConsumer;
use qubit_function::BoxStatefulBiConsumer;
use qubit_function::RcBiConsumer;
use qubit_function::StatefulBiConsumer;

fn main() {
    println!("=== BiConsumer Demo ===\n");

    // 1. BoxBiConsumer - Single ownership
    println!("1. BoxBiConsumer - Single ownership:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let box_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
        println!("  Processing: x={}, y={}", x, y);
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
    });
    box_consumer.accept(&10, &5);
    println!(
        "  Result log: {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 2. Method chaining with BoxBiConsumer
    println!("2. BoxBiConsumer with method chaining:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l1 = log.clone();
    let l2 = log.clone();
    let chained = BoxBiConsumer::new(move |x: &i32, y: &i32| {
        l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        println!("  After first operation: sum = {}", x + y);
    })
    .and_then(move |x: &i32, y: &i32| {
        l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        println!("  After second operation: product = {}", x * y);
    });
    chained.accept(&5, &3);
    println!(
        "  Final log: {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 3. ArcBiConsumer - Thread-safe shared ownership
    println!("3. ArcBiConsumer - Thread-safe shared ownership:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let arc_consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
        println!("  Thread {:?}: sum = {}", thread::current().id(), x + y);
    });

    let consumer1 = arc_consumer.clone();
    let consumer2 = arc_consumer.clone();

    let handle1 = thread::spawn(move || {
        let c = consumer1;
        c.accept(&10, &5);
    });

    let handle2 = thread::spawn(move || {
        let c = consumer2;
        c.accept(&20, &8);
    });

    handle1.join().expect("thread should not panic");
    handle2.join().expect("thread should not panic");
    println!(
        "  Final log: {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 4. RcBiConsumer - Single-threaded shared ownership
    println!("4. RcBiConsumer - Single-threaded shared ownership:");
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();
    let rc_consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
        l.borrow_mut().push(*x + *y);
    });

    let clone1 = rc_consumer.clone();
    let clone2 = rc_consumer.clone();

    clone1.accept(&5, &3);
    println!("  After first use: {:?}", *log.borrow());

    clone2.accept(&7, &2);
    println!("  After second use: {:?}\n", *log.borrow());

    // 5. Working with closures directly
    println!("5. Working with closures directly:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let closure = move |x: &i32, y: &i32| {
        let sum = *x + *y;
        l.lock().expect("mutex should not be poisoned").push(sum);
    };
    closure.accept(&10, &20);
    println!(
        "  After closure: {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 6. Conditional BiConsumer
    println!("6. Conditional BiConsumer:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let mut conditional = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0);

    conditional.accept(&5, &3);
    println!(
        "  Positive values: {:?}",
        *log.lock().expect("mutex should not be poisoned")
    );

    conditional.accept(&-5, &3);
    println!(
        "  Negative value (unchanged): {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 7. Conditional branch BiConsumer
    println!("7. Conditional branch BiConsumer:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l1 = log.clone();
    let l2 = log.clone();
    let mut branch = BoxStatefulBiConsumer::new(move |x: &i32, _y: &i32| {
        l1.lock().expect("mutex should not be poisoned").push(*x);
    })
    .when(|x: &i32, y: &i32| *x > *y)
    .or_else(move |_x: &i32, y: &i32| {
        l2.lock().expect("mutex should not be poisoned").push(*y);
    });

    branch.accept(&15, &10);
    println!("  When x > y: {:?}", *log.lock().expect("mutex should not be poisoned"));

    branch.accept(&5, &10);
    println!(
        "  When x <= y: {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 8. Accumulating statistics
    println!("8. Accumulating statistics:");
    let count = Arc::new(Mutex::new(0));
    let sum = Arc::new(Mutex::new(0));
    let c = count.clone();
    let s = sum.clone();
    let stats_consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
        *c.lock().expect("mutex should not be poisoned") += 1;
        *s.lock().expect("mutex should not be poisoned") += x + y;
    });

    stats_consumer.accept(&5, &3);
    stats_consumer.accept(&10, &2);
    stats_consumer.accept(&7, &8);

    println!("  Count: {}", *count.lock().expect("mutex should not be poisoned"));
    println!("  Sum: {}\n", *sum.lock().expect("mutex should not be poisoned"));

    // 9. Name support
    println!("9. Name support:");
    let mut named_consumer = BoxBiConsumer::<i32, i32>::noop();
    println!("  Initial name: {:?}", named_consumer.name());

    named_consumer.set_name("sum_calculator");
    println!("  After setting name: {:?}", named_consumer.name());
    println!("  Display: {}\n", named_consumer);

    println!("=== Demo Complete ===");
}
