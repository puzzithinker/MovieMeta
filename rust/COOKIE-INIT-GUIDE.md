# Cookie Initialization Guide

## Quick Start (2 Minutes)

The easiest way to set up JAVBus cookies is using our interactive helper:

```bash
cd ~/code/Movie_Data_Capture/rust
python3 init_config.py
```

The script will:
1. ✅ Check if cookies are already configured
2. 📖 Guide you through browser cookie extraction
3. ✅ Validate cookies against JAVBus server
4. 💾 Create/update config.ini automatically
5. 🎉 Confirm everything is working

## What You'll Need

- **2 minutes** of your time
- **A web browser** (Chrome, Firefox, Edge, Safari)
- **Internet connection** to access JAVBus

## Interactive Flow

### Example Session

```
╔══════════════════════════════════════════════════════════════════╗
║                    MovieMeta Configuration Initializer           ║
║                        JAVBus Cookie Setup                       ║
╚══════════════════════════════════════════════════════════════════╝

Checking current configuration...

❌ No config.ini found in standard locations
💡 Will create: /home/simon/.mdc/config.ini

Let's set up your JAVBus cookies! This will take ~2 minutes.

======================================================================
Step 1: Open JAVBus in your browser
======================================================================

1. Open Chrome, Firefox, or any modern browser
2. Go to: https://www.javbus.com
3. Complete the age verification (click "Enter" button)
4. Wait for the page to fully load

Press ENTER when you've completed age verification...

======================================================================
Step 2: Extract cookies from browser
======================================================================

Now we need to extract cookies from your browser:

For Chrome/Edge:
  1. Press F12 to open DevTools
  2. Click 'Application' tab at the top
  3. Expand 'Cookies' in the left sidebar
  4. Click on 'https://www.javbus.com'

For Firefox:
  1. Press F12 to open DevTools
  2. Click 'Storage' tab at the top
  3. Expand 'Cookies' in the left sidebar
  4. Click on 'https://www.javbus.com'

You should see a table with cookie names and values.

Look for these THREE cookies (you need all three):
  • dv          (value is usually '1')
  • existmag    (value is usually 'mag')
  • PHPSESSID   (long random string like '5ti9138au9ih2d3gdirp60gdo1')

Press ENTER when you can see the cookies...

======================================================================
Step 3: Enter cookie values
======================================================================

For each cookie, find it in the DevTools table and copy its VALUE.

Cookie 1: dv
  Default value: 1
  Copy the VALUE from the 'Value' column in DevTools
  Enter dv value [1]: 1

Cookie 2: existmag
  Default value: mag
  Copy the VALUE from the 'Value' column in DevTools
  Enter existmag value [mag]: mag

Cookie 3: PHPSESSID
  This is a LONG random string (20+ characters)
  Example: 5ti9138au9ih2d3gdirp60gdo1
  Copy the COMPLETE VALUE from DevTools
  Enter PHPSESSID value: 5ti9138au9ih2d3gdirp60gdo1

======================================================================
Step 4: Validating cookies
======================================================================

🌐 Testing connection to JAVBus...
   URL: https://www.javbus.com/SSIS-001
   Cookies: dv, existmag, PHPSESSID

   ✅ Successfully retrieved test page

✅ Cookies are valid and working!

======================================================================
Step 5: Writing configuration
======================================================================

📝 Created: /home/simon/.mdc/config.ini
✅ JAVBus cookies configured
🔒 Set secure permissions (600)

======================================================================
Configuration Complete!
======================================================================

🎉 Your JAVBus cookies are configured and working!

You can now run MovieMeta to scrape metadata:

  cd /home/simon/code/Movie_Data_Capture/rust
  ./target/release/mdc-cli /path/to/movies -m 1 -s

Or test with a specific file:
  ./target/release/mdc-cli /path/to/SSIS-001.mp4 -m 3 -g

📝 Configuration file: /home/simon/.mdc/config.ini

Note: Cookies typically last 24 hours to 9 months.
Re-run this script if you see authentication errors.

For testing your cookies anytime, run:
  python3 test_cookies.py
```

