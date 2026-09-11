# 🌈 Rainbow Contour Engine

> High-performance CLI & Civil 3D A4 Landscape PDF Generator for Open-Pit Mining Cut & Fill Evaluation.

![Rainbow Contour A4 Map Preview](./screenshots/preview.png)

[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)]()
[![Release](https://img.shields.io/badge/release-v1.3.0-emerald.svg)](https://github.com/fikriardyant/rainbow-contour/releases)

---

## ⚡ Overview

**Rainbow Contour** is a standalone, ultra-fast engineering CLI engine written in Rust. It eliminates reliance on expensive, slow-loading commercial mining CAD packages (Surpac, Minescape, Civil 3D) for routine cut & fill evaluations.

Given high-density Topo survey pickups and Pit Design models in DXF format, the engine triangulates surfaces in parallel via multi-threaded Delaunay TIN, calculates elevation differences ($\Delta Z = Z_{\text{topo}} - Z_{\text{design}}$), and directly produces:
1. **Civil 3D Engineering A4 Landscape Map (HTML / PDF)** with 4-sided UTM coordinate grid ticks, CAD star compass North Arrow, graphic scale bar, corporate title block (KOP), and compact volume summary.
2. **Interactive CAD Viewport** featuring cursor-centered zoom (0.2x to 30x), vector pan, and automated HUD hiding on PDF print.
3. **AutoCAD ACI 3D Vector DXF** isoline contours ready for direct import into any mining GIS software.
4. **Machine-Readable Volume JSON** for ERP or dispatch database ingestion.

---

## 📸 Showcase & Visual Output

| Civil 3D A4 Landscape Map Sheet | Technical Title Block & Volume Summary |
| :---: | :---: |
| ![A4 Kop Map](./screenshots/preview.png) | ![Volume Summary](./screenshots/summary.png) |

---

## 🌟 Key Capabilities

- **Native Delaunay TIN Meshing**: Multi-threaded parsing of `3DFACE`, `POLYLINE`, `LWPOLYLINE`, `LINE`, and `POINT` entities via Rayon with adaptive edge limits for small and large surface models.
- **Lossless RLE Raster Heatmap Engine**: Generates lightweight (< 1.5 MB) standalone HTML viewers with zero browser lag, zero memory leaks, and anti-aliased fill rendering.
- **Smooth 2D Marching Squares Contours**: Per-cell linear interpolation eliminates stair-step artifacts, producing smooth vector isolines at 2-meter gradient intervals.
- **Civil 3D Layout & 4-Sided UTM Grid**:
  - Outer and inner engineering drafting sheet margins (A4 Landscape 297 mm x 210 mm).
  - 200 m UTM grid lines with 50 m sub-tick marks across all 4 viewport borders (horizontal Easting at top/bottom, vertical Northing with -90° rotation at left/right).
  - CAD-standard 4-point star compass North Arrow and dynamic metric graphic scale bar.
- **Restructured Title Block (KOP)**:
  - Top corporate branding with proportional 1:1 company logo support (`company_logo.png`).
  - Site & organizational metadata (`COMPANY_NAME`, `DISTRICT_NAME`, `DEPARTMENT_NAME`).
  - 3-tier engineering sign-off validation block (`Drawn By`, `Reviewed By`, `Approved By`).
- **Compact Volume Summary Table**: Clean tabular layout reporting Cut ($m^3$), Fill ($m^3$), Net Difference ($m^3$), and On-Grade Surface Area ($m^2$).
- **Configurable 13-Slot High-Contrast Palette**: Full HEX color palette and legend range customization via `config.dat`.
- **Automated HUD Concealment**: Floating navigation controls (`+`, `-`, `FIT`, `100%`) hide automatically during 1-click PDF download to ensure pristine drawing exports.

---

## 🚀 Quick Start Tutorial

### Option 1: One-Click Launcher (Recommended for Field Laptops)

No programming knowledge or compilers required. Pre-built packages run 100% offline:

- **Windows**: Double-click **`rainbow-contour.bat`**.
- **Linux / macOS**: Run the launcher script from terminal:
  ```bash
  chmod +x rainbow-contour.sh
  ./rainbow-contour.sh
  ```

### Option 2: First-Run Metadata Wizard

When running for the first time (or when `FIRST_RUN=n` in `config.dat`), the engine guides you through an interactive setup wizard:
```text
======================================================================
  FIRST-RUN SETUP: IDENTITAS KOP & PERUSAHAAN
  (Tekan Enter untuk memakai nilai default, atau ketik untuk mengganti)
======================================================================
1. Company Name [PT MINING NUSANTARA PRIMA]: 
2. District / Site [DISTRIK NUSANTARA]: 
3. Department Name [ENGINEERING & MINE PLANNING DEPT.]: 
4. Company Logo Path [company_logo.png]: 
5. Coordinate System / Projection [UTM ZONE 50S (WGS84)]: 
----------------------------------------------------------------------
  [Saved] Pengaturan identitas berhasil disimpan ke config.dat (FIRST_RUN=y).
```

### Option 3: Command-Line Flags & Automated Scripting

For headless batch jobs, pass input parameters directly via CLI flags:

```bash
rainbow-contour \
  --topo /path/to/topo_survey.dxf \
  --design /path/to/pit_design.dxf \
  --boundary /path/to/boundary.dxf \
  --company "PT MINING NUSANTARA PRIMA" \
  --district "DISTRIK NUSANTARA" \
  --project "PIT ALPHA" \
  --rainbow-title "PIT ALPHA CUT & FILL EVALUATION" \
  --drawn-by "Mine Engineer" \
  --step 0.50 \
  --outdir ./output \
  --no-open
```

#### Available CLI Arguments:
- `-t, --topo <PATH>`: Path to Topo survey DXF.
- `-d, --design <PATH>`: Path to Pit Design DXF.
- `-b, --boundary <PATH>`: Optional boundary DXF (auto-detected if omitted).
- `-s, --step <FLOAT>`: Grid sampling resolution in meters (default: `0.50`).
- `-o, --outdir <DIR>`: Output directory (default: `./output`).
- `--company <NAME>`: Company name displayed in title block.
- `--district <NAME>`: District or mine site name.
- `--project <NAME>`: Project / Pit name.
- `--rainbow-title <TITLE>`: Map drawing title.
- `--drawn-by <NAME>`: Drafter / Engineer name.
- `--reviewed-by <NAME>`: Reviewer name.
- `--approved-by <NAME>`: Approver name.
- `--design-name <NAME>`: Design revision reference name.
- `--topo-date <DATE>`: Topo survey date string.
- `--logo <PATH>`: Path to company logo image file (`.png` / `.jpg`).
- `--config <PATH>`: Path to custom configuration file (default: `config.dat`).
- `--no-open`: Suppress automatic opening of HTML viewer in web browser.

---

## ⚙️ Configuration Reference (`config.dat`)

All parameters, defaults, and visual themes can be customized in `config.dat`:

```ini
# ======================================================================
# RAINBOW CONTOUR ENGINE CONFIGURATION (config.dat)
# ======================================================================

# --- WIZARD SETUP STATUS ---
FIRST_RUN=y

# --- IDENTITAS PERUSAHAAN & SITE ---
COMPANY_NAME=PT MINING NUSANTARA PRIMA
DISTRICT_NAME=DISTRIK NUSANTARA
DEPARTMENT_NAME=ENGINEERING & MINE PLANNING DEPT.
COMPANY_LOGO_PATH=company_logo.png

# --- METADATA & VALIDASI PETA (Civil 3D) ---
PROJECT_NAME=PIT ALPHA
RAINBOW_TITLE=PIT ALPHA CUT & FILL
MAP_SUBTITLE=ISOPACH DIFFERENCE (TOPO - DESIGN)
DRAWN_BY=Mine Engineer
REVIEWED_BY=Reviewer
APPROVED_BY=Approver
COORDINATE_SYSTEM=UTM ZONE 50S (WGS84)
TOPO_DATE=28 July 2026
DESIGN_NAME=
AUTO_OPEN_BROWSER=true

# --- DEFAULT INPUT DXF (Biarkan kosong jika ingin ditanyakan saat runtime) ---
TOPO_PATH=
DESIGN_PATH=

# --- GRID ANOTASI PETA (Meter) ---
GRID_INTERVAL=200.0
SUBTICK_INTERVAL=50.0

# --- PARAMETER PERHITUNGAN GRID & SURFACE ---
GRID_STEP=0.50
MAX_TIN_EDGE=300.0
WEEDING_MIN_DIST=0.50
SUPPLEMENT_MAX_DIST=10.0

# --- TOLERANSI ONGRADE (Meter) ---
ONGRADE_MIN=-0.50
ONGRADE_MAX=0.50

# --- LABEL LEGENDA CUT / FILL / ONGRADE ---
LABEL_CUT_DEEP=> 16m
LABEL_CUT_HIGH=12-16m
LABEL_CUT_MID=8-12m
LABEL_CUT_LOW=4-8m
LABEL_CUT_NEAR=2-4m
LABEL_CUT_TO_GRADE=0-2m
LABEL_ONGRADE=ON GRADE
LABEL_FILL_TO_GRADE=0-2m
LABEL_FILL_NEAR=2-4m
LABEL_FILL_LOW=4-8m
LABEL_FILL_MID=8-12m
LABEL_FILL_HIGH=12-16m
LABEL_FILL_DEEP=> 16m

# --- PALET WARNA KONTUR (HEX) ---
COLOR_CUT_DEEP=#7F0000
COLOR_CUT_HIGH=#B71C1C
COLOR_CUT_MID=#D50000
COLOR_CUT_LOW=#FF3D00
COLOR_CUT_NEAR=#FF9100
COLOR_CUT_TO_GRADE=#FFE600
COLOR_ONGRADE=#00E676
COLOR_FILL_TO_GRADE=#00E5FF
COLOR_FILL_NEAR=#00B0FF
COLOR_FILL_LOW=#2979FF
COLOR_FILL_MID=#0039CB
COLOR_FILL_HIGH=#4A148C
COLOR_FILL_DEEP=#311B92

# --- ISOLINE CONTOUR LEVELS (Meter) ---
CONTOUR_LEVELS=-20.0,-18.0,-16.0,-14.0,-12.0,-10.0,-8.0,-6.0,-4.0,-2.0,0.0,2.0,4.0,6.0,8.0,10.0,12.0,14.0,16.0,18.0,20.0

# --- DEFAULT OUTPUT DIRECTORY ---
DEFAULT_OUTDIR=./output
```

---

## 📷 Automated Screenshot Pipeline (`capture_screenshots.py`)

Rainbow Contour includes an automated screenshot generator using Google Chrome headless rendering and automated container boundary detection:

```bash
# Capture 2x Retina screenshots from generated viewer
python3 capture_screenshots.py --html ./output/rainbow-viewer.html --scale 2

# Output files generated in screenshots/
# - screenshots/preview.png  (Full A4 Landscape Drawing Sheet)
# - screenshots/summary.png  (Title Block & Compact Volume Table)
```

---

## 📦 Generated Output Files

Every run populates the specified output folder (default: `./output/`) with:

1. **`rainbow-viewer.html`**:
   - Zero-dependency standalone HTML viewer.
   - Interactive zoom & pan in browser.
   - 1-click **Download PDF (A4 Landscape)** using built-in high-DPI canvas rasterization.
2. **`rainbow-output.dxf`**:
   - 3D AutoCAD Color Index (ACI) contour polyline drawing.
   - Colors map directly to elevation delta isolines (Red = Cut, Green = On-Grade, Cyan/Blue = Fill).
3. **`volume-summary.json`**:
   - Structured JSON summary containing total cut volume ($m^3$), fill volume ($m^3$), net difference ($m^3$), and ongrade area footprint ($m^2$).

---

## 🛠️ Building from Source

To compile the latest release binary from source:

```bash
# Clone the repository
git clone https://github.com/fikriardyant/rainbow-contour.git
cd rainbow-contour

# Build optimized release binary
cargo build --release

# Run test suite
cargo test --bins --tests

# The binary will be located at target/release/rainbow-contour
```

---

## 📄 License

Distributed under the MIT or Apache-2.0 License. See [LICENSE](./LICENSE) for details.
