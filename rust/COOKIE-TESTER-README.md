# Cookie Configuration Tester

## Quick Start

Test if your cookie configuration is working:

```bash
cd /home/simon/code/Movie_Data_Capture/rust
python3 test_cookies.py
```

## What This Script Does

The cookie tester helps you verify that:
1. ✅ Config file is found and readable
2. ✅ Cookies are properly formatted
3. ✅ Cookies can successfully authenticate with JAV sites
4. ✅ Sites return expected content (not Cloudflare blocks)

## Step-by-Step Setup Guide

### 1. Copy Example Config

```bash
# Create config directory
mkdir -p ~/.mdc

# Copy example config
cp config.ini.example ~/.mdc/config.ini
```

### 2. Extract Cookies from Browser

#### For JavDB (Highest Priority)

1. **Open JavDB in your browser**:
   - Visit: https://javdb.com
   - Complete any age verification

2. **Open DevTools**:
   - Press `F12` (or right-click → Inspect)
   - Go to **Application** tab (Chrome) or **Storage** tab (Firefox)

3. **Find Cookies**:
   - Navigate to **Cookies** → `https://javdb.com`
   - Look for `_jdb_session` cookie

4. **Copy Cookie Value**:
   - Click on the cookie
   - Copy the entire **Value** field (long alphanumeric string)

5. **Add to config**:
   ```bash
   nano ~/.mdc/config.ini
   ```

   Replace this line:
   ```ini
   javdb.com = _jdb_session=PASTE_YOUR_SESSION_TOKEN_HERE
   ```

   With your actual cookie:
   ```ini
   javdb.com = _jdb_session=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
   ```

#### For JAVBus (Secondary Priority)

1. Visit: https://javbus.com (or https://www.javbus.com)
2. Complete Cloudflare challenge if shown
3. Open DevTools → Application → Cookies
4. Copy `cf_clearance` cookie value
5. Add to config:
   ```ini
   javbus.com = cf_clearance=abc123def456...
   ```

#### For JAVLibrary

1. Visit: https://www.javlibrary.com
2. Access any page successfully
3. Open DevTools → Cookies
4. Look for session cookies (name varies)
5. Add to config with actual cookie name

### 3. Test Your Configuration

```bash
python3 test_cookies.py
```

**Expected Success Output**:
```
======================================================================
MovieMeta Cookie Configuration Tester
======================================================================

📂 Loading config from: /home/user/.mdc/config.ini

✅ Loaded cookies for 2 domain(s):
   - javdb.com: _jdb_session
   - javbus.com: cf_clearance

======================================================================
Testing Cookie Authentication
======================================================================

🔍 Testing javdb.com...
   URL: https://javdb.com/search?q=SSIS-001
   Cookies: _jdb_session
   ✅ SUCCESS (1.2s, 45678 bytes)

🔍 Testing javbus.com...
   URL: https://www.javbus.com/SSIS-001
   Cookies: cf_clearance
   ✅ SUCCESS (2.5s, 67890 bytes)

======================================================================
Summary
======================================================================

✅ Working: 2/2

✅ Successfully authenticated:
   - javdb.com
   - javbus.com

======================================================================
Recommendations
======================================================================

🎉 All configured scrapers are working!

You're ready to process files:
  ./mdc-cli /path/to/movies -s
```

### 4. Process Your Files

Once the tester shows success:

```bash
# Test with a single file
./target/release/mdc-cli "/tmp/test-movies/JUL-334.mp4" -s -g

# Process full directory
./target/release/mdc-cli "/path/to/Big Tune" -s
```

## Troubleshooting

### ❌ No config file found

**Error**:
```
❌ No config file found in standard locations
```

**Fix**:
```bash
mkdir -p ~/.mdc
cp config.ini.example ~/.mdc/config.ini
nano ~/.mdc/config.ini  # Add your cookies
```

### ❌ No [cookies] section found

**Error**:
```
❌ No [cookies] section found in config file
```

**Fix**: Add a `[cookies]` section to your config:
```ini
[cookies]
javdb.com = _jdb_session=YOUR_TOKEN
```

### ❌ FORBIDDEN (403)

**Error**:
```
🔍 Testing javdb.com...
   ❌ FORBIDDEN (403)
```

**Causes**:
1. **Cookie expired** - Re-extract from browser
2. **Wrong cookie value** - Check you copied the entire value
3. **Wrong domain** - Ensure domain matches exactly (javdb.com vs www.javdb.com)

**Fix**: Extract fresh cookies and update config

### ⚠️ Cloudflare/Bot protection detected

**Error**:
```
   ⚠️  Cloudflare/Bot protection detected
```

**Causes**:
1. Cookie not sufficient
2. Additional cookies required
3. Site added new protection

