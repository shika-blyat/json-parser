use std::ops::Range;

#[derive(Debug, Clone)]
pub struct ParserError {
    pos_range: Option<Range<usize>>,
    reason: Option<String>,
}
impl ParserError {
    pub fn new_empty() -> Self {
        Self {
            pos_range: None,
            reason: None,
        }
    }
    pub fn new(pos_range: Range<usize>, reason: String) -> Self {
        Self {
            pos_range: Some(pos_range),
            reason: Some(reason),
        }
    }
    pub fn set_reason(&mut self, new_reason: String) {
        self.reason = Some(new_reason);
    }
}
