#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Interactive Cookie Extraction Guide for MovieMeta

This script guides you through extracting cookies from your browser
and adding them to the config file.
"""

import os
import sys
from pathlib import Path


def print_header(text):
    """Print formatted header"""
    print("\n" + "=" * 70)
    print(f"  {text}")
    print("=" * 70 + "\n")


def print_step(number, text):
    """Print step number"""
    print(f"\n🔹 Step {number}: {text}")
    print("-" * 70)


def wait_for_enter():
    """Wait for user to press Enter"""
    input("\nPress ENTER to continue...")


def main():
    """Main interactive guide"""
    print_header("🍪 Cookie Extraction Guide for MovieMeta")

    print("""
This guide will help you extract cookies from your browser to enable
MovieMeta scrapers to bypass Cloudflare protection.

Why do you need cookies?
- Most JAV sites now use Cloudflare to block automated requests
- Cookies from a successful browser session allow scrapers to work
- This is the same approach used by Javinizer

We'll extract cookies for:
  1. www.javbus.com (most important - highest coverage)
  2. www.javlibrary.com (comprehensive database)
  3. javdb.com (optional - for FC2 content)
""")

    wait_for_enter()

    # =========================================================================
    # JAVBUS
    # =========================================================================
    print_header("🎯 Priority #1: JAVBus Cookies")

    print_step(1, "Open your browser (Chrome or Firefox recommended)")
    print("We'll be using Chrome DevTools for this example.")
    wait_for_enter()

    print_step(2, "Visit www.javbus.com")
    print("Open this URL in your browser:")
    print("  🌐 https://www.javbus.com")
    print("\nYou may see a Cloudflare 'Checking your browser' message.")
    print("Wait for it to complete (usually 5-10 seconds).")
    wait_for_enter()

    print_step(3, "Open DevTools")
    print("Method 1: Press F12")
    print("Method 2: Right-click anywhere → 'Inspect'")
    print("Method 3: Menu → More Tools → Developer Tools")
    print("\nThe DevTools panel will open at the bottom or side of your browser.")
    wait_for_enter()

    print_step(4, "Navigate to Cookies")
    print("In DevTools:")
    print("  • Click the 'Application' tab (Chrome)")
    print("  • Or 'Storage' tab (Firefox)")
    print("  • In the left sidebar, expand 'Cookies'")
    print("  • Click on 'https://www.javbus.com'")
    print("\nYou should see a table with cookie names and values.")
    wait_for_enter()

    print_step(5, "Find the cf_clearance cookie")
    print("In the cookies table, look for:")
    print("  📍 Name: cf_clearance")
    print("  📍 Value: A long string like 'abc123def456...'")
    print("\nThis is the Cloudflare bypass token!")
    print("\nClick on this row to select it.")
    wait_for_enter()

    print_step(6, "Copy the cookie value")
    print("In the cookies table:")
    print("  1. Click on the 'cf_clearance' row")
    print("  2. Find the 'Value' column")
    print("  3. Double-click the value to select it")
    print("  4. Press Ctrl+C (or Cmd+C on Mac) to copy")
    print("\n⚠️  Copy the ENTIRE value - it's long!")
    print("⚠️  It usually starts with letters/numbers and contains dots or hyphens")
    wait_for_enter()

    print_step(7, "Also copy PHPSESSID (optional but helpful)")
    print("Find the 'PHPSESSID' cookie and copy its value the same way.")
    wait_for_enter()

    print_step(8, "Update your config file")
    config_path = Path.home() / "code/Movie_Data_Capture/rust/config.ini"
    print(f"Open your config file:")
    print(f"  📝 {config_path}")
    print("\nFind the [cookies] section and UPDATE this line:")
    print("\n  # OLD:")
    print("  www.javbus.com = PHPSESSID=4ufh6tjmkdq8th3a1cu46v5f14")
    print("\n  # NEW (paste your real values):")
    print("  www.javbus.com = cf_clearance=PASTE_YOUR_CF_TOKEN_HERE,PHPSESSID=PASTE_SESSION_HERE")
    print("\n⚠️  Important:")
    print("  • Remove any spaces around the = signs")
    print("  • Separate multiple cookies with commas (no spaces)")
    print("  • Make sure domain is exactly 'www.javbus.com' (with www)")
    wait_for_enter()

    # =========================================================================
    # JAVLIBRARY
    # =========================================================================
    print_header("🎯 Priority #2: JAVLibrary Cookies (Optional)")

    response = input("Do you want to extract JAVLibrary cookies too? (y/n): ")
    if response.lower() == 'y':
        print("\nFollow the same steps as JAVBus:")
        print("  1. Visit https://www.javlibrary.com/en/")
        print("  2. Complete any age verification")
        print("  3. Open DevTools → Application → Cookies")
        print("  4. Copy 'cf_clearance' and any session cookies")
        print("  5. Add to config.ini:")
        print("\n     www.javlibrary.com = cf_clearance=TOKEN,over18=1")
        wait_for_enter()

    # =========================================================================
    # JAVDB
    # =========================================================================
    print_header("🎯 Priority #3: JavDB Cookies (Optional)")

    response = input("Do you want to extract JavDB cookies too? (y/n): ")
    if response.lower() == 'y':
        print("\nJavDB is easier - just one cookie:")
        print("  1. Visit https://javdb.com")
        print("  2. Complete age verification")
        print("  3. Open DevTools → Application → Cookies")
        print("  4. Find '_jdb_session' cookie")
        print("  5. Copy its value")
        print("  6. Add to config.ini:")
        print("\n     javdb.com = _jdb_session=YOUR_SESSION_TOKEN")
        wait_for_enter()

    # =========================================================================
    # TESTING
    # =========================================================================
    print_header("✅ Testing Your Configuration")

    print("After updating your config.ini, test it:")
    print("\n  cd ~/code/Movie_Data_Capture/rust")
    print("  python3 test_cookies.py")
    print("\nExpected output:")
    print("  ✅ Loaded cookies for domains: www.javbus.com")
    print("  🌐 Testing www.javbus.com...")
    print("  ✅ SUCCESS - Page loaded without Cloudflare block")
    print("\nThen test real scraping:")
    print("  ./target/release/mdc-cli /tmp/test-movies/SNIS-091.mp4 -m 3")
    print("\nSuccess rate should jump from 0% → 60-70%! 🎉")

    # =========================================================================
    # TROUBLESHOOTING
    # =========================================================================
    print_header("🔧 Common Issues")

    print("""
