use crate::StandardModule;

#[derive(Debug, Default)]
pub struct PathModule;

impl StandardModule for PathModule {
    fn name(&self) -> &'static str {
        "std.path"
    }
}
