use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct TomlContent {
    pub rules: Vec<String>,
    pub watch: Vec<String>,
    pub protected: Vec<String>,
    pub targets: HashMap<String, String>,
}
impl TomlContent {
    pub fn merge(&mut self, other: TomlContent) {
        self.rules.extend(other.rules);
        self.watch.extend(other.watch);
        self.protected.extend(other.protected);
        self.targets.extend(other.targets);
    }
}
pub fn create_config(path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("config path {:?} has no parent directory", path))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create config directory for {:?}", path))?;

    let tomlcontent = TomlContent::default();

    let toml = toml::to_string(&tomlcontent)
        .context("failed to serialize default config to TOML")?;

    fs::write(path, toml)
        .with_context(|| format!("failed to write default config to {:?}", path))?;

    Ok(())
}

pub fn load_config(path: &PathBuf) -> anyhow::Result<TomlContent> {
    let toml_content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config from {:?}", path))?;

    let content = toml::from_str(&toml_content)
        .with_context(|| format!("failed to parse config at {:?}", path))?;

    Ok(content)
}

pub fn update_config(path: &PathBuf, content: &TomlContent) -> anyhow::Result<()> {
    let update = toml::to_string(&content)
        .context("failed to serialize config to TOML")?;
    fs::write(path, update)
        .with_context(|| format!("failed to write updated config to {:?}", path))?;

    Ok(())
}
