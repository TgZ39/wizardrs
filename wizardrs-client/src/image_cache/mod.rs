use crate::error::*;
use egui_extras::image::load_image_bytes;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use wizardrs_core::card::color::CardColor;
use wizardrs_core::card::value::CardValue;
use wizardrs_core::card::Card;

#[derive(Clone)]
pub struct ImageCache {
    cards: HashMap<Card, PathBuf>,
    average_aspect_ratio: Option<f32>,
}

impl ImageCache {
    pub fn new(path: &Path) -> Result<Self> {
        let mut cache = HashMap::new();
        let mut sum = 0.0;
        let mut num_cards = 0;

        for entry in path.read_dir()?.flatten() {
            if !entry.path().is_file() {
                // entry is not a file
                continue;
            }

            // try loading image
            if let Ok(bytes) = fs::read(entry.path()) {
                if let Ok(image) = load_image_bytes(&bytes) {
                    let width = image.width() as f32;
                    let height = image.height() as f32;
                    let ratio = height / width;

                    sum += ratio;
                    num_cards += 1;
                } else {
                    // file is not an image
                    continue;
                }
            } else {
                // couldn't read file
                continue;
            }

            // parse file name into card
            let path = entry.path();
            let file_stem = if let Some(stem) = path.file_stem() {
                stem.to_string_lossy()
            } else {
                // no file stem?
                continue;
            };
            // parse file stem to color and value
            let (color, value) = if let Some((color, value)) = file_stem.split_once('-') {
                (color, value)
            } else {
                // file name could not be parsed
                continue;
            };

            let color = match color.to_ascii_lowercase().as_str().trim() {
                "blue" => CardColor::Blue,
                "green" => CardColor::Green,
                "red" => CardColor::Red,
                "yellow" => CardColor::Yellow,
                _ => continue,
            };
            let value = match value.to_ascii_lowercase().trim() {
                "fool" => CardValue::Fool,
                "wizard" => CardValue::Wizard,
                other => {
                    if let Ok(value) = other.parse::<u8>() {
                        if let Ok(value) = CardValue::new(value) {
                            value
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            };
            let card = Card { color, value };

            // write to cache
            cache.insert(card, path.to_path_buf());
        }

        let average_aspect_ratio = {
            if num_cards == 0 {
                None
            } else {
                Some(sum / num_cards as f32)
            }
        };

        Ok(Self {
            cards: cache,
            average_aspect_ratio,
        })
    }

    /// Returns the path of the image. E.g. file://root/something/image.png
    pub fn get_image_path(&self, card: &Card) -> Option<String> {
        self.cards
            .get(card)
            .map(|path| format!("file://{}", path.to_string_lossy()))
    }

    pub fn average_aspect_ratio(&self) -> Option<f32> {
        self.average_aspect_ratio
    }
}
