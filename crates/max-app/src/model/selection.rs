#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    pub row: usize,
    pub column: usize,
}

impl Selection {
    pub const FIRST: Self = Self { row: 0, column: 0 };

    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}
