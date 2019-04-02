#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SrcLoc {
    pub line: usize,
    pub col: usize,
}

impl SrcLoc {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SrcRange {
    pub loc: SrcLoc,
    pub len: usize,
}

impl SrcRange {
    pub fn new(loc: SrcLoc, len: usize) -> Self {
        Self { loc, len }
    }

    pub fn grow_by(mut self, n: usize) -> Self {
        self.len += n;
        self
    }
}
