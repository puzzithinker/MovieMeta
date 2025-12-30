# Cookie Configuration Guide

## Overview

MovieMeta now supports persistent cookie configuration for scrapers like JavDB and JAVBus. This allows you to bypass bot protection (Cloudflare, etc.) and access authenticated content without needing to pass `--cookie` flags every time.

## Why Cookies Are Needed

Some JAV scraper websites implement protection measures:
- **JavDB**: May require `_jdb_session` cookie for authenticated requests
- **JAVBus**: May use Cloudflare protection requiring `cf_clearance` cookie
- **Other sites**: Similar anti-bot measures

Without proper cookies, you may encounter:
- 403 Forbidden errors
- "Just a moment..." Cloudflare pages
- Blocked or rate-limited requests

## Quick Start

### Step 1: Create Config File

Create a config file in one of these locations (first found will be used):
1. `./config.ini` (current directory)
2. `~/mdc.ini`
3. `~/.mdc.ini`
4. `~/.mdc/config.ini`
5. `~/.config/mdc/config.ini`

**Recommended**: Use `~/.mdc/config.ini` for persistent configuration

```bash
mkdir -p ~/.mdc
cp config.ini.example ~/.mdc/config.ini
```

### Step 2: Get Your Cookies

#### For JavDB:
1. Visit https://javdb.com in your browser
2. Login or complete any verification if prompted
3. Open Browser DevTools (F12)
4. Go to **Application** tab (Chrome) or **Storage** tab (Firefox)
5. Navigate to **Cookies** → `https://javdb.com`
6. Find the `_jdb_session` cookie
7. Copy its **Value** (long alphanumeric string)

#### For JAVBus:
1. Visit https://javbus.com in your browser
2. If you see a Cloudflare challenge, complete it
3. Open Browser DevTools (F12)
4. Go to **Application** → **Cookies** → `https://javbus.com`
5. Find the `cf_clearance` cookie
6. Copy its **Value**

### Step 3: Add Cookies to Config

Edit your `config.ini` file and add the `[cookies]` section:

```ini
[cookies]
# JavDB session cookie
javdb.com = _jdb_session=YOUR_SESSION_TOKEN_HERE

# JAVBus Cloudflare clearance (if needed)
javbus.com = cf_clearance=YOUR_CF_TOKEN_HERE
```

**Multiple cookies per domain** (comma-separated):
```ini
[cookies]
javdb.com = _jdb_session=abc123def456,over18=1
```

### Step 4: Run MovieMeta

Cookies will be loaded automatically:

```bash
./mdc-cli /path/to/movies -s
```

You'll see output like:
```
Loaded cookies for domains: javdb.com, javbus.com
```

## Configuration Examples

### Example 1: JavDB Only
```ini
[cookies]
javdb.com = _jdb_session=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Example 2: Multiple Sites
```ini
[cookies]
javdb.com = _jdb_session=abc123def456
javbus.com = cf_clearance=xyz789ghi012
avmoo.com = session_id=qwerty123
```

### Example 3: Multiple Cookies Per Site
```ini
[cookies]
javdb.com = _jdb_session=abc123,over18=1,locale=en
```

## How It Works

### Architecture

1. **Config Loading** (`mdc-storage/src/config.rs`):
   - Reads `[cookies]` section from INI file
   - Parses domain-based cookie configuration
   - Returns `HashMap<String, HashMap<String, String>>`

2. **Scraper Configuration** (`mdc-scraper/src/scraper.rs`):
   - `ScraperConfig` holds cookies for all domains
   - `get_cookie_header(domain)` formats cookies for HTTP headers

3. **HTTP Client** (`mdc-scraper/src/client.rs`):
   - `get_with_cookies(url, cookie_header)` makes authenticated requests
   - Automatic fallback to CloudScraper if needed

4. **Scraper Usage** (`mdc-scraper/src/scrapers/javdb.rs`):
   - Checks if cookies exist for domain
   - Adds Cookie header to HTTP requests
   - Works transparently with existing code

### Cookie Lifecycle

```
Config File → Config::load() → cookies: HashMap
                                      ↓
                              ScraperConfig::cookies()
                                      ↓
                   ScraperConfig::get_cookie_header("javdb.com")
                                      ↓
                         Format: "name1=value1; name2=value2"
                                      ↓
                    ScraperClient::get_with_cookies(url, cookies)
                                      ↓
                           HTTP Request with Cookie header
