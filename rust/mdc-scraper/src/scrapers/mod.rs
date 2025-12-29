//! Available scraper implementations

pub mod avmoo;
pub mod dmm;
pub mod fc2;
pub mod imdb;
pub mod javbus;
pub mod javlibrary;
pub mod r18dev;
pub mod tmdb;
pub mod tokyohot;

pub use avmoo::AvmooScraper;
pub use dmm::DmmScraper;
pub use fc2::Fc2Scraper;
pub use imdb::ImdbScraper;
pub use javbus::JavbusScraper;
pub use javlibrary::JavlibraryScraper;
pub use r18dev::R18DevScraper;
pub use tmdb::TmdbScraper;
pub use tokyohot::TokyohotScraper;
