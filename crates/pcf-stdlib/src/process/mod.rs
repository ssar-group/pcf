use crate::StandardModule;

#[derive(Debug, Default)]
pub struct ProcessModule;

impl StandardModule for ProcessModule {
    fn name(&self) -> &'static str {
        "std.process"
    }
}
