use pcf_ecosystem::Ecosystem;
use pcf_runtime::RuntimeRegistry;

#[derive(Debug, Default)]
pub struct OpenScriptEcosystem;

impl Ecosystem for OpenScriptEcosystem {
    fn id(&self) -> &str {
        "openscript"
    }

    fn name(&self) -> &str {
        "OpenScript"
    }

    fn register(
        &self,
        _registry: &mut RuntimeRegistry,
    ) -> Result<(), pcf_ecosystem::EcosystemError> {
        Ok(())
    }
}