```

## Supported Scrapers

Currently, cookie support is implemented for:
- ✅ **JavDB** - Full cookie support (search + detail pages)
- ⏳ **JAVBus** - Infrastructure ready, can be enabled easily
- ⏳ **AVMOO** - Infrastructure ready, can be enabled easily

### Adding Cookie Support to Other Scrapers

Any scraper can use cookies by overriding `fetch_html()`:

```rust
async fn fetch_html(&self, url: &str, config: &ScraperConfig) -> Result<Html> {
    let html_text = if let Some(cookie_header) = config.get_cookie_header("example.com") {
        config.client.get_with_cookies(url, Some(&cookie_header)).await?
    } else {
        config.client.get(url).await?
    };

    Ok(Html::parse_document(&html_text))
}
```

## Troubleshooting

### Cookies Not Working

1. **Verify config location**:
   ```bash
   # Check which config file is being loaded
   RUST_LOG=debug ./mdc-cli /path -s 2>&1 | grep "Loading config from"
   ```

2. **Check cookie format**:
   - Must be in `[cookies]` section
   - Format: `domain = name=value` or `domain = name1=value1,name2=value2`
   - No extra spaces around `=` signs

3. **Cookie expired**:
   - Session cookies may expire after a few hours/days
   - Re-extract cookies from browser and update config

4. **Debug mode**:
   ```bash
   ./mdc-cli /path -s -g  # Enable debug output
   ```
   Look for:
   ```
   [JavDB] Using cookies for request
   ```

### Config File Not Found

If you see: `Config file not found in any of the following locations...`

**Solution**: Create config file in one of the listed paths:
```bash
mkdir -p ~/.mdc
touch ~/.mdc/config.ini
# Add [cookies] section
```

### 403 Errors Despite Cookies

1. **Cookie value incorrect**: Re-extract from browser
2. **Additional cookies needed**: Some sites require multiple cookies
3. **IP-based blocking**: Cookie alone may not be enough
4. **CloudScraper fallback**: MovieMeta will automatically try CloudScraper

## Security Considerations

### Cookie Safety

⚠️ **Cookies are sensitive credentials!**

- Store config file with restricted permissions:
  ```bash
  chmod 600 ~/.mdc/config.ini
  ```

- **Never share** your config file with cookies
- **Never commit** config.ini to version control
- Use `.gitignore`:
  ```
  config.ini
  ```

### Cookie Lifetime

- **Session cookies**: Expire when browser closes or after timeout
- **Persistent cookies**: May last days/weeks/months
- **Update regularly**: Re-extract cookies if errors occur

## Advanced Usage

### Environment-Specific Configs

**Development**:
```bash
# Use local config
cp config.ini.example ./config.ini
# Edit with dev cookies
./mdc-cli /path -s
```

**Production**:
```bash
# Use system-wide config
sudo cp config.ini.example /etc/mdc/config.ini
# Edit with prod cookies
```

### Multiple Profiles

Create different config files:
```bash
~/.mdc/config.javdb.ini    # JavDB-only cookies
~/.mdc/config.javbus.ini   # JAVBus-only cookies
~/.mdc/config.all.ini      # All cookies
```

Note: Currently only one config file is loaded (first found). Multiple profile support may be added in the future.

### Programmatic Cookie Management

For advanced users, cookies can be set programmatically:

```rust
use std::collections::HashMap;

let mut cookies = HashMap::new();
let mut javdb_cookies = HashMap::new();
javdb_cookies.insert("_jdb_session".to_string(), "abc123".to_string());
cookies.insert("javdb.com".to_string(), javdb_cookies);

let scraper_config = ScraperConfig::new(client)
    .cookies(cookies);
```

## FAQs

**Q: Do I need cookies for all scrapers?**
A: No, only for sites with bot protection. Try without cookies first.

**Q: Can I use cookies from Incognito/Private mode?**
A: Yes, but they may expire faster when the browser window closes.

**Q: Will cookies work with CloudScraper backend?**
A: CloudScraper manages its own cookies internally. Config cookies are for reqwest backend only.

**Q: Can I share cookies between multiple users?**
A: Not recommended. Cookies are tied to browser sessions and may be user-specific.

**Q: How often do I need to update cookies?**
A: Depends on the site. Some last days, others expire in hours. Update when you see 403 errors.

## See Also

- [QUICKSTART.md](QUICKSTART.md) - Getting started guide
- [USER-GUIDE.md](USER-GUIDE.md) - Comprehensive user manual
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Problem solving guide
- [config.ini.example](config.ini.example) - Example configuration file

## Support

If you encounter issues with cookie configuration:

1. Check this guide's Troubleshooting section
2. Enable debug mode: `./mdc-cli -g`
3. Check logs for cookie-related messages
4. Report issues at: https://github.com/anthropics/claude-code/issues

---

**Last Updated**: 2025-12-29
**Javinizer Integration**: Phase 5 Complete + Cookie Configuration
