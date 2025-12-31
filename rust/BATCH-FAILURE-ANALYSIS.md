# Batch Processing Failure Analysis

## Overall Results
```
Total:     93
Succeeded: 67 ✓ (72%)
Failed:    26 ✗ (28%)
Skipped:   0
```

## Failure Categories

### Category 1: Multi-Disc Releases (8 failures)
**Issue**: Movies split into multiple parts with A/B/C suffixes

Examples:
- `AVOP-212A.HD.mp4` → Searched for AVOP-212A (not found)
- `AVOP-212B.HD.mp4` → Searched for AVOP-212B (not found)
- `RCT-642A.mp4` → Searched for RCT-642A (not found)
- `RCT-642B.mp4` → Searched for RCT-642B (not found)
- `DVAJ00452A.mp4` → Searched for DVAJ00452A (not found)
- `MIRD-129C.AVI` → Searched for MIRD-129C (not found)
- `DV888A.HQ.wmv` → Parsing error (HQ suffix + A suffix)
- `DV888B.HQ.wmv` → Parsing error (HQ suffix + B suffix)

**Why it failed**:
- Scrapers only have metadata for base number (e.g., AVOP-212)
- Our parser includes the A/B/C suffix in the search
- No scraper has entries for disc-specific IDs

**Solution Options**:

1. **Strip A/B/C suffix before searching** (recommended)
   - Parser detects A/B/C suffix at end
   - Searches for base number (AVOP-212)
   - Both files get same metadata
   - Add disc number to filename template

2. **Manual override** (current workaround)
   ```bash
   # For AVOP-212A.mp4
   ./mdc-cli /path/to/AVOP-212A.mp4 -n AVOP-212 -m 3
   ```

3. **Rename files** (simplest)
   ```
   AVOP-212A.HD.mp4 → AVOP-212-CD1.mp4
   AVOP-212B.HD.mp4 → AVOP-212-CD2.mp4
   ```

### Category 2: Not in Database (12 failures)
**Issue**: Titles legitimately not found in any scraper database

Examples:
- `TT022` - Unknown studio/series
- `TYOD124` - Checked 12 scrapers, none have it
- `APAE054` - Not in JAVBus, JAVLibrary, etc.
- `BEB077` - Not found
- `HNDB036` - Not found
- `UUE29` - Unknown series
- `UUV79` - Unknown series
- `FDD-1212` - Not in databases
- `KV-082` - Not found

**Why it failed**:
- Older/rare titles not in scraper databases
- Small studio releases
- Amateur/indie content
- Possibly incorrect number extraction

**Solutions**:

1. **Try alternative number parsing**
   - Sometimes filename has wrong format
   - Check actual DVD case/cover for correct ID

2. **Manual metadata creation**
   - Create NFO file manually
   - Use template from a successful scrape

3. **Try TMDB/IMDB scrapers**
   - Enable general movie databases
   - May have some titles under different names

4. **Accept limitation**
   - Not all JAV titles are indexed
   - Estimated coverage: 60-80% of all releases

### Category 3: Malformed Filenames (6 failures)
**Issue**: Filenames that don't match any standard DVD ID format

Examples:
```
加入微信號13655544看妹不用等.avi
  → "Join WeChat 13655544 to see girls no waiting" (spam)

ri-ben-mei-zhi-bo-zuo-ai.mp4
ri-ben-mei-zhi-bo-zuo-ai-2.mp4
  → "Japanese beauty live sex" (descriptive pinyin, not a DVD ID)

soe00753mhb.wmv
  → SOE-753 + "mhb" suffix (extra garbage)

Tokyo_Hot_th101-060-111077.mp4
  → Multiple hyphens, non-standard format

MKBD-S137-H265-1080P.mp4
  → Has "S" in number (MKBD-S137 not MKBD-137)

ses23.com NHDTA-609.1080p.mkv
  → Extracted "SES23" from domain watermark instead of "NHDTA-609"
```

