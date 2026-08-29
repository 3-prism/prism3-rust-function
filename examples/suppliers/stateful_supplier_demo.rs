// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Demonstration of StatefulSupplier types usage

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use qubit_function::ArcStatefulSupplier;
use qubit_function::ArcSupplier;
use qubit_function::BoxStatefulSupplier;
use qubit_function::BoxSupplier;
use qubit_function::BoxSupplierOnce;
use qubit_function::RcStatefulSupplier;
use qubit_function::RcSupplier;
use qubit_function::StatefulSupplier;
use qubit_function::Supplier;
use qubit_function::SupplierOnce;

fn is_even_i32(value: &i32) -> bool {
    value % 2 == 0
}

fn main() {
    println!("=== Supplier Demo ===\n");

    demo_closure_supplier();
    demo_box_supplier_basics();
    demo_box_supplier_methods();
    demo_box_supplier_once();
    demo_arc_supplier();
    demo_arc_supplier_threading();
    demo_rc_supplier();
    demo_type_conversions();
}

fn demo_closure_supplier() {
    println!("--- Closure as Supplier ---");

    // Simple closure (Fn)
    let closure = || 42;
    let boxed = BoxSupplier::new(closure);
    println!("Closure: {}", boxed.get());

    // Stateful closure (FnMut) - use StatefulSupplier
    let mut counter = 0;
    let stateful = move || {
        counter += 1;
        counter
    };
    let mut boxed_stateful = BoxStatefulSupplier::new(stateful);
    println!("Counter: {}", boxed_stateful.get());
    println!("Counter: {}", boxed_stateful.get());
    println!("Counter: {}", boxed_stateful.get());
    println!();
}

fn demo_box_supplier_basics() {
    println!("--- BoxSupplier Basics ---");

    // Basic usage (Fn)
    let supplier = BoxSupplier::new(|| 42);
    println!("Basic: {}", supplier.get());

    // Constant supplier (Fn)
    let constant = BoxSupplier::constant(100);
    println!("Constant: {}", constant.get());
    println!("Constant: {}", constant.get());

    // Stateful counter (FnMut) - use BoxStatefulSupplier
    let mut counter = 0;
    let mut counter_supplier = BoxStatefulSupplier::new(move || {
        counter += 1;
        counter
    });
    println!("Counter: {}", counter_supplier.get());
    println!("Counter: {}", counter_supplier.get());
    println!("Counter: {}", counter_supplier.get());
    println!();
}

fn demo_box_supplier_methods() {
    println!("--- BoxStatefulSupplier Methods ---");

    // Map (FnMut)
    let mut counter = 0;
    let mut mapped = BoxStatefulSupplier::new(move || {
        counter += 1;
        counter * 10
    })
    .map(|x| x + 5);
    println!("Mapped: {}", mapped.get());
    println!("Mapped: {}", mapped.get());

    // Filter (FnMut)
    let mut counter = 0;
    let mut filtered = BoxStatefulSupplier::new(move || {
        counter += 1;
        counter
    })
    .filter(is_even_i32);
    println!("Filtered (odd): {:?}", filtered.get());
    println!("Filtered (even): {:?}", filtered.get());

    // Zip (Fn)
    let first = BoxSupplier::new(|| 42);
    let second = BoxSupplier::new(|| "hello");
    let zipped = first.zip(second);
    println!("Zipped: {:?}", zipped.get());

    // Memoize (FnMut)
    let mut call_count = 0;
    let mut memoized = BoxStatefulSupplier::new(move || {
        call_count += 1;
        println!("  Expensive computation #{}", call_count);
        42
    })
    .memoize();
    println!("First call: {}", memoized.get());
    println!("Second call (cached): {}", memoized.get());
    println!();
}

fn demo_box_supplier_once() {
    println!("--- BoxSupplierOnce ---");

    // Basic usage
    let once = BoxSupplierOnce::new(|| {
        println!("  Expensive initialization");
        42
    });
    println!("Value: {}", once.get());

    // Moving captured values
    let data = String::from("Hello, World!");
    let once = BoxSupplierOnce::new(move || data);
    println!("Moved data: {}", once.get());
    println!();
}

