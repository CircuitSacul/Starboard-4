pub fn div_ceil(a: usize, b: usize) -> usize {
    (0..a).step_by(b).size_hint().0
}
