# Quick Cookie Fix Guide

## Problem
Your scrapers are working correctly, but need **valid cookies** to bypass Cloudflare protection.

## Current Status
✅ Cookie system implemented and working
✅ Auto-detection from URLs working
❌ Cookie values in config are invalid/expired

## Quick Fix (5 Minutes)

### Method 1: Browser DevTools (Manual)

#### For JAVBus (Most Important)

1. **Open Chrome/Firefox**
2. **Visit**: https://www.javbus.com
3. **Complete Cloudflare challenge** (click checkbox)
4. **Press F12** → Go to **Application** tab (Chrome) or **Storage** tab (Firefox)
5. **Click Cookies** → `https://www.javbus.com`
6. **Find these cookies**:
   - `cf_clearance` - Cloudflare bypass token
   - `PHPSESSID` - Session cookie
7. **Copy the values** (long strings)
8. **Edit config**:
   ```bash
   nano ~/code/Movie_Data_Capture/rust/config.ini
   ```
9. **Replace line 86**:
   ```ini
   # OLD (invalid):
   www.javbus.com = PHPSESSID=4ufh6tjmkdq8th3a1cu46v5f14

   # NEW (with real values):
   www.javbus.com = cf_clearance=YOUR_CF_TOKEN_HERE,PHPSESSID=YOUR_SESSION_HERE
   ```

#### For JAVLibrary (Tier 2)

1. **Visit**: https://www.javlibrary.com/en/
2. **Complete verification**
3. **F12** → **Cookies**
4. **Find**:
   - `cf_clearance`
   - `over18` (or similar)
5. **Add to config**:
   ```ini
   www.javlibrary.com = cf_clearance=TOKEN,over18=1
   ```

#### For JavDB (Optional)

1. **Visit**: https://javdb.com
2. **F12** → **Cookies**
3. **Find**: `_jdb_session`
4. **Add to config**:
   ```ini
   javdb.com = _jdb_session=YOUR_SESSION_TOKEN
   ```

### Method 2: Browser Extension (Easier)

**Option A: Cookie-Editor Extension**
1. Install: [Cookie-Editor](https://cookie-editor.com/) (Chrome/Firefox)
2. Visit www.javbus.com
3. Click Cookie-Editor icon
4. Export as "Header String"
5. Paste into config

**Option B: EditThisCookie (Chrome Only)**
1. Install from Chrome Web Store
2. Visit www.javbus.com
3. Click extension → Export
4. Copy cookie string

### Cookie Format Rules

**Single cookie**:
```ini
domain.com = cookiename=value
```

**Multiple cookies** (comma-separated):
```ini
domain.com = cookie1=value1,cookie2=value2
```

**Example (real format)**:
```ini
www.javbus.com = cf_clearance=0pJN.kM5.abc123-xyz789,PHPSESSID=abc123def456
```

## Test Your Cookies

```bash
cd ~/code/Movie_Data_Capture/rust
python3 test_cookies.py
```

**Expected output**:
```
✅ Loaded cookies for 3 domain(s):
   - www.javbus.com
   - www.javlibrary.com
   - javdb.com

🌐 Testing www.javbus.com...
   ✅ SUCCESS - Page loaded without Cloudflare block
```

## Test Real Scraping

```bash
./target/release/mdc-cli /tmp/test-movies/SNIS-091.mp4 -m 3
```

**Expected**:
- Success rate should jump from 0% → 60-70%
- Should see metadata retrieved from JAVBus or JAVLibrary

## Common Issues

### Issue 1: "Cloudflare challenge" still appearing
**Solution**: Your `cf_clearance` cookie expired (they last ~30 minutes)
- Revisit the site in browser
- Complete challenge again
- Extract fresh cookie

### Issue 2: "Invalid cookie format"
**Solution**: Check for spaces
```ini
# WRONG (spaces around =):
www.javbus.com = PHPSESSID = value

# RIGHT (no spaces):
www.javbus.com = PHPSESSID=value
```

### Issue 3: Cookies not being used
**Solution**: Check domain matches
```bash
# In debug logs, check:
[javbus] Using cookies for www.javbus.com (auto-detected)

# Config must have matching domain:
www.javbus.com = cookie=value  # ✅ Correct
javbus.com = cookie=value      # ❌ Wrong (missing www)
```

## Cookie Lifespan

- **cf_clearance**: ~30 minutes
- **PHPSESSID**: Until browser closes
- **_jdb_session**: 24 hours

**Recommendation**: Extract fresh cookies before each batch scraping session.

## Priority Domains

Focus on these in order:

1. **www.javbus.com** - Most comprehensive, works with cookies
2. **www.javlibrary.com** - Best for older titles
3. **javdb.com** - Good for recent titles + FC2 content

DMM and R18Dev don't need cookies (tier 1 official sources).

## Automation (Optional)

For regular scraping, consider:
1. Create a shell alias to refresh cookies
2. Use Selenium/Puppeteer to auto-solve Cloudflare
3. Use FlareSolverr (like Javinizer does)

## Verification Checklist

- [ ] Visited sites in browser successfully
- [ ] Extracted cookies from DevTools
- [ ] Updated config.ini with real values
- [ ] Removed spaces from cookie format
- [ ] Domain matches exactly (including www if needed)
- [ ] Tested with `python3 test_cookies.py`
- [ ] Ran real scraping test

Once done, your scrapers will work like Javinizer! 🎉
