use std::{fs, path::PathBuf};

pub fn fixture_name(name: &str) -> String {
    name.to_string()
}

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|root| root.parent())
        .map(|root| root.join("tests").join("fixtures").join(name))
        .unwrap_or_else(|| PathBuf::from(name))
}

pub fn read_fixture(name: &str) -> std::io::Result<String> {
    fs::read_to_string(fixture_path(name))
}

pub fn parse_fixture(name: &str) -> Result<pcf::ast::Program, Box<dyn std::error::Error>> {
    let source = read_fixture(name)?;
    pcf::parse(&source).map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
}

pub fn assert_valid_fixture(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = parse_fixture(name)?;
    Ok(())
}
