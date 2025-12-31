#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Cookie Configuration Tester for MovieMeta

Tests if your configured cookies can successfully access JAV scraper sites.
This helps diagnose authentication issues before running full batch processing.

Usage:
    python3 test_cookies.py [config_file]

If no config file specified, searches standard locations:
  - ./config.ini
  - ~/mdc.ini
  - ~/.mdc.ini
  - ~/.mdc/config.ini
  - ~/.config/mdc/config.ini
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


class CookieTester:
    """Tests cookie configuration for JAV scraper sites"""

    # Test URLs for each domain
    TEST_URLS = {
        "javdb.com": "https://javdb.com/search?q=SSIS-001",
        "www.javbus.com": "https://www.javbus.com/SSIS-001",
        "javlibrary.com": "https://www.javlibrary.com/en/?v=javlissis001",
        "avmoo.com": "https://avmoo.com/en/search/SSIS-001",
        "mgstage.com": "https://www.mgstage.com/search/cSearch.php?search_word=SIRO-123",
    }

    # Success indicators (HTML content that proves access worked)
    SUCCESS_INDICATORS = {
        "javdb.com": ["search-result", "movie-list", "video-detail"],
        "www.javbus.com": ["movie", "info", "star"],
        "javlibrary.com": ["video_id", "video_title", "video_info"],
        "avmoo.com": ["movie", "info"],
        "mgstage.com": ["product_detail", "search_result"],
    }

    def __init__(self, config_path=None):
        """Initialize tester with config file"""
        self.config_path = config_path or self.find_config()
        self.cookies = {}
        self.results = {}

    def find_config(self):
        """Find config file in standard locations"""
        locations = [
            Path("./config.ini"),
            Path.home() / "mdc.ini",
            Path.home() / ".mdc.ini",
            Path.home() / ".mdc" / "config.ini",
            Path.home() / ".config" / "mdc" / "config.ini",
        ]

        for path in locations:
            if path.exists():
                return str(path)

        return None

    def load_cookies(self):
        """Load cookies from config file"""
        if not self.config_path:
            print("❌ No config file found in standard locations:")
            print("   - ./config.ini")
            print("   - ~/mdc.ini")
            print("   - ~/.mdc.ini")
            print("   - ~/.mdc/config.ini")
            print("   - ~/.config/mdc/config.ini")
            print()
            print("💡 Create one with: mkdir -p ~/.mdc && touch ~/.mdc/config.ini")
            return False

        if not Path(self.config_path).exists():
            print(f"❌ Config file not found: {self.config_path}")
            return False

        print(f"📂 Loading config from: {self.config_path}")
        print()

        config = configparser.ConfigParser()
        config.read(self.config_path)

        if "cookies" not in config:
            print("❌ No [cookies] section found in config file")
            print()
            print("💡 Add a [cookies] section like this:")
            print()
            print("   [cookies]")
            print("   javdb.com = _jdb_session=YOUR_SESSION_TOKEN")
            print("   javbus.com = cf_clearance=YOUR_CF_TOKEN")
            print()
            return False

        # Parse cookies
        for domain, cookie_str in config["cookies"].items():
            cookie_dict = {}
            for pair in cookie_str.split(","):
                pair = pair.strip()
                if "=" in pair:
                    name, value = pair.split("=", 1)
                    cookie_dict[name.strip()] = value.strip()

            if cookie_dict:
                self.cookies[domain] = cookie_dict

        if not self.cookies:
            print("❌ No valid cookies found in [cookies] section")
            return False

        print(f"✅ Loaded cookies for {len(self.cookies)} domain(s):")
        for domain, cookies in self.cookies.items():
            cookie_names = ", ".join(cookies.keys())
            print(f"   - {domain}: {cookie_names}")
        print()

        return True

    def test_domain(self, domain):
        """Test if cookies work for a specific domain"""
        if domain not in self.cookies:
            return {
                "status": "skipped",
                "reason": "No cookies configured",
            }

        if domain not in self.TEST_URLS:
            return {
                "status": "skipped",
                "reason": "No test URL defined",
            }

        url = self.TEST_URLS[domain]
        cookies = self.cookies[domain]

        print(f"🔍 Testing {domain}...")
        print(f"   URL: {url}")
        print(f"   Cookies: {', '.join(cookies.keys())}")

        try:
            # Try with cloudscraper first
            scraper = cloudscraper.create_scraper()
            scraper.cookies.update(cookies)

            start_time = time.time()
            response = scraper.get(url, timeout=15, verify=True)
            elapsed = time.time() - start_time

            status_code = response.status_code
            content_length = len(response.text)

            # Check for success indicators
            success_found = False
            if domain in self.SUCCESS_INDICATORS:
                for indicator in self.SUCCESS_INDICATORS[domain]:
                    if indicator in response.text.lower():
                        success_found = True
                        break

            # Check for common error patterns
            is_blocked = any(
                pattern in response.text.lower()
                for pattern in [
                    "cloudflare",
                    "just a moment",
                    "access denied",
                    "forbidden",
                    "ray id",
                ]
            )

            if status_code == 200 and success_found:
                print(f"   ✅ SUCCESS ({elapsed:.1f}s, {content_length} bytes)")
                return {
                    "status": "success",
                    "status_code": status_code,
                    "elapsed": elapsed,
                    "content_length": content_length,
                }
            elif status_code == 200 and is_blocked:
                print(f"   ⚠️  Cloudflare/Bot protection detected")
                return {
                    "status": "blocked",
                    "status_code": status_code,
                    "reason": "Cloudflare or bot protection active",
                }
            elif status_code == 200:
                print(f"   ⚠️  Page loaded but content unexpected")
                return {
                    "status": "warning",
                    "status_code": status_code,
                    "reason": "Page structure may have changed",
                }
            elif status_code == 403:
                print(f"   ❌ FORBIDDEN (403)")
                return {
                    "status": "forbidden",
                    "status_code": 403,
                    "reason": "Cookies rejected or expired",
                }
            elif status_code == 404:
                print(f"   ⚠️  NOT FOUND (404)")
                return {
                    "status": "not_found",
                    "status_code": 404,
                    "reason": "Test URL may be invalid",
                }
            else:
                print(f"   ❌ HTTP {status_code}")
                return {
                    "status": "error",
                    "status_code": status_code,
                    "reason": f"HTTP {status_code}",
                }

        except requests.exceptions.Timeout:
            print(f"   ❌ TIMEOUT (>15s)")
            return {
                "status": "timeout",
                "reason": "Request timed out after 15 seconds",
            }
        except requests.exceptions.SSLError as e:
            print(f"   ❌ SSL ERROR")
            return {
                "status": "ssl_error",
                "reason": str(e),
            }
        except Exception as e:
            print(f"   ❌ ERROR: {str(e)[:100]}")
            return {
                "status": "error",
                "reason": str(e)[:200],
            }

    def run_tests(self):
        """Run tests for all configured domains"""
        print("=" * 70)
        print("MovieMeta Cookie Configuration Tester")
        print("=" * 70)
        print()

        # Load cookies
        if not self.load_cookies():
            return False

        # Test each configured domain
        print("=" * 70)
        print("Testing Cookie Authentication")
        print("=" * 70)
        print()

        for domain in sorted(self.cookies.keys()):
            result = self.test_domain(domain)
            self.results[domain] = result
            print()
            time.sleep(1)  # Be nice to servers

        # Print summary
        self.print_summary()

        # Return success if at least one domain works
        return any(r["status"] == "success" for r in self.results.values())

    def print_summary(self):
        """Print test results summary"""
        print("=" * 70)
        print("Summary")
        print("=" * 70)
        print()

        success_count = sum(1 for r in self.results.values() if r["status"] == "success")
        total_count = len(self.results)

        print(f"✅ Working: {success_count}/{total_count}")
        print()

        # Group results by status
        by_status = {}
        for domain, result in self.results.items():
            status = result["status"]
            if status not in by_status:
                by_status[status] = []
            by_status[status].append(domain)

        # Print each status group
        if "success" in by_status:
            print("✅ Successfully authenticated:")
            for domain in by_status["success"]:
                print(f"   - {domain}")
            print()

        if "forbidden" in by_status:
            print("❌ Cookies rejected (403 Forbidden):")
            for domain in by_status["forbidden"]:
                print(f"   - {domain}")
            print("   💡 Re-extract cookies from browser (they may have expired)")
            print()

        if "blocked" in by_status:
            print("⚠️  Cloudflare/Bot protection detected:")
            for domain in by_status["blocked"]:
                print(f"   - {domain}")
            print("   💡 Try extracting fresh cookies or use CloudScraper backend")
            print()

        if "timeout" in by_status:
            print("⏱️  Connection timeouts:")
            for domain in by_status["timeout"]:
                print(f"   - {domain}")
            print("   💡 Check internet connection or try again later")
            print()

        if "skipped" in by_status:
            print("⏭️  Skipped (no cookies configured):")
            for domain in by_status["skipped"]:
                print(f"   - {domain}")
            print()

        # Recommendations
        print("=" * 70)
        print("Recommendations")
        print("=" * 70)
        print()

        if success_count == 0:
            print("❌ No scrapers are working with current cookies!")
            print()
            print("Next steps:")
            print("1. Extract fresh cookies from your browser")
            print("2. Update ~/.mdc/config.ini with new cookie values")
            print("3. Run this test script again")
            print()
            print("See: COOKIE-CONFIGURATION.md for detailed instructions")
        elif success_count < total_count:
            print("⚠️  Some scrapers are working, others need attention")
            print()
            print("For scrapers with 403 errors:")
            print("- Extract fresh cookies (they may have expired)")
            print("- Ensure you copied the complete cookie value")
            print("- Check that domain name matches exactly")
        else:
            print("🎉 All configured scrapers are working!")
            print()
            print("You're ready to process files:")
            print("  ./mdc-cli /path/to/movies -s")

        print()


def main():
    """Main entry point"""
    config_file = sys.argv[1] if len(sys.argv) > 1 else None

    tester = CookieTester(config_file)
    success = tester.run_tests()

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
