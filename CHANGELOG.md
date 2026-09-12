# Changelog

All notable changes to the **Rainbow Contour Engine** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.4.0] - 2026-09-12

### Added
- **Gross Volume Parity with MineScape & Civil 3D**: Discarded the ongrade deadband volume exclusion. The calculation now computes full gross Cut ($\Delta Z > 0$) and Fill ($\Delta Z < 0$), achieving < 0.2% parity with MineScape and Civil 3D benchmark figures.
- **3x3 Spatial Raster Majority Denoiser**: Implemented an 8-neighborhood majority filter on the raster grid before RLE compression to eliminate high-frequency quantization noise (salt-and-pepper pixel artifacts along threshold boundaries).
- **15-Band Tri-Green Spectral Palette**: Expanded elevation bands from 13 to 15, adding transitional sub-bands for near-grade excavation (`0.2-1m` Lime Green, `1-2m` Yellow) and fill (`0.2-1m` Sea Green, `1-2m` Teal).

### Changed
- **Symmetric 2-Column Legend (CUT vs FILL)**: Redesigned the KOP legend into a clean, balanced 2-column tabular layout separating Cut (+) and Fill (-). Removed ambiguous negative double-hyphens (` - - `) and eliminated empty grid holes.
- **Explicit Grade Center Range**: Standardized the central level bar to explicit numerical bounds (`LEVEL / ON GRADE: ±0.2m`) rather than generic text.
- **Smooth GIS Fill Spectrum**: Replaced the abrupt dark forest green-to-cyan transition with a smooth, continuous natural gradient (`#00E676` $\to$ `#00BFA5` $\to$ `#00ACC1` $\to$ `#00B0FF`).

---

## [1.3.0] - 2026-09-11

### Added
- **Civil 3D Engineering Drawing Layout**: Upgraded the A4 landscape map sheet with professional mining drawing standards:
  - Precise outer and inner drawing border frames.
  - 4-sided coordinate tick marks and hairline grid lines across the entire map area (horizontal Easting at top & bottom, vertical Northing with -90° rotation at left & right).
  - CAD-standard 4-point star compass North Arrow in the upper left corner.
  - Dynamic metric graphic scale bar and ratio scale indicator in the lower left corner.
- **Restructured Title Block (KOP) Hierarchy**: Redesigned the right-hand technical title block to mirror standard civil and mine engineering construction drawings:
  - Top corporate branding header with prominent company logo support.
  - Dedicated site metadata fields including District and Department identity.
  - Clear hierarchical project naming, subtitle, and drawing numbers.
  - Formal 3-tier validation sign-off block (Drawn By, Reviewed By, Approved By).
- **District & Site Metadata Support**: Added `DISTRICT_NAME` parameter in `config.dat` and `--district` CLI argument, fully integrated with the first-run setup wizard.
- **Automated 2x Retina Screenshot Generator**: Added `capture_screenshots.py` leveraging headless Chrome and automatic container boundary detection to generate crystal-clear, high-resolution documentation and showcase images (`preview.png` and `summary.png`).
- **Generative 1:1 Corporate Emblem**: Replaced legacy branding with a clean, modern geometric 1:1 square corporate mining emblem (`company_logo.png`) featuring stylized pit terraces and elevation contours.

### Changed
- **Compact Volume Summary Table**: Replaced oversized card widgets with a unified, high-density tabular summary showing Cut ($m^3$), Fill ($m^3$), Net Difference ($m^3$), and On-Grade Surface Area ($m^2$) for clean, uncluttered reporting.
- **Unified 13-Range Isoline Legend**: Standardized elevation delta intervals with clean, uniform range brackets and vivid high-contrast color coding.
- **Public Release Sanitization**: Replaced all proprietary site and contractor names across default configurations, engine fallbacks, and test fixtures with generic mining enterprise profiles (`PT MINING NUSANTARA PRIMA`, `DISTRIK NUSANTARA`, `PIT ALPHA`).

### Fixed
- **Strict Repository Privacy Protection**: Hardened `.gitignore` to strictly exclude all raw customer survey DXFs, trial folders (`trial*/`), mock layouts (`mock*/`), output directories (`output*/`), and temporary cache files from version control.

---

## [1.2.0] - 2026-09-08

### Added
- **Interactive CAD Viewport Navigation**: Added cursor-centered mouse wheel zoom (0.2x to 30x) and drag panning to the HTML viewer canvas. Design polylines re-render dynamically as sharp vectors on every redraw.
- **Automated HUD Concealment**: Floating navigation buttons hide automatically during PDF export to keep the exported A4 landscape sheet clean.

### Changed
- **Config-Driven 0.50m Default Sampling**: Updated the default `GRID_STEP` in `config.dat` and engine code to 0.50m, providing 4x higher resolution over the previous 1.0m baseline while keeping file sizes lightweight.

