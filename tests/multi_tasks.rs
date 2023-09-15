use std::collections::HashMap;

use conker::{interpreter::Value, run_code};
use indoc::indoc;

mod utils;

#[test]
fn test_multi_task() {
    assert_eq!(
        run_code(indoc!{"
            task ConstantSource[5]
                $index -> Main

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
            ("ConstantSource[0]".to_string(), Ok(Value::Null)),
            ("ConstantSource[1]".to_string(), Ok(Value::Null)),
            ("ConstantSource[2]".to_string(), Ok(Value::Null)),
            ("ConstantSource[3]".to_string(), Ok(Value::Null)),
            ("ConstantSource[4]".to_string(), Ok(Value::Null)),
            ("Main".to_string(), Ok(Value::Integer(0 + 1 + 2 + 3 + 4))),
        ]))
    );
}
