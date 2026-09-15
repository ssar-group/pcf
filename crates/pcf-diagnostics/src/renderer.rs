use crate::Diagnostic;

#[derive(Debug, Default)]
pub struct Renderer;

impl Renderer {
    pub fn render(&self, diagnostic: &Diagnostic) -> String {
        format!("{}: {}", diagnostic.code.0, diagnostic.message)
    }
}
