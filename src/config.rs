use ron::de;
use serde::Deserialize;
use anyhow::{Result, Context};

use std::path::Path;
use std::fs::read_to_string;
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum Requirement {
    RqFocus(String),
    RqExtEvent(String),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Agg {
    pub requirements: HashSet<Requirement>,
    pub on_ipc: String,
    pub off_ipc: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AltCfg {
    pub aggs: Vec<Agg>,
}

impl AltCfg {
    pub fn parse(path: &Path) -> Result<Self> {
        let cfg_str = read_to_string(path)
            .with_context(|| format!("Failed to read the config file at {:?}", path))?;

        let cfg: AltCfg = de::from_str(&cfg_str)?;
        Ok(cfg)
    }
}
