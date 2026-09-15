use pcf_ast::Program;

#[derive(Debug, Default)]
pub struct Formatter;

impl Formatter {
    pub fn format_program(&self, _program: &Program) -> String {
        String::new()
    }
}
