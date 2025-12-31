# Cloudflare Bypass Guide (No Cookie Extraction Needed!)

## Problem
Can't find `cf_clearance` cookie for JAVBus? **You don't need it!**

## Solution: Use CloudScraper (Already Built-In)

We have **two automatic solutions** that don't require manual cookie extraction:

---

## Option 1: CloudScraper (Recommended) ⚡

### What is CloudScraper?

CloudScraper is a Python library that automatically solves Cloudflare JavaScript challenges. It's already installed and integrated into your MovieMeta build!

**Advantages**:
- ✅ No manual cookie extraction
- ✅ Automatically bypasses Cloudflare
- ✅ Already installed (`cloudscraper 1.2.71`)
- ✅ Works for most JAV sites
- ✅ Built-in to mdc-scraper

### Quick Test

Test if CloudScraper can access JAVBus:

```bash
cd ~/code/Movie_Data_Capture/rust/mdc-scraper

# Test direct access
python3 cloudflare_bridge.py "https://www.javbus.com/SSIS-001"
```

**Expected output**: HTML content of the page (bypassing Cloudflare)

**If it fails**: Try with custom user-agent:

```bash
python3 cloudflare_bridge.py "https://www.javbus.com/SSIS-001" \
  --user-agent "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"
```

### Enable CloudScraper for JAVBus

The CloudScraper integration exists but isn't enabled by default. To use it, we'd need to update the JAVBus scraper to use CloudScraperClient instead of the regular HTTP client.

**Current status**: CloudScraper infrastructure ready, needs scraper integration.

---

## Option 2: Try Other JAVBus Cookies (Simpler)

JAVBus might work with **session cookies** instead of cf_clearance:

### Step 1: Visit JAVBus

1. Open https://www.javbus.com in your browser
2. Wait for the page to fully load (age verification, etc.)

### Step 2: Check Available Cookies

Press **F12** → **Application** → **Cookies** → `https://www.javbus.com`

Look for **ANY** of these cookies:
- `PHPSESSID` - PHP session
- `__cfduid` - Cloudflare user ID (deprecated but might still work)
- `cf_chl_2` - Cloudflare challenge cookie
- `cf_chl_prog` - Challenge progress
- Any cookie with `cf_` prefix

### Step 3: Update Config (Try Without cf_clearance)

Edit `~/code/Movie_Data_Capture/rust/config.ini`:

```ini
[cookies]
# Try with just PHPSESSID (if available)
www.javbus.com = PHPSESSID=YOUR_SESSION_ID_HERE

# Or combine multiple cookies you found
www.javbus.com = PHPSESSID=xxx,__cfduid=yyy
```

### Step 4: Test

```bash
cd ~/code/Movie_Data_Capture/rust
python3 test_cookies.py
```

---

## Option 3: FlareSolverr (Javinizer's Approach) 🚀

FlareSolverr is a proxy service that automatically solves Cloudflare challenges. This is what Javinizer recommends!

### Installation (Docker)

```bash
# Start FlareSolverr
docker run -d \
  --name=flaresolverr \
  -p 8191:8191 \
  -e LOG_LEVEL=info \
  --restart unless-stopped \
  ghcr.io/flaresolverr/flaresolverr:latest
```

### Test FlareSolverr

```bash
# Test if it's working
curl -X POST http://localhost:8191/v1 \
  -H "Content-Type: application/json" \
  -d '{
    "cmd": "request.get",
    "url": "https://www.javbus.com/SSIS-001"
  }'
```

**Expected**: JSON response with HTML content and cookies

### Integration with MovieMeta

FlareSolverr provides cookies that can be extracted from its responses. We'd need to:
1. Call FlareSolverr API to get page + cookies
2. Extract cookies from response
3. Use them in subsequent requests

**Status**: Would need implementation in scrapers.

---

## Option 4: Manual Cookie Extraction with Browser Extensions

If you still want to try getting cf_clearance manually:

### Using Cookie-Editor Extension

