use conker::interpreter::Value;
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
