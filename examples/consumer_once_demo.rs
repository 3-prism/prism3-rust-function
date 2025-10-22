/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # ConsumerOnce Demo
//!
//! Demonstrates the usage of ConsumerOnce trait and its implementations.

use prism3_function::{BoxConsumerOnce, ConsumerOnce, FnConsumerOnceOps};
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== ConsumerOnce Demo ===\n");

    // 1. BoxConsumerOnce - Single ownership, one-time use
    println!("1. BoxConsumerOnce - Single ownership");
    {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
            println!("  BoxConsumerOnce consumed: {}", x);
        });
        consumer.accept_once(&42);
        println!("  Log: {:?}\n", *log.lock().unwrap());
    }

    // 2. BoxConsumerOnce - Method chaining
    println!("2. BoxConsumerOnce - Method chaining");
    {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let chained = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
            println!("  Step 1: {} * 2 = {}", x, x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
            println!("  Step 2: {} + 10 = {}", x, x + 10);
        })
        .and_then(move |x: &i32| {
            l3.lock().unwrap().push(*x - 1);
            println!("  Step 3: {} - 1 = {}", x, x - 1);
        });
        chained.accept_once(&5);
        println!("  Log: {:?}\n", *log.lock().unwrap());
    }

    // 3. BoxConsumerOnce - Factory methods
    println!("3. BoxConsumerOnce - Factory methods");
    {
        // No-op consumer
        let noop = BoxConsumerOnce::<i32>::noop();
        noop.accept_once(&42);
        println!("  No-op consumer executed (no output)");

        // Print consumer
        print!("  Print consumer: ");
        let print = BoxConsumerOnce::new(|x: &i32| println!("{}", x));
        print.accept_once(&42);

        // Print with prefix
        print!("  Print with prefix: ");
        let print_with = BoxConsumerOnce::new(|x: &i32| println!("Value: {}", x));
        print_with.accept_once(&42);

        // Conditional consumer
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let conditional = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        })
        .when(|x: &i32| *x > 0);
        conditional.accept_once(&5);
        println!("  Conditional (positive): {:?}", *log.lock().unwrap());

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let conditional = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        })
        .when(|x: &i32| *x > 0);
        conditional.accept_once(&-5);
        println!("  Conditional (negative): {:?}\n", *log.lock().unwrap());
    }

    // 4. Closure usage
    println!("4. Closure usage");
    {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
            println!("  Closure consumed: {}", x);
        };
        closure.accept_once(&42);
        println!("  Log: {:?}\n", *log.lock().unwrap());
    }

    // 5. Closure chaining
    println!("5. Closure chaining");
    {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = (move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
            println!("  Closure 1: {} * 2 = {}", x, x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
            println!("  Closure 2: {} + 10 = {}", x, x + 10);
        });
        chained.accept_once(&5);
        println!("  Log: {:?}\n", *log.lock().unwrap());
    }

    // 6. Type conversions
    println!("6. Type conversions");
    {
        let log = Arc::new(Mutex::new(Vec::new()));

        // Closure to BoxConsumerOnce
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x);
        };
        let box_consumer = closure.into_box_once();
        box_consumer.accept_once(&1);
        println!("  BoxConsumerOnce: {:?}", *log.lock().unwrap());
    }

    // 7. Using with iterators (BoxConsumerOnce)
    println!("7. Using with iterators");
    {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        // Note: This will panic because BoxConsumerOnce can only be called once
        // vec![1, 2, 3, 4, 5].iter().for_each(consumer.into_fn_once());
        consumer.accept_once(&1);
        println!(
            "  BoxConsumerOnce with single value: {:?}\n",
            *log.lock().unwrap()
        );
    }

    println!("=== Demo Complete ===");
}
