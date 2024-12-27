use egui_extras::image::load_image_bytes;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use wizardrs_core::card::value::CardValue;
use wizardrs_core::card::Card;

#[derive(Clone)]
pub struct ImageCache {
    cards: HashMap<Card, Option<String>>,
    average_aspect_ratio: Option<f32>,
}

impl ImageCache {
    pub fn new(path: &Path) -> Self {
        let mut cache = HashMap::new();
        let mut sum = 0.0;
        let mut num_cards = 0;

        for card in Card::all() {
            let file_name = match card.value {
                CardValue::Fool => format!("{}-fool.jpg", card.color),
                CardValue::Simple(value) => format!("{}-{value}.jpg", card.color),
                CardValue::Wizard => format!("{}-wizard.jpg", card.color),
            };
            let mut path = path.to_path_buf();
            path.push(file_name);

            if !path.exists() {
                // path doesnt exist
                cache.insert(card, None);
            } else {
                cache.insert(card, Some(path.to_string_lossy().to_string()));
            }

            if let Ok(bytes) = fs::read(path) {
                if let Ok(image) = load_image_bytes(&bytes) {
                    let width = image.width() as f32;
                    let height = image.height() as f32;
                    let ratio = height / width;

                    sum += ratio;
                    num_cards += 1;
                }
            }
        }

        let average_aspect_ratio = {
            if num_cards == 0 {
                None
            } else {
                Some(sum / num_cards as f32)
            }
        };

        Self {
            cards: cache,
            average_aspect_ratio,
        }
    }

    /// Returns the path of the image. E.g. file://root/something/image.png
    pub fn get_image_path(&self, card: &Card) -> Option<String> {
        if let Some(Some(path)) = self.cards.get(card) {
            return Some(format!("file://{path}"));
        }

        None
    }

    pub fn average_aspect_ratio(&self) -> Option<f32> {
        self.average_aspect_ratio
    }
}
