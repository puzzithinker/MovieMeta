#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
CloudFlare bypass bridge using cloudscraper

This script is called from Rust to bypass CloudFlare protection.
It uses cloudscraper (a requests wrapper) to handle CF challenges.

Usage:
    python cloudflare_bridge.py <url> [options]

Options:
    --method GET|POST     HTTP method (default: GET)
    --timeout N           Timeout in seconds (default: 10)
    --proxy URL           Proxy URL
    --user-agent STR      User agent string
    --cookie KEY=VALUE    Add cookie (can be repeated)
    --header KEY:VALUE    Add header (can be repeated)
    --data KEY=VALUE      POST data (can be repeated)
    --output text|json|bytes  Output type (default: text)

Output:
    Prints response body to stdout on success
    Prints error to stderr on failure
    Exit code 0 on success, non-zero on failure
"""

import sys
import argparse
import json
from urllib.parse import urlparse

try:
    import cloudscraper
except ImportError:
    print("ERROR: cloudscraper not installed. Run: pip install cloudscraper", file=sys.stderr)
    sys.exit(1)


def main():
    parser = argparse.ArgumentParser(description='CloudFlare bypass bridge')
    parser.add_argument('url', help='URL to fetch')
    parser.add_argument('--method', default='GET', choices=['GET', 'POST'],
                        help='HTTP method')
    parser.add_argument('--timeout', type=int, default=10,
                        help='Timeout in seconds')
    parser.add_argument('--proxy', help='Proxy URL')
    parser.add_argument('--user-agent', default='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
                        help='User agent string')
    parser.add_argument('--cookie', action='append', default=[],
                        help='Cookie in KEY=VALUE format')
    parser.add_argument('--header', action='append', default=[],
                        help='Header in KEY:VALUE format')
    parser.add_argument('--data', action='append', default=[],
                        help='POST data in KEY=VALUE format')
    parser.add_argument('--output', default='text', choices=['text', 'json', 'bytes'],
                        help='Output format')
    parser.add_argument('--verify-ssl', action='store_true',
                        help='Verify SSL certificates (default: false)')

    args = parser.parse_args()

    # Create cloudscraper session
    scraper = cloudscraper.create_scraper(
        browser={'custom': args.user_agent}
    )

    # Parse cookies
    cookies = {}
    for cookie_str in args.cookie:
        if '=' in cookie_str:
            key, value = cookie_str.split('=', 1)
            cookies[key] = value

    if cookies:
        scraper.cookies.update(cookies)

    # Parse headers
    headers = {}
    for header_str in args.header:
        if ':' in header_str:
            key, value = header_str.split(':', 1)
            headers[key.strip()] = value.strip()

    if headers:
        scraper.headers.update(headers)

    # Setup proxy
    proxies = None
    if args.proxy:
        parsed = urlparse(args.proxy)
        proxies = {
            'http': args.proxy,
            'https': args.proxy,
        }

    # Verify SSL
    verify = args.verify_ssl

    try:
        # Make request
        if args.method == 'GET':
            response = scraper.get(
                args.url,
                timeout=args.timeout,
                proxies=proxies,
                verify=verify
            )
        else:  # POST
            # Parse POST data
            data = {}
            for data_str in args.data:
                if '=' in data_str:
                    key, value = data_str.split('=', 1)
                    data[key] = value

            response = scraper.post(
                args.url,
                data=data,
                timeout=args.timeout,
                proxies=proxies,
                verify=verify
            )

        # Check status
        response.raise_for_status()

        # Output response
        if args.output == 'text':
            print(response.text)
        elif args.output == 'json':
            # Output as JSON with status code and headers
            output = {
                'status': response.status_code,
                'headers': dict(response.headers),
                'body': response.text,
            }
            print(json.dumps(output))
        elif args.output == 'bytes':
            # Output raw bytes to stdout
            sys.stdout.buffer.write(response.content)

        return 0

    except cloudscraper.exceptions.CloudflareChallengeError as e:
        print(f"ERROR: CloudFlare challenge failed: {e}", file=sys.stderr)
        return 2
    except Exception as e:
        print(f"ERROR: Request failed: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
