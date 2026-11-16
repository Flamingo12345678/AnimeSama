use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeSeason {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeDetails {
    pub slug: String,
    pub title: String,
    pub banner: String,
    pub synopsis: String,
    pub genres: Vec<String>,
    pub seasons: Vec<AnimeSeason>,
}


// Ne pas prendre pour des saisons les exemples de code
// situés dans les commentaires JavaScript /* ... */.
fn active_panel_captures<'a>(
    html: &'a str,
    pattern: &Regex,
) -> Vec<regex::Captures<'a>> {
    let comments: Vec<(usize, usize)> =
        Regex::new(r"(?s)/\*.*?\*/")
            .unwrap()
            .find_iter(html)
            .map(|m| (m.start(), m.end()))
            .collect();

    pattern
        .captures_iter(html)
        .filter(|cap| {
            let start = cap.get(0).unwrap().start();

            !comments.iter().any(|&(begin, end)| {
                start >= begin && start < end
            })
        })
        .collect()
}

pub fn fetch_anime_details(slug: &str) -> Result<AnimeDetails, String> {
    let clean_slug = slug.trim().trim_matches('/');

    if !super::http::valid_slug(clean_slug) {
        return Err("Slug du catalogue invalide".to_string());
    }

    let url = format!("https://anime-sama.to/catalogue/{}/", clean_slug);
    let html = super::http::fetch_site_text(
    &url,
    8,
    4 * 1024 * 1024,
    None,
)?;

    let title_re = Regex::new(r#"<h1[^>]*id=["']titreOeuvre["'][^>]*>(.*?)</h1>"#).unwrap();
    let fallback_title_re = Regex::new(r#"<h1[^>]*>(.*?)</h1>"#).unwrap();
    let title = title_re
        .captures(&html)
        .or_else(|| fallback_title_re.captures(&html))
        .map(|c| c[1].trim().to_string())
        .unwrap_or_else(|| clean_slug.replace('-', " "));

    let syn_re = Regex::new(r#"<p[^>]*id=["']synopsisText["'][^>]*>(.*?)</p>"#).unwrap();
    let syn_re2 = Regex::new(r#"<p[^>]*id=["']synopsis["'][^>]*>(.*?)</p>"#).unwrap();
    let syn_fallback = Regex::new(r#"class=["'][^"']*synopsis[^"']*["'][^>]*>(.*?)</p>"#).unwrap();
    let synopsis = syn_re
        .captures(&html)
        .or_else(|| syn_re2.captures(&html))
        .or_else(|| syn_fallback.captures(&html))
        .map(|c| {
            let tag_strip = Regex::new(r#"<[^>]+>"#).unwrap();
            tag_strip.replace_all(&c[1], "").trim().to_string()
        })
        .unwrap_or_default();

    let genre_re = Regex::new(r#"<a[^>]*href=["']/catalogue/\?[^"']*genre[^"']*["'][^>]*>(.*?)</a>"#).unwrap();
    let mut genres: Vec<String> = genre_re
        .captures_iter(&html)
        .map(|c| c[1].trim().to_string())
        .filter(|g| !g.is_empty())
        .collect();
    if genres.is_empty() {
        let alt_genre_re = Regex::new(r#"class=["'][^"']*genre[^"']*["'][^>]*>(.*?)<"#).unwrap();
        genres = alt_genre_re
            .captures_iter(&html)
            .map(|c| c[1].trim().to_string())
            .filter(|g| !g.is_empty() && g.len() < 30)
            .collect();
    }

    let season_re = Regex::new(r#"panneauAnime\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#).unwrap();
    let scan_re = Regex::new(r#"panneauScan\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#).unwrap();

    let mut seasons: Vec<AnimeSeason> = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for cap in active_panel_captures(&html, &season_re) {
        let name = cap[1].trim().to_string();
        let path = cap[2].trim().trim_matches('/').to_string();

        if name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") || name.starts_with("/*") {
            continue;
        }

        if seen_paths.insert(path.clone()) {
            seasons.push(AnimeSeason { name, path });
        }
    }

    for cap in active_panel_captures(&html, &scan_re) {
        let name = cap[1].trim().to_string();
        let path = cap[2].trim().trim_matches('/').to_string();

        if name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") || name.starts_with("/*") {
            continue;
        }

        if seen_paths.insert(path.clone()) {
            seasons.push(AnimeSeason {
                name: format!("Scan: {}", name),
                path,
            });
        }
    }

    if seasons.is_empty() {
        seasons.push(AnimeSeason {
            name: "Saison 1".to_string(),
            path: "saison1/vostfr".to_string(),
        });
    }

    let banner = format!("https://cdn.jsdelivr.net/gh/Anime-Sama/IMG@img/contenu/{}.jpg", clean_slug);

    let details = AnimeDetails {
        slug: clean_slug.to_string(),
        title,
        banner,
        synopsis,
        genres,
        seasons,
    };

    Ok(details)
}


#[cfg(test)]
mod panel_comment_filter_tests {
    use super::*;

    fn parse(html: &str, kind: &str) -> Vec<(String, String)> {
        let pattern = Regex::new(&format!(
            r#"panneau{}\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#,
            kind
        )).unwrap();

        active_panel_captures(html, &pattern)
            .iter()
            .map(|cap| (
                cap[1].to_string(),
                cap[2].to_string()
            ))
            .collect()
    }

    #[test]
    fn ignores_commented_seasons() {
        let html = r#"
            panneauAnime("Saison 1", "saison1/vostfr");
            /*
            panneauAnime("Saison 2", "saison2/vostfr");
            panneauAnime("Film", "film/vostfr");
            */
        "#;

        assert_eq!(
            parse(html, "Anime"),
            vec![(
                "Saison 1".into(),
                "saison1/vostfr".into()
            )]
        );
    }

    #[test]
    fn keeps_real_multiple_seasons() {
        let html = r#"
            panneauAnime("Saison 1", "saison1/vostfr");
            panneauAnime("Saison 2", "saison2/vostfr");
        "#;

        assert_eq!(parse(html, "Anime").len(), 2);
    }

    #[test]
    fn keeps_scans_outside_comments() {
        let html = r#"
            /* panneauScan("nom", "url"); */
            panneauScan("Scans", "scan/vf");
        "#;

        assert_eq!(
            parse(html, "Scan"),
            vec![("Scans".into(), "scan/vf".into())]
        );
    }
}
