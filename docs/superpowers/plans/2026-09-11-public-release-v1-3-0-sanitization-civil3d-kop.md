# Public Release v1.3.0 Sanitization, Civil 3D Layout, and Automated Showcase Assets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sanitize the Rainbow Contour codebase for public open-source release by replacing all private company/mine identifiers with generic mining contractor profiles, replace the legacy logo with a generative 1:1 corporate mining emblem, update the automated screenshot assets, modernize the README with step-by-step tutorials, bump version to v1.3.0 with complete changelog, and push tag v1.3.0 to remote.

**Architecture:** 
1. Generate an isometric vector/geometric 1:1 corporate mining logo (`company_logo.png`) using SVG-to-Chrome headless rasterization.
2. Sanitize internal strings and defaults across `config.dat`, `src/config.rs`, `src/html_exporter.rs`, tests, and `.gitignore`.
3. Re-render the HTML viewer and execute `capture_screenshots.py` to produce pristine 2x Retina public showcase screenshots (`preview.png`, `summary.png`).
4. Overhaul `README.md` and `CHANGELOG.md`, then tag and push release `v1.3.0`.

**Tech Stack:** Rust (1.80+), Cargo, Python 3 (Pillow, NumPy), Headless Google Chrome, SVG, Git.

---

## Global Constraints
- Target version: `1.3.0`
- Zero occurrences of sensitive internal names or internal pit names in tracked code, tests, or screenshots.
- Generic corporate profile: `COMPANY_NAME = "PT MINING NUSANTARA PRIMA"`, `DISTRICT_NAME = "DISTRIK NUSANTARA"`, `DRAWN_BY = "Mine Engineer"`, `PROJECT_NAME = "PIT ALPHA"`, `RAINBOW_TITLE = "PIT ALPHA CUT & FILL"`.
- Company logo aspect ratio: 1:1 (square), transparent background, high resolution (1000x1000 or 1024x1024 px PNG).
- Non-technical release description matching CHANGELOG for easy contractor and mine management understanding.

---

## Interfaces
- Consumes: `trial_intan` or synthetic DXF surfaces for rendering, `capture_screenshots.py` CLI.
- Produces: `company_logo.png` (1:1 PNG), `screenshots/preview.png`, `screenshots/summary.png`, `Cargo.toml` (1.3.0), `CHANGELOG.md`, `README.md`, Git Tag `v1.3.0`.

---

### Task 1: Generate Generative 1:1 Ratio Corporate Mining Logo

**Files:**
- Create: `scripts/generate_logo.py`
- Replace: `company_logo.png`
- Test: `tests/test_logo_dimensions.py`

- [ ] **Step 1: Write test verifying logo exists, has 1:1 ratio, RGBA mode, and >= 512px size**

```python
# tests/test_logo_dimensions.py
import os
from PIL import Image

def test_company_logo_aspect_ratio_and_resolution():
    logo_path = "company_logo.png"
    assert os.path.exists(logo_path), f"{logo_path} must exist"
    with Image.open(logo_path) as im:
        w, h = im.size
        assert w == h, f"Logo must be 1:1 square ratio, got {w}x{h}"
        assert w >= 512, f"Logo must be at least 512px resolution, got {w}"
        assert im.mode in ("RGBA", "RGB"), f"Logo mode must be RGBA or RGB, got {im.mode}"

if __name__ == "__main__":
    test_company_logo_aspect_ratio_and_resolution()
    print("Logo dimensions test passed!")
```

- [ ] **Step 2: Run test to verify it fails with current non-1:1 PAMA logo**

Run: `python3 tests/test_logo_dimensions.py`
Expected: FAIL (current logo is 1786x2000, not 1:1).

- [ ] **Step 3: Implement logo generator script and generate 1:1 corporate emblem**