fn demo_arc_supplier() {
    println!("--- ArcSupplier ---");

    // Basic usage (Fn)
    let supplier = ArcSupplier::new(|| 42);
    let s = supplier;
    println!("Basic: {}", s.get());

    // Reusable transformations (Fn)
    let source = ArcSupplier::new(|| 10);
    let doubled = source.map(|x| x * 2);
    let tripled = source.map(|x| x * 3);

    let s = source;
    let d = doubled;
    let t = tripled;
    println!("Source: {}", s.get());
    println!("Doubled: {}", d.get());
    println!("Tripled: {}", t.get());

    // Stateful with ArcStatefulSupplier (FnMut)
    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = Arc::clone(&call_count);
    let source = ArcStatefulSupplier::new(move || {
        let mut c = call_count_clone.lock().expect("mutex should not be poisoned");
        *c += 1;
        println!("  Computation #{}", *c);
        42
    });

    // Memoization with ArcStatefulSupplier
    let memoized = source.memoize();
    let mut m = memoized;
    println!("First call: {}", m.get());
    println!("Second call (cached): {}", m.get());
    println!(
        "Call count: {}",
        *call_count.lock().expect("mutex should not be poisoned")
    );
    println!();
}

fn demo_arc_supplier_threading() {
    println!("--- ArcStatefulSupplier Threading ---");

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    let supplier = ArcStatefulSupplier::new(move || {
        let mut c = counter_clone.lock().expect("mutex should not be poisoned");
        *c += 1;
        *c
    });

    let mut s1 = supplier.clone();
    let mut s2 = supplier.clone();
    let mut s3 = supplier;

    let h1 = thread::spawn(move || {
        let v1 = s1.get();
        let v2 = s1.get();
        println!("Thread 1: {} {}", v1, v2);
        (v1, v2)
    });

    let h2 = thread::spawn(move || {
        let v1 = s2.get();
        let v2 = s2.get();
        println!("Thread 2: {} {}", v1, v2);
        (v1, v2)
    });

    let v1 = s3.get();
    let v2 = s3.get();
    println!("Main thread: {} {}", v1, v2);

    h1.join().expect("thread should not panic");
    h2.join().expect("thread should not panic");

    println!(
        "Final counter: {}",
        *counter.lock().expect("mutex should not be poisoned")
    );
    println!();
}

fn demo_rc_supplier() {
    println!("--- RcSupplier ---");

    // Basic usage (Fn)
    let supplier = RcSupplier::new(|| 42);
    let s = supplier;
    println!("Basic: {}", s.get());

    // Shared state (FnMut) - use RcStatefulSupplier
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let supplier = RcStatefulSupplier::new(move || {
        let mut c = counter_clone.borrow_mut();
        *c += 1;
        *c
    });

    let mut s1 = supplier.clone();
    let mut s2 = supplier.clone();

    println!("First clone: {}", s1.get());
    println!("Second clone: {}", s2.get());
    println!("First clone again: {}", s1.get());

    // Reusable transformations (Fn)
    let source = RcSupplier::new(|| 10);
    let doubled = source.map(|x| x * 2);
    let tripled = source.map(|x| x * 3);
    let squared = source.map(|x| x * x);

    let s = source;
    let d = doubled;
    let t = tripled;
    let sq = squared;

    println!("Source: {}", s.get());
    println!("Doubled: {}", d.get());
    println!("Tripled: {}", t.get());
    println!("Squared: {}", sq.get());
    println!();
}

fn demo_type_conversions() {
    println!("--- Type Conversions ---");

    // Closure to Box (Fn)
    let closure = || 42;
    let boxed = BoxSupplier::new(closure);
    println!("Closure -> Box: {}", boxed.get());

    // Closure to Rc (Fn)
    let closure = || 100;
    let rc = RcSupplier::new(closure);
    println!("Closure -> Rc: {}", rc.get());

    // Closure to Arc (Fn)
    let closure = || 200;
    let arc = ArcSupplier::new(closure);
    println!("Closure -> Arc: {}", arc.get());

    // Box to Rc (Fn)
    let boxed = BoxSupplier::new(|| 42);
    let rc = RcSupplier::new(boxed);
    println!("Box -> Rc: {}", rc.get());

    // Arc to Box (Fn)
    let arc = ArcSupplier::new(|| 42);
    let boxed = BoxSupplier::new(arc);
    println!("Arc -> Box: {}", boxed.get());

    // Rc to Box (Fn)
    let rc = RcSupplier::new(|| 42);
    let boxed = BoxSupplier::new(rc);
    println!("Rc -> Box: {}", boxed.get());

    println!();
}
