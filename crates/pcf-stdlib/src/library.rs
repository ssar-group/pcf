use crate::StandardModule;

#[derive(Debug, Default)]
pub struct Library {
    pub modules: Vec<&'static str>,
}

impl Library {
    pub fn register<M: StandardModule>(&mut self, module: &M) {
        self.modules.push(module.name());
    }
}
