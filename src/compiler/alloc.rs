#[derive(Debug, Default)]
pub struct FrameAlloc {
    offset: usize,
}

impl FrameAlloc {
    pub fn alloc(&mut self, offset: usize) -> usize {
        let offset = self.offset;

        self.offset = offset;

        offset
    }

    pub fn dealloc(&mut self, offset: usize) {
        self.offset -= offset;
    }
}