## What The Script Does

### 1. Config Detection
- Checks standard locations for existing config.ini
- Loads and parses existing configuration
- Determines if cookies need to be added/updated

### 2. Cookie Status Check
- Looks for JAVBus cookies in config
- Validates cookies against live server
- Reports if cookies are working, expired, or missing

### 3. Guided Extraction
- Step-by-step browser instructions
- Clear prompts for each cookie value
- Validation of input format

### 4. Server Validation
- Tests cookies against https://www.javbus.com/SSIS-001
- Checks for success indicators in response
- Detects Cloudflare blocks or invalid cookies

### 5. Config Initialization
- Creates config.ini in recommended location
- Writes [cookies] section with validated values
- Sets secure file permissions (Unix/Linux)

## The Three Required Cookies

### dv (Driver Verify)
- **Purpose**: Age verification flag
- **Value**: Usually `1`
- **How it works**: Set by server after you click "Enter" on age verification page
- **Important**: Must complete age verification first to get this cookie

### existmag (Magazine Access)
- **Purpose**: Content access permission
- **Value**: Usually `mag`
- **How it works**: Server-side flag for magazine/content access
- **Important**: Some browsers show this, others don't - try `mag` if not visible

### PHPSESSID (PHP Session ID)
- **Purpose**: Session identifier
- **Value**: 20-30 character random string (e.g., `5ti9138au9ih2d3gdirp60gdo1`)
- **How it works**: Server generates this when you first visit the site
- **Important**: Must be copied completely - missing even one character breaks it

## Troubleshooting

### "Cookie validation failed"

**Common Causes**:

1. **PHPSESSID incomplete**
   - Solution: Go back and copy the ENTIRE value from DevTools
   - Check: Should be 20+ characters long

2. **Wrong domain**
   - Solution: Extract from `https://www.javbus.com` (note the `www`)
   - NOT from `javbus.com` without `www`

3. **Age verification not completed**
   - Solution: Click the "Enter" button on JAVBus first
   - The `dv=1` cookie only appears AFTER verification

4. **Cookies expired during extraction**
   - Solution: Try the entire process again from the beginning
   - Work quickly once you start extracting

### "Can't find PHPSESSID cookie"

**Check**:
1. Make sure you're looking at the right domain (`www.javbus.com`)
2. Scroll down in the cookies list - it might be alphabetically sorted
3. Look for any cookie with "session" or "sess" in the name
4. If truly missing, refresh the page and check again

### "Can't find dv cookie"

**This means you didn't complete age verification**:
1. Visit https://www.javbus.com
2. Look for age verification page (18+ warning)
3. Click the "Enter" or confirmation button
4. ONLY THEN will the `dv=1` cookie appear

### "Script says cookies are expired"

**Solution**:
- Re-run the script and extract fresh cookies
- Cookies typically last:
  - Minimum: 30 minutes
  - Typical: 24 hours
  - Maximum: 9 months

### "Cloudflare/bot protection detected"

**This means Cloudflare is blocking the request despite cookies**:

**Solutions**:
1. Try extracting fresh cookies
2. Use a different IP address (VPN)
3. Use FlareSolverr (see CLOUDFLARE-BYPASS-GUIDE.md)
4. Use CloudScraper integration (coming soon)

## Manual Alternative

If you prefer manual configuration, you can edit config.ini directly:

```ini
[cookies]
www.javbus.com = dv=1,existmag=mag,PHPSESSID=YOUR_SESSION_VALUE_HERE
```

Then test with:
```bash
python3 test_cookies.py
```

## Cookie Lifespan

**How long do cookies last?**

- **dv cookie**: Usually lasts until browser close or 24 hours
- **existmag cookie**: Usually persistent (months)
- **PHPSESSID**: Varies by server configuration (30min - 9 months)

**When to refresh**:
- When you see 403 Forbidden errors
- When scrapers report "Cloudflare challenge"
- When test_cookies.py fails validation

## Security Notes

