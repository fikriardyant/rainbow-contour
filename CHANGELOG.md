# Changelog

All notable changes to the **Rainbow Contour Engine** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
