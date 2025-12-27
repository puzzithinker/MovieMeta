//! IMDB (Internet Movie Database) scraper

use anyhow::Result;
use async_trait::async_trait;
use scraper::Html;

use crate::metadata::MovieMetadata;
use crate::scraper::Scraper;

/// IMDB scraper implementation
pub struct ImdbScraper;

impl ImdbScraper {
    /// Create a new IMDB scraper
    pub fn new() -> Self {
        Self
    }
}

impl Default for ImdbScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for ImdbScraper {
    fn source(&self) -> &str {
        "imdb"
    }

    fn imagecut(&self) -> i32 {
        0 // Don't crop IMDB covers
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // number is expected to be IMDB ID (e.g., "tt1234567")
        // TODO: Implement search by name
        Ok(format!("https://www.imdb.com/title/{}", number))
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Extract movie ID from URL
        if let Some(id_part) = url.split('/').find(|s| s.starts_with("tt")) {
            metadata.number = id_part.to_string();
        }

        // Title: //h1[@data-testid="hero-title-block__title"]/text()
        metadata.title = self.select_text(html, r#"h1[data-testid="hero-title-block__title"]"#);

        // If the data-testid selector fails, try alternative selectors
        if metadata.title.is_empty() {
            metadata.title = self.select_text_by_exprs(
                html,
                &[
                    r#"h1[data-testid="hero__pageTitle"]"#,
                    "h1.sc-afe43def-0",
                    "h1",
                ],
            );
        }

        // Release date: //a[contains(text(),"Release date")]/following-sibling::div[1]/ul/li/a/text()
        // CSS selector approximation: look for release date link and extract text
        metadata.release = self.select_text_by_exprs(
            html,
            &[
                r#"a[href*="releaseinfo"] + div ul li a"#,
                r#"li[data-testid="title-details-releasedate"] a"#,
                r#"a[href*="releaseinfo"]"#,
            ],
        );

        // Cover: //head/meta[@property="og:image"]/@content
        metadata.cover = self.select_attr(html, r#"meta[property="og:image"]"#, "content");

        // Outline: //head/meta[@property="og:description"]/@content
        metadata.outline = self.select_attr(html, r#"meta[property="og:description"]"#, "content");

        // Actors: //h3[contains(text(),"Top cast")]/../../../following-sibling::div[1]/div[2]/div/div/a/text()
        // CSS approximation: look for cast section
        metadata.actor = self.select_all_text_by_exprs(
            html,
            &[
                r#"section[data-testid="title-cast"] a[data-testid="title-cast-item__actor"]"#,
                r#"div[data-testid="title-cast-item"] a"#,
                r#"table.cast_list td:nth-child(2) a"#,
            ],
        );

        // Tags/Genres: //div[@data-testid="genres"]/div[2]/a/ul/li/text()
        metadata.tag = self.select_all_text_by_exprs(
            html,
            &[
                r#"div[data-testid="genres"] a span"#,
                r#"div[data-testid="genres"] span"#,
                r#"a[href*="/search/title/?genres="] span"#,
            ],
        );

        // Runtime
        metadata.runtime = self.select_text_by_exprs(
            html,
            &[
                r#"li[data-testid="title-techspec_runtime"] div"#,
                r#"time[datetime]"#,
            ],
        );

        // Rating
        let rating_str = self.select_text_by_exprs(
            html,
            &[
                r#"div[data-testid="hero-rating-bar__aggregate-rating__score"] span"#,
                r#"span[class*="AggregateRating"] span"#,
            ],
        );
        if !rating_str.is_empty() {
            // Extract number from string like "7.5/10"
            if let Some(num_part) = rating_str.split('/').next() {
                if let Ok(rating) = num_part.trim().parse::<f32>() {
                    metadata.userrating = rating;
                }
            }
        }

        // Vote count
        let votes_str = self.select_text_by_exprs(
            html,
            &[
                r#"div[data-testid="hero-rating-bar__aggregate-rating__score"] ~ div"#,
                r#"div[class*="TotalRating"]"#,
            ],
        );
        if !votes_str.is_empty() {
            // Extract number from string like "1.2M" or "234K"
            let clean_votes = votes_str
                .replace(",", "")
                .replace("K", "000")
                .replace("M", "000000");
            if let Some(num_part) = clean_votes.split_whitespace().next() {
                if let Ok(votes) = num_part.parse::<u32>() {
                    metadata.uservotes = votes;
                }
            }
        }

        // Director
        metadata.director = self.select_text_by_exprs(
            html,
            &[
                r#"li[data-testid="title-pc-principal-credit"] a"#,
                r#"a[href*="/name/"]"#,
            ],
        );

        Ok(metadata)
    }
}

/// Helper to try multiple selector strategies
impl ImdbScraper {
    fn select_all_text_by_exprs(&self, html: &Html, selectors: &[&str]) -> Vec<String> {
        for selector_str in selectors {
            let results = self.select_all_text(html, selector_str);
            if !results.is_empty() {
                return results;
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_imdb_scraper_url() {
        let scraper = ImdbScraper::new();
        let url = scraper.query_number_url("tt1234567").await.unwrap();
        assert_eq!(url, "https://www.imdb.com/title/tt1234567");
    }

    #[test]
    fn test_imdb_scraper_parse() {
        let scraper = ImdbScraper::new();

        let html_content = r#"
        <html>
        <head>
            <meta property="og:title" content="Test Movie" />
            <meta property="og:description" content="A thrilling test movie" />
            <meta property="og:image" content="https://m.media-amazon.com/images/M/poster.jpg" />
        </head>
        <body>
            <h1 data-testid="hero-title-block__title">Test Movie</h1>
            <div data-testid="genres">
                <a><span>Action</span></a>
                <a><span>Thriller</span></a>
            </div>
            <div data-testid="hero-rating-bar__aggregate-rating__score">
                <span>8.5/10</span>
            </div>
        </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper
            .parse_metadata(&html, "https://www.imdb.com/title/tt1234567")
            .unwrap();

        assert_eq!(metadata.number, "tt1234567");
        assert_eq!(metadata.title, "Test Movie");
        assert_eq!(metadata.outline, "A thrilling test movie");
        assert!(metadata.cover.contains("poster.jpg"));
        assert_eq!(metadata.tag, vec!["Action", "Thriller"]);
        assert_eq!(metadata.userrating, 8.5);
    }
}
