use crate::value::DataType;

type IndexAlloc = usize;

#[derive(Debug, Default)]
pub struct ScopeAlloc {
    index: usize,
    size_offset: usize,
}

impl ScopeAlloc {
    pub fn alloc(&mut self, data_type: DataType) -> IndexAlloc {
        // TODO : Make this properly alloc
        let index = self.index;

        self.size_offset = 8;
        self.index += 1;

        index
    }
}
