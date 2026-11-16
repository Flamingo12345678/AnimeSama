pub mod skips;
pub mod trailer;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardItem {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub nom: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub img: String,
    #[serde(default)]
    pub thumb: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub badges: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub etat: String,
    #[serde(default)]
    pub info: String,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub synopsis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroSlide {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub badge: String,
    #[serde(default)]
    pub bg: String,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub synopsis: String,
    #[serde(default)]
    pub cta_link: String,
    #[serde(default)]
    pub cta_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub cards: Vec<CardItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogData {
    pub slides: Vec<HeroSlide>,
    pub planning: HashMap<String, Vec<CardItem>>,
    pub sections: Vec<Section>,
    pub catalogue: Vec<CardItem>,
}

impl CatalogData {
    pub fn load() -> Result<Self, String> {
        let data = crate::scraper::Scraper::new().get_home()?;
        println!(
            "[Catalogue] Chargé en direct depuis anime-sama.to ({} animes, {} sections, {} slides)",
            data.catalogue.len(),
            data.sections.len(),
            data.slides.len()
        );
        let mut data = data;

        if let Some(path) = std::env::var_os(
            "CATACLYSE_CATALOGUE_SNAPSHOT"
        ) {
            let metadata = std::fs::metadata(&path)
                .map_err(|e| format!("Snapshot inaccessible : {e}"))?;

            if !metadata.is_file() || metadata.len() > 8_000_000 {
                return Err("Snapshot catalogue invalide".into());
            }

            let raw = std::fs::read(&path)
                .map_err(|e| format!("Lecture snapshot : {e}"))?;

            let cards: Vec<CardItem> = serde_json::from_slice(&raw)
                .map_err(|e| format!("JSON catalogue : {e}"))?;

            if !(2000..=20000).contains(&cards.len()) {
                return Err("Taille du catalogue inattendue".into());
            }

            let mut seen = std::collections::HashSet::new();

            for card in &cards {
                if card.slug.is_empty()
                    || card.slug.len() > 256
                    || card.slug == "."
                    || card.slug.contains("..")
                    || !card.slug.chars().all(|c| {
                        c.is_alphanumeric()
                            || matches!(c, '-' | '_' | '.')
                    })
                    || card.nom.trim().is_empty()
                    || !seen.insert(card.slug.as_str())
                {
                    return Err(
                        "Slug dupliqué ou fiche invalide".into()
                    );
                }
            }

            println!(
                "[Catalogue] Snapshot validé : {} œuvres",
                cards.len()
            );

            data.catalogue = cards;
        }

        Ok(data)
    }

    pub fn empty() -> Self {
        CatalogData {
            slides: vec![],
            planning: HashMap::new(),
            sections: vec![],
            catalogue: vec![],
        }
    }

    pub fn get_by_slug(&self, slug: &str) -> Option<CardItem> {
        let clean = slug.trim().trim_matches('/');
        self.catalogue.iter().find(|i| i.slug == clean).cloned()
    }
}
