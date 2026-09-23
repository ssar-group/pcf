use std::collections::HashSet;

use pcf_ast::{Item, Program};

use crate::{ModulePath, ResolveError, ResolvedModule};

#[derive(Debug, Clone, Default)]
pub struct ResolveResult {
    pub module: Option<ResolvedModule>,
}

#[derive(Debug, Default)]
pub struct Resolver;

impl Resolver {
    pub fn resolve_program(&self, program: &Program) -> Result<ResolveResult, ResolveError> {
        let mut segments = HashSet::new();
        let mut modules = Vec::new();

        for item in &program.items {
            match item {
                Item::Module(module) => {
                    modules.push(module.name.name.clone());
                    segments.insert(module.name.name.clone());
                }
                Item::Import(import) => {
                    if import.path.is_empty() {
                        return Err(ResolveError {
                            message: "import declarations require a path".to_string(),
                        });
                    }
                    for identifier in &import.path {
                        segments.insert(identifier.name.clone());
                    }
                    modules.extend(import.path.iter().map(|identifier| identifier.name.clone()));
                }
                Item::Function(function) => {
                    segments.insert(function.name.name.clone());
                }
                Item::Variable(variable) => {
                    segments.insert(variable.statement.name.name.clone());
                }
                Item::Schema(schema) => {
                    segments.insert(schema.name.name.clone());
                }
                Item::Statement(_) => {}
            }
        }

        let mut ordered = modules;
        ordered.sort();
        ordered.dedup();

        Ok(ResolveResult {
            module: Some(ResolvedModule {
                path: ModulePath { segments: ordered },
            }),
        })
    }
}
