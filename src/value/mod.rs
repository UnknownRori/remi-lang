#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
}

pub enum DataType {
    I32,
}

#[macro_export]
macro_rules! i32 {
    ($x:expr) => {
        crate::value::Value::I32($x)
    };
}
