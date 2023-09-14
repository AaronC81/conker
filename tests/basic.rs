use concurrent_lang::interpreter::Value;
use indoc::indoc;

use crate::utils::run_one_task;

mod utils;

#[test]
fn test_arithmetic() {
    assert_eq!(
        run_one_task(indoc! {"
            task X
                12 + 3
        "}),
        Ok(Value::Integer(15))
    )
}
