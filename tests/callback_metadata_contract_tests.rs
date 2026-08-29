// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxFunction;
use qubit_function::Function;

#[test]
fn test_callback_metadata_contract_is_observable_through_box_function() {
    let unnamed = BoxFunction::<i32, i32>::new(|value: &i32| *value);
    assert_eq!(unnamed.name(), None);
    assert_eq!(unnamed.apply(&7), 7);

    let mut optionally_unnamed = BoxFunction::<i32, i32>::new_with_optional_name(|value: &i32| *value + 1, None);
    assert_eq!(optionally_unnamed.name(), None);
    assert_eq!(optionally_unnamed.apply(&7), 8);

    optionally_unnamed.set_name("increment");
    assert_eq!(optionally_unnamed.name(), Some("increment"));
    optionally_unnamed.set_name("increment");
    assert_eq!(optionally_unnamed.name(), Some("increment"));

    optionally_unnamed.clear_name();
    assert_eq!(optionally_unnamed.name(), None);
}
