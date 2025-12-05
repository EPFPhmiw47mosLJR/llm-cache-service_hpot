use std::fs;
use std::path::Path;

use super::{PromptConfig, PromptProfile};

const SYSTEM_FILE: &str = "system.txt";
const CONFIG_FILE: &str = "config.toml";

pub fn load_prompt_profile(dir: &str) -> Result<Vec<PromptProfile>, Box<dyn std::error::Error>> {
    let mut profiles = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        match load_single_profile(&path) {
            Ok(profile) => profiles.push(profile),
            Err(e) => {
                return Err(format!("Failed loading profile '{}': {}", path.display(), e).into());
            }
        }
    }

    Ok(profiles)
}

fn load_single_profile(path: &Path) -> Result<PromptProfile, Box<dyn std::error::Error>> {
    let profile_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid profile directory name")?
        .to_ascii_lowercase();

    let system_prompt_path = path.join(SYSTEM_FILE);
    let config_path = path.join(CONFIG_FILE);

    let system_prompt = read_required_file(&system_prompt_path)?;
    let config: PromptConfig = {
        let raw = read_required_file(&config_path)?;
        toml::from_str(&raw).map_err(|e| {
            format!(
                "Failed to parse config file '{}': {}",
                config_path.display(),
                e
            )
        })?
    };

    Ok(PromptProfile {
        name: profile_name,
        system_prompt,
        config,
    })
}

fn read_required_file(path: &std::path::PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("Required file '{}' does not exist", path.display()).into());
    }

    fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read required file '{}': {}", path.display(), e).into())
}
