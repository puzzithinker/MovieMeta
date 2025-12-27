//! NFO file generation for Kodi/Jellyfin

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Generate NFO XML content from metadata
pub fn generate_nfo(metadata: &serde_json::Value, movie_id: &str) -> Result<String> {
    let mut nfo = String::new();

    nfo.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
    nfo.push_str("<movie>\n");

    // Title
    if let Some(title) = metadata.get("title").and_then(|v| v.as_str()) {
        nfo.push_str(&format!("  <title>{}</title>\n", xml_escape(title)));
    }

    // Original title (same as title for now)
    if let Some(title) = metadata.get("title").and_then(|v| v.as_str()) {
        nfo.push_str(&format!("  <originaltitle>{}</originaltitle>\n", xml_escape(title)));
    }

    // Sort title (number for proper sorting)
    nfo.push_str(&format!("  <sorttitle>{}</sorttitle>\n", xml_escape(movie_id)));

    // User rating
    if let Some(rating) = metadata.get("userrating").and_then(|v| v.as_f64()) {
        nfo.push_str(&format!("  <rating>{:.1}</rating>\n", rating));
        nfo.push_str(&format!("  <criticrating>{:.1}</criticrating>\n", rating));
    }

    // Votes
    if let Some(votes) = metadata.get("uservotes").and_then(|v| v.as_u64()) {
        nfo.push_str(&format!("  <votes>{}</votes>\n", votes));
    }

    // Outline/Plot
    if let Some(outline) = metadata.get("outline").and_then(|v| v.as_str()) {
        nfo.push_str(&format!("  <outline>{}</outline>\n", xml_escape(outline)));
        nfo.push_str(&format!("  <plot>{}</plot>\n", xml_escape(outline)));
    }

    // Runtime
    if let Some(runtime) = metadata.get("runtime").and_then(|v| v.as_str()) {
        if let Ok(mins) = runtime.parse::<u32>() {
            nfo.push_str(&format!("  <runtime>{}</runtime>\n", mins));
        }
    }

    // Release date
    if let Some(release) = metadata.get("release").and_then(|v| v.as_str()) {
        nfo.push_str(&format!("  <releasedate>{}</releasedate>\n", xml_escape(release)));
        nfo.push_str(&format!("  <premiered>{}</premiered>\n", xml_escape(release)));
    }

    // Year
    if let Some(year) = metadata.get("year").and_then(|v| v.as_str()) {
        nfo.push_str(&format!("  <year>{}</year>\n", xml_escape(year)));
    }

    // Director
    if let Some(director) = metadata.get("director").and_then(|v| v.as_str()) {
        if !director.is_empty() {
            nfo.push_str(&format!("  <director>{}</director>\n", xml_escape(director)));
        }
    }

    // Studio/Maker
    if let Some(studio) = metadata.get("studio").and_then(|v| v.as_str()) {
        if !studio.is_empty() {
            nfo.push_str(&format!("  <studio>{}</studio>\n", xml_escape(studio)));
            nfo.push_str(&format!("  <maker>{}</maker>\n", xml_escape(studio)));
        }
    }

    // Label
    if let Some(label) = metadata.get("label").and_then(|v| v.as_str()) {
        if !label.is_empty() {
            nfo.push_str(&format!("  <label>{}</label>\n", xml_escape(label)));
        }
    }

    // Series
    if let Some(series) = metadata.get("series").and_then(|v| v.as_str()) {
        if !series.is_empty() {
            nfo.push_str(&format!("  <set>{}</set>\n", xml_escape(series)));
        }
    }

    // Tags/Genres
    if let Some(tags) = metadata.get("tag").and_then(|v| v.as_array()) {
        for tag in tags {
            if let Some(tag_str) = tag.as_str() {
                if !tag_str.is_empty() {
                    nfo.push_str(&format!("  <tag>{}</tag>\n", xml_escape(tag_str)));
                    nfo.push_str(&format!("  <genre>{}</genre>\n", xml_escape(tag_str)));
                }
            }
        }
    }

    // Actors
    if let Some(actors) = metadata.get("actor").and_then(|v| v.as_array()) {
        let actor_photos = metadata.get("actor_photo").and_then(|v| v.as_object());

        for actor in actors {
            if let Some(actor_name) = actor.as_str() {
                if !actor_name.is_empty() {
                    nfo.push_str("  <actor>\n");
                    nfo.push_str(&format!("    <name>{}</name>\n", xml_escape(actor_name)));

                    // Add photo if available
                    if let Some(photos) = actor_photos {
                        if let Some(photo_url) = photos.get(actor_name).and_then(|v| v.as_str()) {
                            nfo.push_str(&format!("    <thumb>{}</thumb>\n", xml_escape(photo_url)));
                        }
                    }

                    nfo.push_str("  </actor>\n");
                }
            }
        }
    }

    // Cover/Poster
    if let Some(cover) = metadata.get("cover").and_then(|v| v.as_str()) {
        if !cover.is_empty() {
            nfo.push_str(&format!("  <thumb>{}</thumb>\n", xml_escape(cover)));
        }
    }

    // Fanart (same as cover for now)
    if let Some(cover) = metadata.get("cover").and_then(|v| v.as_str()) {
        if !cover.is_empty() {
            nfo.push_str("  <fanart>\n");
            nfo.push_str(&format!("    <thumb>{}</thumb>\n", xml_escape(cover)));
            nfo.push_str("  </fanart>\n");
        }
    }

    // Trailer
    if let Some(trailer) = metadata.get("trailer").and_then(|v| v.as_str()) {
        if !trailer.is_empty() {
            nfo.push_str(&format!("  <trailer>{}</trailer>\n", xml_escape(trailer)));
        }
    }

    // Movie number as unique ID
    nfo.push_str(&format!("  <num>{}</num>\n", xml_escape(movie_id)));
    nfo.push_str(&format!("  <id>{}</id>\n", xml_escape(movie_id)));

    // Website/Source
    if let Some(website) = metadata.get("website").and_then(|v| v.as_str()) {
        if !website.is_empty() {
            nfo.push_str(&format!("  <website>{}</website>\n", xml_escape(website)));
        }
    }

    // Source
    if let Some(source) = metadata.get("source").and_then(|v| v.as_str()) {
        if !source.is_empty() {
            nfo.push_str(&format!("  <source>{}</source>\n", xml_escape(source)));
        }
    }

    nfo.push_str("</movie>\n");

    Ok(nfo)
}