**Why it failed**:
- Parser couldn't extract valid DVD ID
- Or extracted wrong ID (like ses23.com domain)
- Filenames need manual fixing

**Solutions**:

1. **Rename files properly**
   ```bash
   # Fix spam filename
   加入微信號13655544看妹不用等.avi → [unknown].avi

   # Fix SOE-753
   [NoDRM]-soe00753mhb.wmv → SOE-753.wmv

   # Fix NHDTA-609
   【ses23.com】NHDTA-609.1080p.mkv → NHDTA-609.mkv

   # Fix Tokyo Hot
   Tokyo_Hot_th101-060-111077.mp4 → th101-060-111077.mp4
   ```

2. **Use manual number override**
   ```bash
   ./mdc-cli file.mp4 -n CORRECT-123 -m 3
   ```

3. **Delete spam files**
   ```bash
   # This is just spam/advertising
   rm "加入微信號13655544看妹不用等.avi"
   ```

## Detailed Failure Breakdown

### Multi-Disc with A/B Suffix (6 files)
```
AVOP-212A.HD.mp4        → Base: AVOP-212 (exists in DB)
AVOP-212B.HD.mp4        → Base: AVOP-212 (exists in DB)
RCT-642A.mp4            → Base: RCT-642 (likely exists)
RCT-642B.mp4            → Base: RCT-642 (likely exists)
DVAJ00452A.mp4          → Base: DVAJ-452 (likely exists)
MIRD-129C.AVI           → Base: MIRD-129 (likely exists)
```

**Easy fix**: Strip suffix, re-run
```bash
# Rename approach
mv AVOP-212A.HD.mp4 AVOP-212-CD1.mp4
mv AVOP-212B.HD.mp4 AVOP-212-CD2.mp4

# Or manual override
./mdc-cli AVOP-212A.HD.mp4 -n AVOP-212 -m 3
./mdc-cli AVOP-212B.HD.mp4 -n AVOP-212 -m 3
```

### Parser Rejected (HQ suffix) (2 files)
```
DV888A.HQ.wmv → Parser saw "DV888A.HQ" (strict mode rejected)
DV888B.HQ.wmv → Parser saw "DV888B.HQ" (strict mode rejected)
```

**Fix**: Rename
```bash
mv DV888A.HQ.wmv DV-888-CD1.wmv
mv DV888B.HQ.wmv DV-888-CD2.wmv
```

### Spam/Garbage Filenames (4 files)
```
加入微信號13655544看妹不用等.avi → Delete (spam)
ri-ben-mei-zhi-bo-zuo-ai.mp4 → Rename or delete
ri-ben-mei-zhi-bo-zuo-ai-2.mp4 → Rename or delete
soe00753mhb.wmv → Rename to SOE-753.wmv
```

### Wrong ID Extracted (1 file)
```
【ses23.com】NHDTA-609.1080p.mkv
  → Extracted: SES23 (from watermark domain)
  → Should be: NHDTA-609
```

**Fix**: Rename to remove watermark
```bash
mv "【ses23.com】NHDTA-609.1080p.mkv" "NHDTA-609.mkv"
```

### Complex Format (2 files)
```
Tokyo_Hot_th101-060-111077.mp4 → Non-standard TokyoHot format
MKBD-S137-H265-1080P.mp4 → Has "S" in number
```

**Fix**: Manual override
```bash
# TokyoHot format might be: th101-060 or n111077
./mdc-cli Tokyo_Hot_th101-060-111077.mp4 -n n111077 -m 3

# MKBD: Try without S
./mdc-cli MKBD-S137-H265-1080P.mp4 -n MKBD-137 -m 3
```

### Genuinely Not in Database (12 files)
```
TT022, TYOD124, APAE054, BEB077, UUE29, UUV79,
FDD-1212, KV-082, HNDB036
```

**Options**:
1. Accept as unfindable
2. Try different scrapers (TMDB/IMDB)
3. Create manual NFO files
4. Search manually on JAVLibrary/JAVBus to verify

