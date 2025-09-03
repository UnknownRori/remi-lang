pub enum FunctionStorage {
    External,
    Internal,
}

pub struct FunctionSymbol {
    pub args: Vec<String>,
    pub return_type: String,
    pub storage: FunctionStorage,
}
