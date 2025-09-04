pub fn align_mem(size: usize) -> usize {
    (size + 15) & !15
}
