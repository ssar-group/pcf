use crate::StandardModule;

#[derive(Debug, Default)]
pub struct CollectionsModule;

impl StandardModule for CollectionsModule {
    fn name(&self) -> &'static str {
        "std.collections"
    }
}
