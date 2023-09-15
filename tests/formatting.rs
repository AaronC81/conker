use std::collections::HashMap;

use conker::{interpreter::Value, run_code};
use indoc::indoc;

use crate::utils::run_one_task;

mod utils;

#[test]
fn test_blank_line() {
    assert_eq!(
        run_one_task(indoc!{"
            task X
                1

                2
        "}),
        Ok(Value::Integer(2))
    );

    assert_eq!(
        run_one_task(indoc!{"
            task X
                1



                
                2
        "}),
        Ok(Value::Integer(2))
    );
}