### Fixed
- **Small Surface Adaptive Triangulation**: Connected `MAX_TIN_EDGE`, `WEEDING_MIN_DIST`, and `SUPPLEMENT_MAX_DIST` from `config.dat` to the mesh parser. The engine adapts triangle edge limits for small models (diagonal under 100m) to stop Delaunay triangles from stretching across voids.
- **Smooth 2D Marching Squares Isolines**: Upgraded contour generation with per-cell linear edge interpolation, removing faceted stair-step artifacts from isolines.
- **Preserved Sub-Decimeter Grid Steps**: Fixed `html_exporter` step detection to read `config.grid_step` directly from configuration, preserving custom steps like 0.05m or 0.25m without falling back to 1.0m.
- **Repository Cleanliness**: Added `trial_intan/` and `graphify-out/` to `.gitignore` to keep raw survey attachments and local graph indexes untracked.

---

## [1.1.1] - 2026-09-03

### Added
- **Configurable Ongrade Tolerance Range**: Added `ONGRADE_MIN` and `ONGRADE_MAX` parameters (default `-0.50m` to `+0.50m`) in `config.dat` and engine calculation.
- **Ongrade Surface Area**: Added calculation of `ongrade_area_m2` in volume summary for exact ongrade footprint tracking.
- **Customizable High-Contrast Palette**: 13-slot vivid HEX color palette in `config.dat` replacing hardcoded colors for sharp visibility in both dark CAD canvas and white PDF exports.
- **Configurable Legend Labels**: All 13 legend text labels can now be customized directly via `config.dat` (`LABEL_CUT_*`, `LABEL_ONGRADE`, `LABEL_FILL_*`).

### Changed
- **Unified & Clean Legend Format**: Standardized all legend range labels to compact, consistent notation (`> 16m (Cut)`, `12-16m`, `8-12m`, `4-8m`, `2-4m`, `0-2m`, `ON GRADE`, `0-2m`, `2-4m`, `4-8m`, `8-12m`, `12-16m`, `> 16m (Fill)`).
- **Corrected Survey Elevation Differencing**: Realigned delta computation to standard mining convention ($\Delta Z = Z_{\text{topo}} - Z_{\text{design}}$) where positive values represent Cut (excavation) and negative values represent Fill.
- **Enhanced KOP Sidebar Layout**: Replaced narrow div container on the left coordinate ruler with full-precision SVG matrix rendering, and added `word-break: break-all;` on long design names to prevent table overflow.
- **Sample Output Cleanup**: Added `sample_demo_output/` to `.gitignore` and removed demo binaries from repository tracking.

---

## [1.1.0] - 2026-09-02

### Added
- **Interactive Metadata Prompts**: Added prompts during runtime for `Company Name`, `Map Title` / `Project Title`, `Drawn By`, and `Design Name` with intelligent defaults from `config.dat` and input DXF file names.
- **Dynamic Date Created**: Automatically generates today's formatted date (`D MMMM YYYY`, e.g. `2 September 2026`) in Kop metadata without manual input.
- **Smart Topo Survey Date**: Automatically reads the Topo DXF file's last modified timestamp as the default suggested date in interactive prompts.
- **Company Logo Support (`company_logo.png`)**: Added support for corporate logo display on the top section of the Kop sidebar.
- **Auto-Persist Logo**: Automatically copies any newly supplied logo file to `./company_logo.png` for seamless reuse in subsequent runs.
- **Auto-Open Default Browser**: Automatically opens the generated `rainbow-viewer.html` in the user's default web browser across Windows, macOS, and Linux upon completion (with `--no-open` flag support).
- **Proportional Logo Sizing**: High-definition scaling with CSS `object-contain` (max-height 93px) preventing stretching or distortion.
- **Standalone Linux Package**: Pre-configured distribution directory in `./build/linux/` with `run_test.sh` test runner.

### Changed
- Reordered header layout on Kop sidebar: Company name & logo now sit prominently at the top above "PETA RAINBOW CONTOUR" and the map title.
- Updated CLI parser and `config.dat` configuration parameters to synchronize with all new metadata fields and options.

---

## [1.0.1] - 2026-09-02

### Added
- Cross-platform GitHub Actions release workflow for Windows (x86_64 MSVC), macOS (Apple Silicon ARM64 & Intel), and Linux.
- Added native batch (`.bat`) and shell (`.sh`) launchers for non-Rust/non-Cargo environments.

---

## [1.0.0] - 2026-09-01

### Added
- Initial core release of Rainbow Contour Cut & Fill Engine.
- 3D Delaunay triangulation for TIN surface modeling from DXF entities (3DFACE, POLYLINE, LWPOLYLINE, LINE).
- High-performance Marching Squares isoline contour generation.
- Instant HTML5/Canvas Neobrutalism map visualizer with client-side PDF Kop exporter via html2canvas & jsPDF.
- Volume estimation engine (Cut, Fill, Net) with comma-formatted metric display.