```python
# scripts/generate_logo.py
import os
import subprocess
import tempfile
from PIL import Image

def generate_logo(output_path="company_logo.png"):
    svg_content = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">
  <defs>
    <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#1E293B"/>
      <stop offset="100%" stop-color="#0F172A"/>
    </linearGradient>
    <linearGradient id="goldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#F59E0B"/>
      <stop offset="100%" stop-color="#D97706"/>
    </linearGradient>
    <linearGradient id="emeraldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#10B981"/>
      <stop offset="100%" stop-color="#047857"/>
    </linearGradient>
    <linearGradient id="cyanGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#38BDF8"/>
      <stop offset="100%" stop-color="#0284C7"/>
    </linearGradient>
    <filter id="shadow" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="16" stdDeviation="24" flood-color="#000000" flood-opacity="0.4"/>
    </filter>
  </defs>

  <!-- Circular Outer Seal / Frame -->
  <circle cx="512" cy="512" r="480" fill="url(#bgGrad)" stroke="#334155" stroke-width="16" filter="url(#shadow)"/>
  
  <!-- Outer Geometric Contour Rings -->
  <circle cx="512" cy="512" r="430" fill="none" stroke="#475569" stroke-width="4" stroke-dasharray="16 12"/>
  <circle cx="512" cy="512" r="390" fill="none" stroke="#64748B" stroke-width="6"/>

  <!-- Stylized Isometric Pit Hexagon / Strata Steps -->
  <!-- Upper Topo Surface Prism (Amber/Gold) -->
  <polygon points="512,180 772,330 512,480 252,330" fill="url(#goldGrad)" stroke="#0F172A" stroke-width="8"/>
  
  <!-- West Pit Bench Wall (Emerald Green) -->
  <polygon points="252,345 504,490 504,750 252,605" fill="url(#emeraldGrad)" stroke="#0F172A" stroke-width="8"/>
  
  <!-- East Pit Bench Wall (Cyan Blue) -->
  <polygon points="520,490 772,345 772,605 520,750" fill="url(#cyanGrad)" stroke="#0F172A" stroke-width="8"/>

  <!-- Internal Contour Step Terraces -->
  <polyline points="290,400 504,520 504,570 330,470" fill="#065F46" opacity="0.6"/>
  <polyline points="734,400 520,520 520,570 694,470" fill="#0369A1" opacity="0.6"/>

  <!-- Compass Star Zenith Point -->
  <circle cx="512" cy="330" r="22" fill="#FFFFFF" stroke="#0F172A" stroke-width="6"/>
  <line x1="512" y1="280" x2="512" y2="380" stroke="#0F172A" stroke-width="6" stroke-linecap="round"/>
  <line x1="462" y1="330" x2="562" y2="330" stroke="#0F172A" stroke-width="6" stroke-linecap="round"/>

  <!-- Base Inscription Bar -->
  <path d="M 330 830 Q 512 870 694 830" fill="none" stroke="#F59E0B" stroke-width="12" stroke-linecap="round"/>
</svg>"""

    with tempfile.NamedTemporaryFile("w", suffix=".svg", delete=False) as f:
        f.write(svg_content)
        svg_file = f.name

    tmp_png = output_path + ".tmp.png"
    cmd = [
        "google-chrome",
        "--headless",
        "--disable-gpu",
        "--default-background-color=00000000",
        "--window-size=1024,1024",
        f"--screenshot={tmp_png}",
        f"file://{os.path.abspath(svg_file)}"
    ]
    subprocess.run(cmd, check=True, capture_output=True)
    os.unlink(svg_file)

    # Crop to exact 1024x1024
    with Image.open(tmp_png) as im:
        cropped = im.crop((0, 0, 1024, 1024))
        cropped.save(output_path, "PNG")
    os.unlink(tmp_png)
    print(f"Generated 1:1 corporate logo at: {output_path}")

if __name__ == "__main__":
    generate_logo()
```

Run: `python3 scripts/generate_logo.py`

- [ ] **Step 4: Run test to verify it passes**

Run: `python3 tests/test_logo_dimensions.py`
Expected: PASS (1024x1024 RGBA square).

- [ ] **Step 5: Commit logo asset**

