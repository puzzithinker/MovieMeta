//! Available scraper implementations

pub mod avmoo;
pub mod fc2;
pub mod imdb;
pub mod javbus;
pub mod javlibrary;
pub mod tmdb;
pub mod tokyohot;

pub use avmoo::AvmooScraper;
pub use fc2::Fc2Scraper;
pub use imdb::ImdbScraper;
pub use javbus::JavbusScraper;
pub use javlibrary::JavlibraryScraper;
pub use tmdb::TmdbScraper;
pub use tokyohot::TokyohotScraper;
