use crate::SourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub id: SourceId,
    pub name: String,
    pub content: String,
}
