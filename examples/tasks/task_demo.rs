// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::ArcCallable;
use qubit_function::BoxCallable;
use qubit_function::BoxCallableOnce;
use qubit_function::BoxCallableWith;
use qubit_function::BoxRunnable;
use qubit_function::BoxRunnableOnce;
use qubit_function::BoxRunnableWith;
use qubit_function::Callable;
use qubit_function::CallableOnce;
use qubit_function::CallableWith;
use qubit_function::Runnable;
use qubit_function::RunnableOnce;
use qubit_function::RunnableWith;

fn main() {
    println!("=== Task Demo ===\n");

    demo_reusable_tasks();
    demo_mutable_input_tasks();
    demo_once_tasks();
    demo_shared_callable();
}

fn demo_reusable_tasks() {
    println!("--- Reusable zero-argument tasks ---");

    let mut attempts = 0;
    let mut callable = BoxCallable::new(move || {
        attempts += 1;
        Ok::<i32, String>(attempts * 10)
    });
    println!("Callable first call: {:?}", callable.call());
    println!("Callable second call: {:?}", callable.call());

    let mut runnable = BoxRunnable::new(|| {
        println!("Runnable side effect executed");
        Ok::<(), String>(())
    });
    println!("Runnable result: {:?}", runnable.run());
    println!();
}

fn demo_mutable_input_tasks() {
    println!("--- Mutable-input tasks ---");

    let mut state = 40;
    let mut callable = BoxCallableWith::new(|input: &mut i32| {
        *input += 2;
        Ok::<i32, String>(*input)
    });
    println!("CallableWith result: {:?}", callable.call_with(&mut state));
    println!("State after CallableWith: {state}");

    let mut runnable = BoxRunnableWith::new(|input: &mut i32| {
        *input *= 2;
        Ok::<(), String>(())
    });
    println!("RunnableWith result: {:?}", runnable.run_with(&mut state));
    println!("State after RunnableWith: {state}");
    println!();
}

fn demo_once_tasks() {
    println!("--- One-time tasks ---");

    let callable_once = BoxCallableOnce::new(|| Ok::<String, String>(String::from("ready")));
    println!("CallableOnce result: {:?}", callable_once.call_once());

    let runnable_once = BoxRunnableOnce::new(|| {
        println!("RunnableOnce side effect executed");
        Ok::<(), String>(())
    });
    println!("RunnableOnce result: {:?}", runnable_once.run_once());
    println!();
}

fn demo_shared_callable() {
    println!("--- Shared callable ---");

    let mut call_count = 0;
    let shared = ArcCallable::new(move || {
        call_count += 1;
        Ok::<usize, String>(call_count)
    });

    let mut first = shared.clone();
    let mut second = shared.clone();
    println!("ArcCallable first clone: {:?}", first.call());
    println!("ArcCallable second clone: {:?}", second.call());
}
