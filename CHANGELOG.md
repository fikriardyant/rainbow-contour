# Changelog

All notable changes to the **Rainbow Contour Engine** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
