#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn lexeme<'s>(&self, source: &'s str) -> &'s str {
        &source[self.start..self.end]
    }
}
