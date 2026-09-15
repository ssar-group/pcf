use crate::StandardModule;

#[derive(Debug, Default)]
pub struct FsModule;

impl StandardModule for FsModule {
    fn name(&self) -> &'static str {
        "std.fs"
    }
}