```bash
git add company_logo.png scripts/generate_logo.py tests/test_logo_dimensions.py
git commit -m "feat(brand): add generative 1:1 corporate mining logo and replace legacy logo"
```

---

### Task 2: Repository Data Sanitization & Public Release Defaults

**Files:**
- Modify: `config.dat`
- Modify: `src/config.rs:210-245`
- Modify: `src/html_exporter.rs:25-45`
- Modify: `tests/test_config.rs:1-40`
- Modify: `tests/test_html_exporter.rs:20-80`
- Modify: `tests/test_sample_output.rs:90-105`
- Modify: `capture_screenshots.py:25-35`
- Modify: `.gitignore`

- [ ] **Step 1: Write failing assertions in tests targeting generic defaults**

In `tests/test_config.rs`:
```rust
assert_eq!(cfg.company_name, "PT MINING NUSANTARA PRIMA");
assert_eq!(cfg.district_name, "DISTRIK NUSANTARA");
assert_eq!(cfg.drawn_by, "Mine Engineer");
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test test_config`
Expected: FAIL (`assert_eq!(cfg.company_name, ...)` failed).

- [ ] **Step 3: Update `config.dat`, `src/config.rs`, and `src/html_exporter.rs` with generic defaults**

In `config.dat`:
```ini
COMPANY_NAME=PT MINING NUSANTARA PRIMA
DISTRICT_NAME=DISTRIK NUSANTARA
DEPARTMENT_NAME=ENGINEERING & MINE PLANNING DEPT.
DRAWN_BY=Mine Engineer
PROJECT_NAME=PIT ALPHA
RAINBOW_TITLE=PIT ALPHA CUT & FILL
```

In `src/config.rs`:
```rust
company_name: "PT MINING NUSANTARA PRIMA".to_string(),
default_title: "PIT ALPHA CUT & FILL".to_string(),
drawn_by: "Mine Engineer".to_string(),
// ...
district_name: "DISTRIK NUSANTARA".to_string(),
```

In `src/html_exporter.rs`:
```rust
company: "PT MINING NUSANTARA PRIMA",
district: "DISTRIK NUSANTARA",
department: "ENGINEERING & MINE PLANNING DEPT.",
project_name: "PIT ALPHA",
drawn_by: "Mine Engineer",
```

In `tests/test_html_exporter.rs` and `tests/test_sample_output.rs`:
Update expectations to `"PT MINING NUSANTARA PRIMA"` and `"Pit Alpha - July 2026"`.

In `capture_screenshots.py`:
Update `DEFAULT_CANDIDATE_PATHS` to check generic paths first:
```python
DEFAULT_CANDIDATE_PATHS = [
    "output/rainbow-viewer.html",
    "sample_demo_output/rainbow-viewer.html",
    "/tmp/rainbow_showcase_gen/rainbow-viewer.html",
    "trial_intan/output/rainbow-viewer.html",
    "output_latest_trial/rainbow-viewer.html",
]
```

In `.gitignore`:
Ensure strict suppression of any trial or customer artifacts:
```gitignore
target/
demo_output/
demo_fixtures/
output*/
build/
*.dxf
*.dwg
*.json
!Cargo.lock
sample_demo_output/
graphify-out/
trial*/
mock*/
.unlazy/
*.tmp
```

- [ ] **Step 4: Run all test suites to verify they pass**

Run: `cargo test --bins --tests`
Expected: All 25+ tests PASS.

- [ ] **Step 5: Verify zero occurrences of private company and mine names in git**

Run: `git grep -i -E "pamapersada|sangatta|kpcs"`
Expected: No matches found in tracked files.

- [ ] **Step 6: Commit sanitization changes**

```bash
git add config.dat src/config.rs src/html_exporter.rs tests/ capture_screenshots.py .gitignore
git commit -m "fix(security): sanitize corporate defaults and enforce strict gitignore rules for public release"
```

---

### Task 3: Version Bump to v1.3.0 & CHANGELOG.md Documentation

**Files:**
- Modify: `Cargo.toml:3`
- Modify: `Cargo.lock`
- Modify: `CHANGELOG.md:8`

