#[derive(Debug, Clone, Copy)]
pub(crate) struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(crate) fn start(&self) -> usize {
        self.start
    }

    pub(crate) fn end(&self) -> usize {
        self.end
    }

    pub(crate) fn merge(&self, other: &Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }
}

pub(crate) struct SourceMap {
    line_starts: Vec<usize>,
}

impl SourceMap {
    pub(crate) fn new() -> Self {
        Self {
            line_starts: vec![0],
        }
    }

    pub(crate) fn newline(&mut self, offset: usize) {
        let (Ok(i) | Err(i)) = self.line_starts.binary_search(&offset);
        self.line_starts.insert(i, offset);
    }

    pub(crate) fn map_offset(&self, offset: usize) -> (usize, usize) {
        let mut line = self.line_starts.len() - 1;

        for (idx, &start) in self.line_starts.iter().enumerate().rev() {
            if offset >= start {
                line = idx;
                break;
            }
        }

        let col = offset - self.line_starts[line];

        (line, col)
    }
}
