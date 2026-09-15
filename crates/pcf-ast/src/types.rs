#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAnnotation {
    Named(String),
    Array(Box<TypeAnnotation>),
    Object,
    Unknown,
}
