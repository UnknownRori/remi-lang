#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    I32(i32),
    String(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::I32(int) => f.write_fmt(format_args!("INT {}", int)),
            Value::String(str) => f.write_fmt(format_args!("STRING {}", str)),
        }
    }
}

#[macro_export]
macro_rules! i32 {
    ($x:expr) => {
        crate::value::Value::I32($x)
    };
}

#[macro_export]
macro_rules! string {
    ($x:expr) => {
        crate::value::Value::String($x)
    };
}