❌ "Still getting Cloudflare errors"
   → Your cf_clearance cookie expired (lasts ~30 minutes)
   → Extract fresh cookies again

❌ "Cookies not being used"
   → Check domain matches exactly: 'www.javbus.com' not 'javbus.com'
   → Enable debug: ./target/release/mdc-cli FILE -m 3 -g
   → Look for: "[javbus] Using cookies for www.javbus.com"

❌ "Config parse error"
   → Remove spaces: 'cookie=value' not 'cookie = value'
   → Check commas: 'c1=v1,c2=v2' not 'c1=v1, c2=v2'

❌ "Page still returns 404/403"
   → Content might not exist on that site
   → Try different scraper sources
   → Some older titles are delisted
""")

    print_header("📚 Additional Resources")

    print("""
Detailed guides in the rust/ directory:
  • COOKIE-TESTER-README.md - Full testing guide (380 lines)
  • QUICK-COOKIE-FIX.md - Quick reference (100 lines)
  • TROUBLESHOOTING.md - Problem solving

Need help?
  • Check logs with -g flag for debug info
  • Verify cookies with test_cookies.py
  • Refer to Javinizer docs (they use same approach)
""")

    print("\n" + "=" * 70)
    print("  🎉 Cookie extraction guide complete!")
    print("  📝 Update your config.ini and test with test_cookies.py")
    print("=" * 70 + "\n")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted by user. Exiting...")
        sys.exit(0)