## Recommendations

### Quick Wins (16 files can be fixed easily)

1. **Multi-disc releases** (6 files) - Rename or override
2. **HQ suffix issues** (2 files) - Rename
3. **Watermark extraction** (1 file) - Rename
4. **Spam files** (4 files) - Delete or rename
5. **Extra suffixes** (1 file) - Clean filename
6. **Complex formats** (2 files) - Manual override

### Genuine Limitations (12 files)

- Not in any database
- May be:
  - Very old titles (pre-2000)
  - Small studio releases
  - Amateur content
  - Incorrectly identified numbers

### Success Rate Projection

**Current**: 67/93 = 72%

**After easy fixes**: 83/93 = 89% (estimated)

**After manual overrides**: 85/93 = 91% (estimated)

**Unfixable**: 8-10 titles likely not in any database

## Parser Enhancement Opportunities

### 1. Auto-strip disc suffixes
**Current behavior**:
```
AVOP-212A → Searches for "AVOP-212A" → Not found
```

**Enhanced behavior**:
```
AVOP-212A → Detects "A" suffix → Strips to "AVOP-212" → Found!
           → Stores disc_number: 1
```

**Implementation**:
```rust
// In number_parser.rs
if let Some(disc) = extract_disc_suffix(&number) {
    base_number = number.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C');
    attributes.disc_number = Some(disc);
}
```

### 2. Better quality tag removal
**Current behavior**:
```
DV888A.HQ → Rejected (strict mode)
```

**Enhanced behavior**:
```
DV888A.HQ → Strips ".HQ" → "DV888A" → Strips "A" → "DV888" → Success
```

### 3. Watermark domain filtering
**Current behavior**:
```
【ses23.com】NHDTA-609 → Extracts "SES23"
```

**Enhanced behavior**:
```
【ses23.com】NHDTA-609 → Filters "ses23.com" → Extracts "NHDTA-609"
```

### 4. Multi-ID extraction
**Current behavior**:
```
Tokyo_Hot_th101-060-111077 → Parsing error
```

**Enhanced behavior**:
```
Tokyo_Hot_th101-060-111077 → Try: n111077, th101-060, th101
                            → First match wins
```

## Configuration Options

### Enable Loose Parsing Mode
```bash
# Try with relaxed parsing (currently strict mode)
./mdc-cli /path/to/files -m 1 -s --loose-parsing
```

### Override Number for Specific Files
```bash
# For files with wrong extraction
./mdc-cli /path/to/file.mp4 -n CORRECT-123 -m 3
```

### Batch Rename Script
```bash
#!/bin/bash
# Fix common issues

# Strip HQ tags
rename 's/\.HQ//' *.wmv

# Strip watermarks
rename 's/【.*?】//' *.mkv

# Strip quality tags
rename 's/\.1080p//' *.mp4
rename 's/\.HD//' *.mp4

# Convert disc suffixes
rename 's/([A-Z]{3,}-\d+)A/$1-CD1/' *.mp4
rename 's/([A-Z]{3,}-\d+)B/$1-CD2/' *.mp4
rename 's/([A-Z]{3,}-\d+)C/$1-CD3/' *.mp4
```

## Summary

**Good News**:
- 72% success rate is excellent for first run
- 16/26 failures can be fixed with simple renames
- Cookie authentication working perfectly (all 67 successes)

**Areas for Improvement**:
- Auto-strip disc suffixes (A/B/C)
- Better quality tag filtering (.HQ, .HD)
- Watermark domain removal
- Multi-ID fallback attempts

**User Action Items**:
1. Delete spam files (4 files)
2. Rename multi-disc releases (8 files)
3. Clean watermark/quality tags (4 files)
4. Accept 10-12 titles as unfindable
5. Re-run batch processing

**Expected Final Rate**: 89-91% success (83-85/93)

This is within normal range for JAV metadata scraping!