- [ ] **Step 1: Bump version in `Cargo.toml`**

Change `version = "1.2.0"` to `version = "1.3.0"` in `Cargo.toml`.

- [ ] **Step 2: Update Cargo.lock**

Run: `cargo check`
Expected: `Cargo.lock` updated with `1.3.0`.

- [ ] **Step 3: Add `[1.3.0] - 2026-09-11` entry in `CHANGELOG.md`**

Add detailed, professional, non-technical release notes covering:
- **Added**:
  - *Civil 3D Engineering Layout & 4-Sided UTM Coordinate Annotations*: Professional A4 landscape drafting border with 200m UTM coordinate lines, 50m sub-tick marks, CAD 4-point star compass North Arrow, and dynamic metric graphic scale bar.
  - *Restructured Technical Title Block (KOP)*: High-prominence company logo display, District and Department metadata fields, structured title hierarchy, and validation approval block (Drawn / Reviewed / Approved).
  - *District Metadata Support*: Added `DISTRICT_NAME` configuration parameter and `--district` CLI flag.
  - *Automated 2x Retina Screenshot Pipeline*: Added `capture_screenshots.py` leveraging headless Chrome and automatic container boundary detection.
  - *Generative 1:1 Corporate Logo*: Modern square geometric mining emblem replacing legacy assets.
- **Changed**:
  - *Compact Volume Summary Table*: High-density tabular summary of Cut ($m^3$), Fill ($m^3$), Net Difference ($m^3$), and On-Grade Surface Area ($m^2$).
  - *Unified 13-Range Isoline Legend*: High-contrast delta elevation scale aligned with Civil 3D layout aesthetics.
  - *Public Release Sanitization*: Standardized defaults on generic mining contractor profile (`PT MINING NUSANTARA PRIMA`, `DISTRIK NUSANTARA`, `PIT ALPHA`).
- **Fixed**:
  - *Strict Data Privacy*: Hardened `.gitignore` to prevent any raw survey attachments or client data leaks.

- [ ] **Step 4: Commit version bump and changelog**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): bump version to v1.3.0 and update CHANGELOG"
```

---

### Task 4: Interactive Viewer Regeneration & Pixel-Perfect Screenshot Updates

**Files:**
- Regenerate: `screenshots/preview.png`
- Regenerate: `screenshots/summary.png`

- [ ] **Step 1: Execute engine to produce sanitized HTML viewer**

Run:
```bash
cargo run --release -- \
  --topo trial_intan/TOPO_INTAN_WEEK0726_14022026.dxf \
  --design trial_intan/CVL_BUND_INE_CP_INTAN_CMD__R01_250827_646115.dxf \
  --company "PT MINING NUSANTARA PRIMA" \
  --district "DISTRIK NUSANTARA" \
  --department "ENGINEERING & MINE PLANNING DEPT." \
  --drawn-by "Mine Engineer" \
  --project-name "PIT ALPHA" \
  --rainbow-title "PIT ALPHA CUT & FILL" \
  --logo company_logo.png \
  --outdir /tmp/rainbow_public_render \
  --no-open
```
Expected: `/tmp/rainbow_public_render/rainbow-viewer.html` created successfully.

- [ ] **Step 2: Run `capture_screenshots.py` against sanitized HTML viewer**

Run:
```bash
python3 capture_screenshots.py \
  --html /tmp/rainbow_public_render/rainbow-viewer.html \
  --outdir screenshots \
  --scale 2
```
Expected:
`screenshots/preview.png` (approx 2246x1588 px)
`screenshots/summary.png` (approx 600x1530 px)
Both updated with new 1:1 logo, generic company name, and Civil 3D Kop.

- [ ] **Step 3: Verify screenshots using PIL and OCR/Text check**

```python
# scripts/verify_screenshots.py
from PIL import Image

def verify():
    p = Image.open("screenshots/preview.png")
    s = Image.open("screenshots/summary.png")
    print(f"Preview size: {p.size}, Summary size: {s.size}")
    assert p.size[0] > 2000 and p.size[1] > 1400
    assert s.size[0] > 500 and s.size[1] > 1400
    print("Screenshot sizes verified!")

