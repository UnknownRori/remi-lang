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

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("line: {} on column {}", self.row, self.column))
    }
}