### Cookie Safety
- ✅ Cookies are authentication tokens - treat them like passwords
- ✅ Never share cookies publicly
- ✅ Never commit config.ini to git
- ✅ The script sets file permissions to 600 (owner read/write only)

### What Can Someone Do With Your Cookies?
- Access JAVBus as if they were you
- Browse content with your session
- **Cannot**: Access your computer, steal data, or do anything outside JAVBus

### Protecting Your Config
```bash
# Check permissions
ls -l ~/.mdc/config.ini
# Should show: -rw------- (600)

# Fix if needed
chmod 600 ~/.mdc/config.ini

# Never commit to git
echo "config.ini" >> .gitignore
```

## Related Documentation

- **test_cookies.py** - Test existing cookie configuration
- **QUICK-COOKIE-FIX.md** - Fast manual cookie extraction guide
- **CLOUDFLARE-BYPASS-GUIDE.md** - Advanced Cloudflare bypass options
- **config.ini.example** - Example configuration file

## FAQ

### Q: Do I need to do this for every scraper?

**A**: No, just JAVBus (and optionally JavDB, JAVLibrary). Tier 1 scrapers like DMM and R18Dev don't need cookies.

### Q: Can I generate random cookies instead of extracting them?

**A**: No. Cookies are issued by the server and validated against server-side session storage. Random cookies won't work.

### Q: Why can't the script extract cookies automatically?

**A**: Browser security prevents scripts from accessing cookies from other domains. You must extract them manually.

### Q: Will this work on Windows?

**A**: Yes! The script works on Windows, macOS, and Linux. The only difference is file permissions (Windows doesn't use chmod).

### Q: Can I use the same cookies on multiple machines?

**A**: Technically yes, but:
- IP address changes may invalidate session
- Cookies might be tied to user-agent
- Better to extract fresh cookies per machine

### Q: How often do I need to do this?

**A**: Depends on cookie lifespan:
- Daily scraping: Cookies usually last 24+ hours
- Weekly scraping: May need to refresh before each session
- Best practice: Test with `python3 test_cookies.py` before batch processing

### Q: What if I'm behind a VPN/proxy?

**A**:
- Extract cookies while using the same VPN/proxy
- JAVBus may tie cookies to IP address range
- If cookies don't work, try without VPN or use FlareSolverr

### Q: Is this legal?

**A**:
- Yes - you're extracting YOUR OWN cookies from YOUR browser
- This is equivalent to using your browser normally
- For personal use and local media organization only

## Success Indicators

After running `init_config.py`, you should be able to:

✅ Run `python3 test_cookies.py` and see:
```
✅ Working: 1/1
✅ Successfully authenticated:
   - www.javbus.com
🎉 All configured scrapers are working!
```

✅ Run scraper and see in logs:
```
[javbus] Using cookies for www.javbus.com (auto-detected)
Scrapers tried: 12
Scrapers succeeded: 1
Success Rate: 100%
```

✅ Process real files:
```bash
./target/release/mdc-cli /tmp/test-movies/SSIS-001.mp4 -m 3 -g
# Should succeed and create NFO file
```

## Next Steps

After successful configuration:

1. **Test your setup**:
   ```bash
   python3 test_cookies.py
   ```

2. **Process a single file**:
   ```bash
   ./target/release/mdc-cli /path/to/movie.mp4 -m 3 -g
   ```

3. **Process a directory**:
   ```bash
   ./target/release/mdc-cli /path/to/movies -m 1 -s
   ```

4. **Monitor for cookie expiration**:
   - If you see 403 errors, re-run `init_config.py`
   - Set a calendar reminder to refresh monthly

## Support

If you encounter issues:

1. Read the troubleshooting section above
2. Check CLOUDFLARE-BYPASS-GUIDE.md for advanced solutions
3. Run `python3 test_cookies.py` to diagnose
4. Enable debug mode: `./target/release/mdc-cli ... -g`
5. Report issues with full error messages

---

**Remember**: This is a 2-minute one-time setup. Once configured, cookies typically last days to months before needing refresh.
