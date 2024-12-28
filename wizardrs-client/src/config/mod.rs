use crate::error::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,
    pub theme: egui::ThemePreference,
    pub card_deck: Option<PathBuf>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(content) => {
                // load config
                let mut config = serde_json::from_str::<Self>(&content)?;
                // set path
                config.path = path.to_path_buf();
                Ok(config)
            }
            Err(_) => {
                // create new config
                let config = Self {
                    path: path.to_path_buf(),
                    theme: egui::ThemePreference::System,
                    card_deck: None,
                };
                // try saving config
                config.save()?;

                Ok(config)
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let parent = if let Some(parent) = self.path.parent() {
            parent
        } else {
            return Err(Error::Other("can't find parent path".to_string()));
        };

        // check if the path exists
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&self.path)?;
        let json = serde_json::to_string_pretty(self)?;

        file.write_all(json.as_bytes())?;

        Ok(())
    }
}
