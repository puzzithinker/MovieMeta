# Cookie Setup - Quick Start

## TL;DR

Run this to set up JAVBus cookies in 2 minutes:

```bash
cd ~/code/Movie_Data_Capture/rust
python3 init_config.py
```

Follow the prompts, extract cookies from your browser, and you're done!

## What This Does

The interactive helper will:
1. ✅ Check if cookies are already configured
2. 📖 Guide you through extracting cookies from your browser
3. ✅ Validate cookies against JAVBus server
4. 💾 Create/update config.ini automatically
5. 🎉 Confirm everything is working

## Three Required Cookies

You need to extract these three cookies from your browser after visiting https://www.javbus.com:

| Cookie      | Value                  | Purpose                    |
|-------------|------------------------|----------------------------|
| dv          | `1`                    | Age verification flag      |
| existmag    | `mag`                  | Content access permission  |
| PHPSESSID   | `5ti9138...` (random)  | Session identifier         |

## The Process

### Step 1: Visit JAVBus
```
1. Open browser
2. Go to https://www.javbus.com
3. Click "Enter" for age verification
4. Wait for page to load
```

### Step 2: Open DevTools
```
Chrome/Edge:
  F12 → Application → Cookies → www.javbus.com

Firefox:
  F12 → Storage → Cookies → www.javbus.com
```

### Step 3: Run Helper
```bash
python3 init_config.py
# Follow prompts and enter cookie values
```

### Step 4: Done!
```bash
# Test it works
python3 test_cookies.py

# Use it
./target/release/mdc-cli /path/to/movies -m 1 -s
```

## Documentation

- **init_config.py** - Interactive setup tool (this is what you run)
- **COOKIE-INIT-GUIDE.md** - Detailed guide with screenshots and troubleshooting
- **COOKIE-IMPLEMENTATION-SUMMARY.md** - Technical details and architecture
- **test_cookies.py** - Test your configuration anytime
- **CLOUDFLARE-BYPASS-GUIDE.md** - Advanced bypass options

## Quick Commands

```bash
# Set up cookies for the first time
python3 init_config.py

# Test existing cookies
python3 test_cookies.py

# Refresh expired cookies
python3 init_config.py

# Process files with cookies
./target/release/mdc-cli /path/to/movies -m 1 -s
```

## Troubleshooting

### "Can't find dv cookie"
→ Make sure you clicked "Enter" on the age verification page first

### "Can't find PHPSESSID"
→ Scroll down in the cookies list, it's alphabetically sorted

### "Cookie validation failed"
→ Make sure you copied the COMPLETE value (especially PHPSESSID)

### "Cloudflare protection detected"
→ Try refreshing the cookies or see CLOUDFLARE-BYPASS-GUIDE.md

## Why Manual Extraction?

**Q**: Why can't this be automated?

**A**: Cookies are issued by JAVBus server and validated against server-side session storage. There's no way to generate valid cookies without actually visiting the site in a browser. We researched automatic options (CloudScraper, FlareSolverr) but manual extraction is:
- Faster (2 minutes)
- More reliable
- No external dependencies
- Works for everyone

Future versions may add optional automation for power users.

## Cookie Lifespan

Cookies typically last:
- **Minimum**: 30 minutes
- **Typical**: 24 hours
- **Maximum**: 9 months

Re-run `init_config.py` if you see 403 errors or authentication failures.

## Security

- Cookies are authentication tokens (treat like passwords)
- Never commit config.ini to git
- Never share cookies publicly
- The script sets secure file permissions automatically

Cookies can only access JAVBus - they cannot access your computer or steal other data.

## What Gets Created

```
~/.mdc/config.ini
```

With contents:
```ini
[cookies]
www.javbus.com = dv=1,existmag=mag,PHPSESSID=YOUR_SESSION_HERE
```

## Next Steps

After setup:

1. Test: `python3 test_cookies.py`
2. Process: `./target/release/mdc-cli /path/to/movies -m 1 -s`
3. Enjoy 100% success rate! 🎉

## Support

If you have issues:
1. Read COOKIE-INIT-GUIDE.md troubleshooting section
2. Check CLOUDFLARE-BYPASS-GUIDE.md for advanced options
3. Run `python3 test_cookies.py` to diagnose
4. Enable debug: `./target/release/mdc-cli ... -g`

---

**Remember**: This is a one-time 2-minute setup. Once configured, cookies last for days to months before needing refresh.