if __name__ == "__main__":
    verify()
```
Run: `python3 scripts/verify_screenshots.py`
Expected: PASS.

- [ ] **Step 4: Commit updated screenshots**

```bash
git add screenshots/preview.png screenshots/summary.png
git commit -m "docs(screenshots): update preview and summary screenshots with sanitized Civil 3D Kop and 1:1 logo"
```

---

### Task 5: Comprehensive README Overhaul with Modern Tutorials & Workflows

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Draft restructured README.md content**

Structure:
1. **Hero & Badges**:
   - Rust Engine, Zero CAD License, Civil 3D Kop Layout, Offline CLI, Multi-OS.
2. **Visual Showcase**:
   - Embed `screenshots/preview.png` and `screenshots/summary.png`.
3. **Key Capabilities**:
   - High-throughput Delaunay TIN Meshing.
   - Lossless RLE Raster Heatmap with smooth 2D Marching Squares.
   - Civil 3D Engineering Layout: 4-sided UTM annotations, compass north arrow, graphic scale bar, approval title block.
   - AutoCAD Color Index (ACI) 3D Vector DXF export.
4. **Step-by-Step Tutorial & Quick Start**:
   - Option A: Interactive One-Click Launcher (`rainbow-contour.sh` / `rainbow-contour.bat`).
   - Option B: Interactive Metadata Wizard.
   - Option C: Command-Line Automation with CLI flags.
5. **Configuration Guide (`config.dat`)**:
   - Document all parameters including `COMPANY_NAME`, `DISTRICT_NAME`, `COMPANY_LOGO_PATH`, `GRID_INTERVAL`, `SUBTICK_INTERVAL`, `ONGRADE_MIN/MAX`, and custom color palette.
6. **Automated Screenshot Tool (`capture_screenshots.py`)**:
   - Guide on how users/engineers can generate 2x Retina presentation graphics.

- [ ] **Step 2: Apply edits to `README.md` and verify formatting**

Verify: Read `README.md`, ensure all links, markdown tables, and code snippets are clean. Confirm zero mentions of private client data.

- [ ] **Step 3: Commit updated README**

```bash
git add README.md
git commit -m "docs: overhaul README with Civil 3D layout guide, interactive wizard tutorial, and updated assets"
```

---

### Task 6: Git Commit, Push & GitHub Release v1.3.0

**Files:**
- Workspace Git repository

- [ ] **Step 1: Check git status to ensure working directory is completely clean**

Run: `git status`
Expected: `nothing to commit, working tree clean`.

- [ ] **Step 2: Push commits to origin main**

Run: `git push origin main`
Expected: Successfully pushed to `origin/main`.

- [ ] **Step 3: Create annotated tag v1.3.0 with non-technical release description**

Run:
```bash
git tag -a v1.3.0 -m "Release v1.3.0: Civil 3D engineering layout, automated screenshot generator, and public release sanitization

- Civil 3D Drawing Layout: Added A4 landscape engineering border with 4-sided UTM coordinate grid ticks, North Arrow star compass, and metric scale bar.
- Title Block (KOP) Upgrade: Prominent company logo, District and Department metadata fields, and approval sign-off block.
- Compact Volume Summary: High-density volume table detailing Cut, Fill, Net volume, and ongrade footprint.
- Unified Isoline Legend: Standardized 13-tier delta range legend with high-contrast color coding.
- Automated Screenshot Tool: Included capture_screenshots.py for 2x Retina presentation assets.
- Public Release Ready: Sanitized all defaults with generic mining enterprise profiles."
```

- [ ] **Step 4: Push tag to origin**

Run: `git push origin v1.3.0`
Expected: `[new tag] v1.3.0 -> v1.3.0`.

- [ ] **Step 5: Verify release tag on remote**

Run: `git ls-remote --tags origin | grep v1.3.0`
Expected: Tag `v1.3.0` confirmed on remote.
