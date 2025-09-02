#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::I32(int) => f.write_fmt(format_args!("INT {}", int)),
        }
    }
}

#[macro_export]
macro_rules! i32 {
    ($x:expr) => {
        crate::value::Value::I32($x)
    };
}
