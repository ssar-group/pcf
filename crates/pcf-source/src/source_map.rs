use crate::{SourceFile, SourceId};
use pcf_span::Position;

#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    sources: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, name: impl Into<String>, content: impl Into<String>) -> SourceId {
        let id = SourceId(self.sources.len());
        self.sources.push(SourceFile {
            id,
            name: name.into(),
            content: content.into(),
        });
        id
    }

    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.sources.get(id.0)
    }

    pub fn position(&self, _id: SourceId, offset: usize) -> Position {
        Position {
            line: 1,
            column: offset + 1,
        }
    }
}
