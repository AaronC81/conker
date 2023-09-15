use std::collections::HashMap;

use conker::{interpreter::Value, run_code};
use indoc::indoc;

mod utils;

#[test]
fn test_multi_task() {
    assert_eq!(
        run_code(indoc!{"
            task ConstantSource[5]
                5 -> Main

            task Main
                total = 0
                i = 0
                while i < 5
                    x <- ConstantSource[i]
                    total = total + x
                    i = i + 1
                total
        "}),
        Some(HashMap::from([
            ("ConstantSource".to_string(), Ok(Value::Null)),
            ("Main".to_string(), Ok(Value::Integer(25))),
        ]))
    );
}