/// Write NFO file to disk
pub fn write_nfo(path: &Path, content: &str) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    tracing::info!("Created NFO file: {}", path.display());
    Ok(())
}

/// Escape XML special characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_generate_nfo_basic() {
        let metadata = json!({
            "title": "Test Movie",
            "number": "TEST-001",
            "year": "2024",
            "studio": "Test Studio",
            "outline": "A test movie description"
        });

        let nfo = generate_nfo(&metadata, "TEST-001").unwrap();

        assert!(nfo.contains("<title>Test Movie</title>"));
        assert!(nfo.contains("<year>2024</year>"));
        assert!(nfo.contains("<studio>Test Studio</studio>"));
        assert!(nfo.contains("<plot>A test movie description</plot>"));
        assert!(nfo.contains("<num>TEST-001</num>"));
    }

    #[test]
    fn test_generate_nfo_with_actors() {
        let metadata = json!({
            "title": "Test Movie",
            "number": "TEST-001",
            "actor": ["Actor One", "Actor Two"],
            "actor_photo": {
                "Actor One": "http://example.com/actor1.jpg"
            }
        });

        let nfo = generate_nfo(&metadata, "TEST-001").unwrap();

        assert!(nfo.contains("<name>Actor One</name>"));
        assert!(nfo.contains("<name>Actor Two</name>"));
        assert!(nfo.contains("http://example.com/actor1.jpg"));
    }

    #[test]
    fn test_generate_nfo_with_tags() {
        let metadata = json!({
            "title": "Test Movie",
            "number": "TEST-001",
            "tag": ["Action", "Drama", "Thriller"]
        });

        let nfo = generate_nfo(&metadata, "TEST-001").unwrap();

        assert!(nfo.contains("<tag>Action</tag>"));
        assert!(nfo.contains("<genre>Action</genre>"));
        assert!(nfo.contains("<tag>Drama</tag>"));
        assert!(nfo.contains("<tag>Thriller</tag>"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("Test & Movie"), "Test &amp; Movie");
        assert_eq!(xml_escape("Test < Movie"), "Test &lt; Movie");
        assert_eq!(xml_escape("Test > Movie"), "Test &gt; Movie");
        assert_eq!(xml_escape("Test \"Movie\""), "Test &quot;Movie&quot;");
        assert_eq!(xml_escape("Test's Movie"), "Test&apos;s Movie");
    }

    #[test]
    fn test_write_nfo() {
        let temp = TempDir::new().unwrap();
        let nfo_path = temp.path().join("movie.nfo");

        let content = "<movie><title>Test</title></movie>";
        write_nfo(&nfo_path, content).unwrap();

        assert!(nfo_path.exists());
        let read_content = std::fs::read_to_string(&nfo_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_generate_nfo_complete() {
        let metadata = json!({
            "title": "Complete Test Movie",
            "number": "TEST-001",
            "year": "2024",
            "release": "2024-03-15",
            "studio": "Test Studio",
            "director": "Test Director",
            "label": "Test Label",
            "series": "Test Series",
            "outline": "A complete test movie",
            "runtime": "120",
            "userrating": 8.5,
            "uservotes": 1000,
            "actor": ["Actor One"],
            "tag": ["Action"],
            "cover": "http://example.com/cover.jpg",
            "trailer": "http://example.com/trailer.mp4",
            "website": "http://example.com/movie",
            "source": "test_source"
        });

        let nfo = generate_nfo(&metadata, "TEST-001").unwrap();

        // Verify all fields are present
        assert!(nfo.contains("<title>Complete Test Movie</title>"));
        assert!(nfo.contains("<runtime>120</runtime>"));
        assert!(nfo.contains("<rating>8.5</rating>"));
        assert!(nfo.contains("<votes>1000</votes>"));
        assert!(nfo.contains("<director>Test Director</director>"));
        assert!(nfo.contains("<label>Test Label</label>"));
        assert!(nfo.contains("<set>Test Series</set>"));
        assert!(nfo.contains("<releasedate>2024-03-15</releasedate>"));
        assert!(nfo.contains("<trailer>http://example.com/trailer.mp4</trailer>"));
        assert!(nfo.contains("<website>http://example.com/movie</website>"));
    }
}
