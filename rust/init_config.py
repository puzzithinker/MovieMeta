#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
MovieMeta Configuration Initializer - JAVBus Cookie Setup

Guides users through extracting browser cookies and initializing config.ini
with working authentication for JAVBus scraper.

Usage:
    python3 init_config.py

This script will:
1. Check current configuration state
2. Guide you through cookie extraction from browser (2 minutes)
3. Validate cookies against JAVBus server
4. Initialize config.ini with working values
5. Provide troubleshooting if validation fails
"""

import sys
import os
from pathlib import Path
import configparser
import time

try:
    import cloudscraper
except ImportError:
    print("❌ ERROR: cloudscraper not installed")
    print("   Install: pip3 install cloudscraper")
    sys.exit(1)

try:
    import requests
except ImportError:
    print("❌ ERROR: requests not installed")
    print("   Install: pip3 install requests")
    sys.exit(1)


class ConfigInitializer:
    """Interactive config initializer for JAVBus cookies"""

    # Standard config locations (in priority order)
    CONFIG_LOCATIONS = [
        Path("./config.ini"),
        Path.home() / ".mdc" / "config.ini",
        Path.home() / ".config" / "mdc" / "config.ini",
        Path.home() / "mdc.ini",
        Path.home() / ".mdc.ini",
    ]

    # Test URL and success indicators
    TEST_URL = "https://www.javbus.com/SSIS-001"
    SUCCESS_INDICATORS = ["movie", "info", "star", "genre"]
    BLOCK_INDICATORS = ["cloudflare", "just a moment", "access denied", "forbidden"]

    def __init__(self):
        """Initialize the config initializer"""
        self.config_path = None
        self.config = None
        self.cookies_valid = False
        self.cookies_expired = False

    def print_welcome(self):
        """Print welcome banner"""
        print()
        print("╔" + "═" * 68 + "╗")
        print("║" + " " * 20 + "MovieMeta Configuration Initializer" + " " * 13 + "║")
        print("║" + " " * 24 + "JAVBus Cookie Setup" + " " * 25 + "║")
        print("╚" + "═" * 68 + "╝")
        print()

    def find_config(self):
        """Find existing config file or determine where to create it"""
        print("Checking current configuration...")
        print()

        # Check if any config exists
        for path in self.CONFIG_LOCATIONS:
            if path.exists():
                self.config_path = path
                print(f"✅ Found existing config: {path}")
                return True

        # No config found - recommend location
        recommended = self.CONFIG_LOCATIONS[1]  # ~/.mdc/config.ini
        print(f"❌ No config.ini found in standard locations")
        print(f"💡 Will create: {recommended}")
        self.config_path = recommended
        return False

    def load_config(self):
        """Load existing config if available"""
        if not self.config_path.exists():
            return False

        self.config = configparser.ConfigParser()
        try:
            self.config.read(self.config_path)
            return True
        except Exception as e:
            print(f"⚠️  Warning: Could not parse config: {e}")
            return False

    def check_cookies_status(self):
        """Check if cookies are configured and validate them"""
        if not self.config or "cookies" not in self.config:
            print("❌ No [cookies] section found")
            return False

        # Check for javbus cookies
        javbus_domains = ["www.javbus.com", "javbus.com", "javbus"]
        javbus_cookies = None
        javbus_domain = None

        for domain in javbus_domains:
            if domain in self.config["cookies"]:
                javbus_cookies = self.config["cookies"][domain]
                javbus_domain = domain
                break

        if not javbus_cookies:
            print("❌ No JAVBus cookies configured")
            return False

        print(f"✅ Found JAVBus cookies for: {javbus_domain}")

        # Parse cookies
        cookie_dict = self.parse_cookie_string(javbus_cookies)
        if not cookie_dict:
            print("⚠️  Cookies are empty or invalid format")
            return False

        print(f"   Cookies: {', '.join(cookie_dict.keys())}")
        print()
        print("🌐 Validating cookies against JAVBus server...")

        # Test cookies
        is_valid = self.validate_cookies(cookie_dict)

        if is_valid:
            print("✅ Cookies are valid and working!")
            self.cookies_valid = True
            return True
        else:
            print("❌ Cookies are configured but not working (may be expired)")
            self.cookies_expired = True
            return False

    def parse_cookie_string(self, cookie_str):
        """Parse cookie string into dictionary"""
        cookie_dict = {}
        for pair in cookie_str.split(","):
            pair = pair.strip()
            if "=" in pair:
                name, value = pair.split("=", 1)
                cookie_dict[name.strip()] = value.strip()
        return cookie_dict

    def validate_cookies(self, cookie_dict):
        """Validate cookies against JAVBus server"""
        try:
            scraper = cloudscraper.create_scraper()
            scraper.cookies.update(cookie_dict)

            response = scraper.get(self.TEST_URL, timeout=15, verify=True)

            # Check status code
            if response.status_code != 200:
                print(f"   ❌ HTTP {response.status_code}")
                return False

            # Check for success indicators
            content_lower = response.text.lower()
            success_found = any(indicator in content_lower for indicator in self.SUCCESS_INDICATORS)

            # Check for block indicators
            is_blocked = any(indicator in content_lower for indicator in self.BLOCK_INDICATORS)

            if success_found and not is_blocked:
                print(f"   ✅ Successfully retrieved test page")
                return True
            elif is_blocked:
                print(f"   ❌ Cloudflare/bot protection detected")
                return False
            else:
                print(f"   ⚠️  Page loaded but content unexpected")
                return False

        except requests.exceptions.Timeout:
            print(f"   ❌ Request timed out")
            return False
        except Exception as e:
            print(f"   ❌ Error: {str(e)[:100]}")
            return False

    def guide_cookie_extraction(self):
        """Guide user through cookie extraction process"""
        print()
        print("Let's set up your JAVBus cookies! This will take ~2 minutes.")
        print()
        print("=" * 70)
        print("Step 1: Open JAVBus in your browser")
        print("=" * 70)
        print()
        print("1. Open Chrome, Firefox, or any modern browser")
        print("2. Go to: https://www.javbus.com")
        print("3. Complete the age verification (click \"Enter\" button)")
        print("4. Wait for the page to fully load")
        print()
        input("Press ENTER when you've completed age verification...")
        print()

        print("=" * 70)
        print("Step 2: Extract cookies from browser")
        print("=" * 70)
        print()
        print("Now we need to extract cookies from your browser:")
        print()
        print("For Chrome/Edge:")
        print("  1. Press F12 to open DevTools")
        print("  2. Click 'Application' tab at the top")
        print("  3. Expand 'Cookies' in the left sidebar")
        print("  4. Click on 'https://www.javbus.com'")
        print()
        print("For Firefox:")
        print("  1. Press F12 to open DevTools")
        print("  2. Click 'Storage' tab at the top")
        print("  3. Expand 'Cookies' in the left sidebar")
        print("  4. Click on 'https://www.javbus.com'")
        print()
        print("You should see a table with cookie names and values.")
        print()
        print("Look for these THREE cookies (you need all three):")
        print("  • dv          (value is usually '1')")
        print("  • existmag    (value is usually 'mag')")
        print("  • PHPSESSID   (long random string like '5ti9138au9ih2d3gdirp60gdo1')")
        print()
        input("Press ENTER when you can see the cookies...")
        print()

    def prompt_for_cookies(self):
        """Prompt user to enter cookie values"""
        print("=" * 70)
        print("Step 3: Enter cookie values")
        print("=" * 70)
        print()
        print("For each cookie, find it in the DevTools table and copy its VALUE.")
        print()

        # Prompt for dv
        print("Cookie 1: dv")
        print("  Default value: 1")
        print("  Copy the VALUE from the 'Value' column in DevTools")
        dv = input("  Enter dv value [1]: ").strip()
        if not dv:
            dv = "1"
        print()

        # Prompt for existmag
        print("Cookie 2: existmag")
        print("  Default value: mag")
        print("  Copy the VALUE from the 'Value' column in DevTools")
        existmag = input("  Enter existmag value [mag]: ").strip()
        if not existmag:
            existmag = "mag"
        print()

        # Prompt for PHPSESSID
        print("Cookie 3: PHPSESSID")
        print("  This is a LONG random string (20+ characters)")
        print("  Example: 5ti9138au9ih2d3gdirp60gdo1")
        print("  Copy the COMPLETE VALUE from DevTools")
        while True:
            phpsessid = input("  Enter PHPSESSID value: ").strip()
            if phpsessid:
                if len(phpsessid) < 10:
                    print()
                    print("  ⚠️  Warning: PHPSESSID seems too short. Make sure you copied the complete value.")
                    retry = input("  Continue anyway? [y/N]: ").strip().lower()
                    if retry == 'y':
                        break
                else:
                    break
            else:
                print("  ❌ PHPSESSID is required! Please enter a value.")
        print()

        return {
            "dv": dv,
            "existmag": existmag,
            "PHPSESSID": phpsessid,
        }

    def test_and_validate(self, cookies):
        """Test cookies against server and validate"""
        print("=" * 70)
        print("Step 4: Validating cookies")
        print("=" * 70)
        print()
        print("🌐 Testing connection to JAVBus...")
        print(f"   URL: {self.TEST_URL}")
        print(f"   Cookies: {', '.join(cookies.keys())}")
        print()

        is_valid = self.validate_cookies(cookies)

        if is_valid:
            print()
            print("✅ Cookies are valid and working!")
            return True
        else:
            print()
            print("❌ Cookie validation failed")
            return False

    def troubleshoot(self):
        """Provide troubleshooting guidance"""
        print()
        print("=" * 70)
        print("Troubleshooting")
        print("=" * 70)
        print()
        print("Cookie validation failed. Common issues:")
        print()
        print("1. PHPSESSID incomplete:")
        print("   - Make sure you copied the ENTIRE value from DevTools")
        print("   - It should be 20+ characters long")
        print()
        print("2. Cookies from wrong domain:")
        print("   - Must extract from 'https://www.javbus.com' (note the 'www')")
        print("   - NOT from 'javbus.com' without 'www'")
        print()
        print("3. Age verification not completed:")
        print("   - You must click the 'Enter' button on JAVBus first")
        print("   - The 'dv=1' cookie is only set AFTER verification")
        print()
        print("4. Cookies expired during extraction:")
        print("   - Try the process again from the beginning")
        print("   - Work quickly once you start extracting")
        print()
        print("Would you like to try again? [y/N]: ", end='')
        retry = input().strip().lower()
        return retry == 'y'

    def initialize_config(self, cookies):
        """Initialize config.ini with validated cookies"""
        print()
        print("=" * 70)
        print("Step 5: Writing configuration")
        print("=" * 70)
        print()

        # Ensure parent directory exists
        self.config_path.parent.mkdir(parents=True, exist_ok=True)

        # Create or update config
        if not self.config:
            self.config = configparser.ConfigParser()

        # Add cookies section if missing
        if "cookies" not in self.config:
            self.config.add_section("cookies")

        # Format cookie string
        cookie_str = f"dv={cookies['dv']},existmag={cookies['existmag']},PHPSESSID={cookies['PHPSESSID']}"

        # Set cookie
        self.config["cookies"]["www.javbus.com"] = cookie_str

        # Write to file
        try:
            with open(self.config_path, 'w') as f:
                # Write header comment
                f.write("# MovieMeta Configuration File\n")
                f.write("#\n")
                f.write("# This file can be placed in one of the following locations:\n")
                f.write("#  1. ./config.ini (current directory)\n")
                f.write("#  2. ~/mdc.ini\n")
                f.write("#  3. ~/.mdc.ini\n")
                f.write("#  4. ~/.mdc/config.ini\n")
                f.write("#  5. ~/.config/mdc/config.ini\n")
                f.write("#\n")
                f.write("# The first file found will be used.\n")
                f.write("\n")

                # Write config sections
                self.config.write(f)

            print(f"📝 Created: {self.config_path}")
            print(f"✅ JAVBus cookies configured")

            # Set secure permissions (Unix only)
            if os.name != 'nt':
                os.chmod(self.config_path, 0o600)
                print(f"🔒 Set secure permissions (600)")

            return True

        except Exception as e:
            print(f"❌ Failed to write config: {e}")
            return False

    def print_success(self):
        """Print success message and next steps"""
        print()
        print("=" * 70)
        print("Configuration Complete!")
        print("=" * 70)
        print()
        print("🎉 Your JAVBus cookies are configured and working!")
        print()
        print("You can now run MovieMeta to scrape metadata:")
        print()
        print("  cd /home/simon/code/Movie_Data_Capture/rust")
        print("  ./target/release/mdc-cli /path/to/movies -m 1 -s")
        print()
        print("Or test with a specific file:")
        print("  ./target/release/mdc-cli /path/to/SSIS-001.mp4 -m 3 -g")
        print()
        print("📝 Configuration file: " + str(self.config_path))
        print()
        print("Note: Cookies typically last 24 hours to 9 months.")
        print("Re-run this script if you see authentication errors.")
        print()
        print("For testing your cookies anytime, run:")
        print("  python3 test_cookies.py")
        print()

    def run(self):
        """Main interactive flow"""
        self.print_welcome()

        # Check current state
        config_exists = self.find_config()
        print()

        if config_exists:
            self.load_config()
            if self.check_cookies_status():
                print()
                print("✅ Your configuration is already working!")
                print()
                print("Nothing to do. You're ready to process files:")
                print(f"  ./target/release/mdc-cli /path/to/movies -m 1 -s")
                print()
                return True

            if self.cookies_expired:
                print()
                print("⚠️  Cookies are configured but expired")
                print("    Let's refresh them...")

        # Guide through cookie extraction
        while True:
            self.guide_cookie_extraction()
            cookies = self.prompt_for_cookies()

            # Validate cookies
            if self.test_and_validate(cookies):
                # Initialize config
                if self.initialize_config(cookies):
                    self.print_success()
                    return True
                else:
                    return False
            else:
                # Troubleshoot
                if not self.troubleshoot():
                    print()
                    print("Configuration cancelled.")
                    print()
                    print("For help, see:")
                    print("  - QUICK-COOKIE-FIX.md")
                    print("  - CLOUDFLARE-BYPASS-GUIDE.md")
                    print()
                    return False


def main():
    """Main entry point"""
    try:
        initializer = ConfigInitializer()
        success = initializer.run()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print()
        print()
        print("Configuration cancelled by user.")
        print()
        sys.exit(1)
    except Exception as e:
        print()
        print(f"❌ Unexpected error: {e}")
        print()
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
