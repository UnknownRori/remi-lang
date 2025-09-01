#[derive(Debug, Default, Clone, Copy)]
pub struct Loc {
    pub column: usize,
    pub row: usize,
}

impl Loc {
    pub fn new(column: usize, row: usize) -> Self {
        Self { column, row }
    }
}