**Fix**: Try these in order:
1. Extract ALL cookies from the site (not just session cookie)
2. Use CloudScraper backend (automatic in MovieMeta)
3. Try a different scraper source

### ⏱️ TIMEOUT

**Error**:
```
   ❌ TIMEOUT (>15s)
```

**Causes**:
1. Slow internet connection
2. Site is down
3. Firewall blocking

**Fix**:
- Check internet connection
- Try again later
- Check if site is accessible in browser

## Understanding Test Results

### Status Indicators

| Status | Meaning | Action |
|--------|---------|--------|
| ✅ SUCCESS | Cookie works perfectly | Ready to use |
| ❌ FORBIDDEN (403) | Cookie rejected/expired | Re-extract cookie |
| ⚠️ Cloudflare detected | Bot protection active | Update cookies or use CloudScraper |
| ⚠️ Content unexpected | Page structure changed | Scraper may need update |
| ⏭️ Skipped | No cookie configured | Optional - add if needed |
| ⏱️ Timeout | Connection too slow | Check network |

## Cookie Lifespan

| Site | Typical Lifespan | Re-extract When |
|------|------------------|-----------------|
| JavDB | 1-7 days | You see 403 errors |
| JAVBus | Hours-days | Cloudflare challenge reappears |
| JAVLibrary | Varies | Access denied |

**Best Practice**: Re-extract cookies when you start seeing failures

## Security Notes

### Protect Your Cookies

Cookies are like passwords! Keep them secure:

```bash
# Set restrictive permissions
chmod 600 ~/.mdc/config.ini

# Never share
# Never commit to git
# Don't post in issues
```

### Add to .gitignore

If you're tracking this repo:

```bash
echo "config.ini" >> .gitignore
echo "*.ini" >> .gitignore
```

## Advanced Usage

### Test Specific Config File

```bash
python3 test_cookies.py /path/to/custom/config.ini
```

### Test Without Installing

The tester only requires:
- Python 3.6+
- `cloudscraper` library
- `requests` library

Install dependencies:
```bash
pip3 install cloudscraper requests
```

### Multiple Cookie Profiles

Create different configs for different scenarios:

```bash
~/.mdc/config.home.ini       # Home network cookies
~/.mdc/config.vpn.ini        # VPN network cookies
~/.mdc/config.all.ini        # All available cookies

# Test each
python3 test_cookies.py ~/.mdc/config.home.ini
python3 test_cookies.py ~/.mdc/config.vpn.ini
```

## Integration with MovieMeta

### How Cookies Are Used

```
test_cookies.py               ← Manual verification tool
     ↓
~/.mdc/config.ini            ← Cookie storage
     ↓
mdc-storage/config.rs        ← Config parser
     ↓
mdc-scraper/scraper.rs       ← ScraperConfig with cookies
     ↓
mdc-scraper/client.rs        ← HTTP client adds Cookie header
     ↓
mdc-scraper/scrapers/*.rs    ← Individual scrapers use cookies
     ↓
JavDB/JAVBus/etc             ← Site returns authenticated content
```

### Automatic Cookie Loading

When you run:
```bash
./mdc-cli /path/to/movies -s
```

The system automatically:
1. Searches for config.ini in standard locations
2. Loads `[cookies]` section
3. Applies cookies to matching domains
4. Falls back to CloudScraper if cookies fail

You don't need to pass any flags - cookies are used automatically!

## FAQ

**Q: Do I need cookies for all sites?**
A: No, only for sites blocking you. Try without cookies first.

**Q: Can I use cookies from private/incognito mode?**
A: Yes, but they may expire faster when you close the browser.

**Q: Will cookies work with CloudScraper backend?**
A: CloudScraper manages its own cookies. Config cookies are for reqwest backend only.

**Q: How many sites should I configure?**
A: Start with JavDB and JAVBus. Add others only if those fail.

**Q: What if test passes but actual scraping fails?**
A: The scraper may need the movie-specific URL. Check with `-g` debug flag.

**Q: Can I share cookies with friends?**
A: Not recommended. Cookies are tied to your browser session.

## Next Steps

After cookies are working:

1. ✅ **Test with failing files**: Try JUL-334, PPPD-887 again
2. ✅ **Re-run Big Tune batch**: Expect 70-85% success rate
3. ✅ **Monitor for expiration**: Re-extract when you see 403s
4. ✅ **Report issues**: If certain titles still fail consistently

## See Also

- [COOKIE-CONFIGURATION.md](COOKIE-CONFIGURATION.md) - Detailed cookie guide
- [USER-GUIDE.md](USER-GUIDE.md) - Complete user manual
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - General troubleshooting
- [config.ini.example](config.ini.example) - Example configuration

---

**Script Location**: `/home/simon/code/Movie_Data_Capture/rust/test_cookies.py`
**Last Updated**: 2025-12-31
