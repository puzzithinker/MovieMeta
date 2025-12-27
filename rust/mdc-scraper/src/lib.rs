pub mod client;
pub mod cloudscraper;
pub mod http;
pub mod metadata;
pub mod registry;
pub mod scraper;
pub mod scrapers;

pub use client::ScraperClient;
pub use cloudscraper::CloudScraperClient;
pub use http::{HttpBackend, HttpClient, HttpClientBuilder, HttpConfig};
pub use metadata::MovieMetadata;
pub use registry::ScraperRegistry;
pub use scraper::{Scraper, ScraperConfig};
