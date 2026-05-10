use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Deserialize, Serialize, Default)]
pub struct TomlContent {
    pub rules: Vec<String>,
    pub watch: Vec<String>,
    pub protected: Vec<String>,
}

pub fn create_config(path: &PathBuf) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(path.parent().unwrap())?;

    let tomlcontent = TomlContent::default();

    let toml = toml::to_string(&tomlcontent)?;

    fs::write(path, toml)?;

    Ok(())
}

pub fn load_config(path: PathBuf) -> Result<TomlContent> {
    let toml_content = fs::read_to_string(path)?;

    let content = toml::from_str(&toml_content)?;

    Ok(content)
}
