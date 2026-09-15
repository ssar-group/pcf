use crate::StandardModule;

#[derive(Debug, Default)]
pub struct StringsModule;

impl StandardModule for StringsModule {
    fn name(&self) -> &'static str {
        "std.strings"
    }
}
