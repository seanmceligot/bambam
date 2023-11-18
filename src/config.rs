use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct BamBamConfig {
    pub access_key: String,
    pub ppn_file: String,
    pub rhn_file: String,
    pub lock_door: String,
    pub kitchen_light_yellow: String,
    pub kitchen_light_purple: String,
}
pub fn read_bambam_config(filename: &PathBuf) -> Result<BamBamConfig> {
    let mut file = File::open(filename)
        .with_context(|| format!("read_bambam_config: could not open {:?}", filename))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: BamBamConfig = serde_json::from_str(&contents)
        .with_context(|| format!("read_bambam_config: could not parse {:?}", filename))?;
    Ok(config)
}
