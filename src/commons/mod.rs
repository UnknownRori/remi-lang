#[derive(Debug, Default, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Loc,
    pub end: Loc,
}

impl Span {
    pub fn new(start: Loc, end: Loc) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}
