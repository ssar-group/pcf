use crate::{CapabilitySet, Environment, PermissionSet, RuntimeRegistry};

#[derive(Debug)]
pub struct RuntimeContext<'a> {
    pub environment: &'a mut Environment,
    pub registry: &'a RuntimeRegistry,
    pub permissions: &'a PermissionSet,
    pub capabilities: &'a CapabilitySet,
}
