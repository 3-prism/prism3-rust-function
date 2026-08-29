// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for callable task types.

use std::cell::Cell;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_function::ArcCallable;
use qubit_function::BoxCallable;
use qubit_function::BoxCallableOnce;
use qubit_function::Callable;
use qubit_function::CallableOnce;
use qubit_function::RcCallable;
use qubit_function::SupplierOnce;

#[derive(Clone)]
struct ClonedCallable {
    value: i32,
}

impl Callable<i32, io::Error> for ClonedCallable {
    fn call(&mut self) -> Result<i32, io::Error> {
        Ok(self.value)
    }
}

#[derive(Clone)]
struct SharedCallable {
    count: Rc<Cell<u32>>,
}

impl Callable<u32, io::Error> for SharedCallable {
    fn call(&mut self) -> Result<u32, io::Error> {
        self.count.set(self.count.get() + 1);
        Ok(self.count.get())
    }
}

#[derive(Clone)]
struct SharedCallableForArc {
    count: Arc<AtomicUsize>,
}

impl Callable<u32, io::Error> for SharedCallableForArc {
    fn call(&mut self) -> Result<u32, io::Error> {
        let value = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(value as u32)
    }
}

#[test]
fn test_callable_closure_call_returns_success_value() {
    let mut task = || Ok::<i32, io::Error>(42);

    assert_eq!(Callable::call(&mut task).expect("callable closure should succeed"), 42);
}

#[test]
fn test_callable_closure_call_returns_error() {
    let mut task = || Err::<i32, _>(io::Error::other("failed"));

    let error = Callable::call(&mut task).expect_err("callable closure should fail");
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "failed");
}

#[test]
fn test_box_callable_new_and_call() {
    let mut task = BoxCallable::new(|| Ok::<i32, io::Error>(21));

    assert_eq!(task.call().expect("box callable should succeed"), 21);
}

#[test]
fn test_box_callable_name_management() {
    let mut task = BoxCallable::<i32, io::Error>::new_with_name("compute", || Ok(1));

    assert_eq!(task.name(), Some("compute"));
    assert_eq!(task.to_string(), "BoxCallable(compute)");
    assert!(format!("{task:?}").contains("compute"));

    task.set_name("renamed");
    assert_eq!(task.name(), Some("renamed"));

    task.clear_name();
    assert_eq!(task.name(), None);
    assert_eq!(task.to_string(), "BoxCallable");
}

#[test]
fn test_box_callable_from_supplier() {
    let supplier = || Ok::<i32, io::Error>(34);

    let mut task = BoxCallable::from_supplier(supplier);

    assert_eq!(task.call().expect("supplier-backed callable should succeed"), 34,);
}

#[test]
fn test_box_callable_implements_supplier_once() {
    let task = BoxCallableOnce::new(|| Ok::<i32, io::Error>(55));

    let result = SupplierOnce::get(task);

    assert_eq!(result.expect("supplier callable should succeed"), 55);
}

#[test]
fn test_box_callable_map_transforms_success_value() {
    let task = BoxCallable::new_with_name("compute", || Ok::<i32, io::Error>(10));

    let mut mapped = task.map(|value| value * 2);

    assert_eq!(mapped.name(), Some("compute"));
    assert_eq!(mapped.call().expect("mapped callable should succeed"), 20);
}

#[test]
fn test_box_callable_map_err_transforms_error_value() {
    let task = BoxCallable::new(|| Err::<i32, _>(io::Error::other("raw")));

    let mut mapped = task.map_err(|error| error.to_string());

    assert_eq!(mapped.call().expect_err("mapped callable should fail"), "raw",);
}

#[test]
fn test_box_callable_and_then_runs_next_on_success() {
    let task = BoxCallable::new(|| Ok::<i32, io::Error>(4));

    let mut chained = task.and_then(|value| Ok(value * 3));

    assert_eq!(chained.call().expect("chained callable should succeed"), 12,);
}

#[test]
fn test_box_callable_and_then_skips_next_on_error() {
    let task = BoxCallable::new(|| Err::<i32, _>(io::Error::other("stop")));

    let mut chained = task.and_then(|_| Ok::<i32, io::Error>(99));

    assert_eq!(
        chained
            .call()
            .expect_err("chained callable should preserve error")
            .to_string(),
        "stop",
    );
}

#[test]
fn test_rc_callable_from_supplier() {
    let count = Rc::new(Cell::new(0));
    let captured = Rc::clone(&count);
    let mut task = RcCallable::from_supplier(move || {
        captured.set(captured.get() + 1);
        Ok::<i32, io::Error>(captured.get())
    });

    assert_eq!(task.call().expect("rc supplier should execute"), 1);
    assert_eq!(task.call().expect("rc supplier should execute again"), 2);
    assert_eq!(count.get(), 2);
}

#[test]
fn test_arc_callable_from_supplier() {
    let count = Arc::new(AtomicUsize::new(0));
    let captured = Arc::clone(&count);
    let mut task = ArcCallable::from_supplier(move || {
        captured.fetch_add(1, Ordering::SeqCst);
        Ok::<i32, io::Error>(captured.load(Ordering::SeqCst) as i32)
    });

    assert_eq!(task.call().expect("arc supplier should execute"), 1);
    assert_eq!(task.call().expect("arc supplier should execute again"), 2);
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[derive(Clone)]
struct TextCallable {
    value: String,
}

impl Callable<String, &'static str> for TextCallable {
    fn call(&mut self) -> Result<String, &'static str> {
        Ok(self.value.clone())
    }
}

#[test]
fn test_box_callable_combinators_with_text_error_type() {
    let mut mapped = BoxCallable::new(|| Ok::<i32, &'static str>(5)).map(|v| v + 7);
    assert_eq!(mapped.call().expect("map should succeed"), 12);

    let mut mapped_err = BoxCallable::new(|| Err::<i32, _>("raw")).map_err(|e| format!("E:{e}"));
    assert_eq!(mapped_err.call().expect_err("map_err should transform error"), "E:raw",);

    let mut chained = BoxCallable::new(|| Ok::<i32, &'static str>(3)).and_then(|v| Ok::<i32, &'static str>(v * 4));
    assert_eq!(chained.call().expect("and_then should succeed"), 12);
}
