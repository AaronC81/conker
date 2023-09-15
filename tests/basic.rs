use std::collections::HashMap;

use conker::{interpreter::Value, run_code};
use indoc::indoc;

use crate::utils::{run_one_task, run_one_expression};

mod utils;

#[test]
fn test_arithmetic() {
    assert_eq!(
        run_one_expression("12 + 3"),
        Ok(Value::Integer(15))
    );
}

#[test]
fn test_comparisons() {
    // TODO: fix precedence!
    assert_eq!(
        run_one_expression("(2 + 2) == 4"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        run_one_expression("(2 + 2) == 5"),
        Ok(Value::Boolean(false))
    );

    assert_eq!(
        run_one_expression("4 > 3"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        run_one_expression("4 > 5"),
        Ok(Value::Boolean(false))
    );

    assert_eq!(
        run_one_expression("4 < 3"),
        Ok(Value::Boolean(false))
    );
    assert_eq!(
        run_one_expression("4 < 5"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn test_array() {
    assert_eq!(
        run_one_expression("[ ]"),
        Ok(Value::Array(vec![]))
    );

    assert_eq!(
        run_one_expression("[ 1, 2, 3 ]"),
        Ok(Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))
    );
    assert_eq!(
        run_one_expression("[ 1, 2, 3, ]"), // Trailing comma
        Ok(Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))
    );
}

#[test]
fn test_assign() {
    assert_eq!(
        run_one_task(indoc!{"
            task X
                x = 3
                x
        "}),
        Ok(Value::Integer(3))
    );
}

#[test]
fn test_precedence() {
    // Arithmetic
    assert_eq!(
        run_one_expression("2 + 3 * 5"),
        Ok(Value::Integer(2 + (3 * 5)))
    );
    assert_eq!(
        run_one_expression("3 * 5 + 2"),
        Ok(Value::Integer((3 * 5) + 2))
    );

    // Assignments and sends
    assert_eq!(
        run_code(indoc!{"
            task Bounce
                x <- ?c
                x -> c

            task X
                x = 2 + 3
                x + 1 -> Bounce
                y <- Bounce
                y
        "}),
        Some(HashMap::from([
            ("Bounce".to_string(), Ok(Value::Null)),
            ("X".to_string(), Ok(Value::Integer(6))),
        ]))
    );
}
