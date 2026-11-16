#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Head {
    Symbolic(String),
    Detached(String),
}