1. Install [Cookie-Editor](https://cookie-editor.com/) for Chrome/Firefox
2. Visit https://www.javbus.com
3. Wait for Cloudflare challenge to complete
4. Click Cookie-Editor icon
5. Export all cookies as "Header String"
6. Paste into config.ini

**Note**: Even if cf_clearance isn't present, other cookies might work!

---

## Recommended Solution for You

Since you can't find `cf_clearance`, I recommend this order:

### 1. Try Without cf_clearance First (1 minute)

```ini
# In config.ini, just use whatever cookies you CAN find
www.javbus.com = PHPSESSID=YOUR_SESSION_VALUE
```

Test:
```bash
./target/release/mdc-cli /tmp/test-movies/SSIS-001.mp4 -m 3 -g
```

Look for: `[javbus] Using cookies for www.javbus.com`

### 2. Use CloudScraper Bridge Directly (5 minutes)

Test if CloudScraper can bypass Cloudflare for you:

```bash
cd ~/code/Movie_Data_Capture/rust/mdc-scraper

# Test JAVBus access
python3 cloudflare_bridge.py "https://www.javbus.com/SSIS-001" > /tmp/javbus-test.html

# Check if it worked
grep -i "ssis-001\|cloudflare" /tmp/javbus-test.html
```

**If successful**: You'll see the actual page HTML, not Cloudflare challenge page!

### 3. Install FlareSolverr (15 minutes, most reliable)

This is what Javinizer users do for persistent scraping:

```bash
# Install via Docker
docker run -d --name=flaresolverr -p 8191:8191 ghcr.io/flaresolverr/flaresolverr:latest

# Test it
curl -X POST http://localhost:8191/v1 \
  -H "Content-Type: application/json" \
  -d '{"cmd": "request.get", "url": "https://www.javbus.com/SSIS-001"}' | jq
```

---

## Why cf_clearance is Hard to Find

According to [web scraping guides](https://roundproxies.com/blog/cf-clearance/):

1. **Not always issued** - Cloudflare uses different challenge types:
   - JavaScript Challenge → cf_clearance cookie
   - Managed Challenge → Different cookies
   - Interactive Challenge → CAPTCHA, no cookie

2. **Depends on threat level** - If Cloudflare doesn't detect automation:
   - Low threat: No challenge, no cf_clearance
   - Medium threat: JavaScript challenge, cf_clearance issued
   - High threat: CAPTCHA required

3. **Browser fingerprinting** - Your browser session might pass without challenge

4. **httpOnly flag** - Cookie might be there but hidden from JavaScript/DevTools

---

## Testing Each Approach

### Test 1: Session Cookies Only
```bash
# Update config.ini with PHPSESSID
./target/release/mdc-cli /tmp/test-movies/SSIS-001.mp4 -m 3 -g 2>&1 | grep -i "javbus\|cloudflare"
```

### Test 2: CloudScraper Bridge
```bash
cd mdc-scraper
python3 cloudflare_bridge.py "https://www.javbus.com/SSIS-001" | head -50
```

### Test 3: FlareSolverr
```bash
curl -X POST http://localhost:8191/v1 \
  -H "Content-Type: application/json" \
  -d '{"cmd": "request.get", "url": "https://www.javbus.com/SSIS-001"}' \
  | jq '.solution.cookies'
```

---

## What About Other Sites?

### JAVLibrary
- Also Cloudflare protected
- Same solutions apply (CloudScraper/FlareSolverr)
- Might have more aggressive protection

### JavDB
- Usually easier (less Cloudflare)
- Often just needs `_jdb_session` cookie
- Try accessing https://javdb.com first

### DMM / R18Dev
- **No cookies needed!** These are tier 1 straightforward sources
- Direct API/HTTP access
- Only issue: content availability (not all titles in database)

---

## Quick Decision Tree

```
Can you access www.javbus.com in your browser?
│
├─ YES → Are there ANY cookies visible in DevTools?
│   │
│   ├─ YES → Copy those cookies to config.ini and test
│   │          (PHPSESSID, __cfduid, anything you see)
│   │
│   └─ NO → Try CloudScraper bridge:
│              python3 cloudflare_bridge.py URL
│
└─ NO → Cloudflare is blocking your IP
          → Need VPN or FlareSolverr
```

---

## Success Criteria

After implementing one of these solutions, you should see:

```
✅ [javbus] Using cookies for www.javbus.com (auto-detected)
✅ Scrapers tried: 12
✅ Scrapers succeeded: 1
✅ Success Rate: 100%
```

Instead of:
```
❌ Failed to extract title from JAVBus
❌ Error scraping from 'javbus': Cloudflare challenge
```

---

## My Recommendation

**Start with #2 (CloudScraper bridge test)**:

```bash
cd ~/code/Movie_Data_Capture/rust/mdc-scraper
python3 cloudflare_bridge.py "https://www.javbus.com/SSIS-001" > /tmp/test.html
cat /tmp/test.html | grep -i "title\|ssis"
```

If you see the actual page title (not "Just a moment..." or Cloudflare), CloudScraper is working and we can integrate it into the scrapers!

If CloudScraper fails too, then **FlareSolverr** is the most reliable solution (same as Javinizer).

---

**Sources**:
- [ZenRows: How to Scrape cf_clearance](https://www.zenrows.com/blog/cf-clearance)
- [GitHub: CF-Clearance-Scraper](https://github.com/Xewdy444/CF-Clearance-Scraper)
- [Scrapeless: Managing cf_clearance](https://www.scrapeless.com/en/blog/cf-clearance)
- [RoundProxies: cf_clearance Methods](https://roundproxies.com/blog/cf-clearance/)
