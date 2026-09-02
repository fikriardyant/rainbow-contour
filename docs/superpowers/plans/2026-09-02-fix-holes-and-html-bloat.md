# Fix TIN Holes & Optimize HTML Viewer Size Implementation Plan

**Goal:**
1. Fix spot bolong-bolong pada hasil interpolasi kontur dan triangulasi pit tambang.
2. Optimasi ukuran file `rainbow-viewer.html` dari ratusan MB (62MB - 267MB) menjadi sangat ringan (< 500KB) dengan kompresi raster canvas / data grid efisien.

---

### Task 1: Fix Triangulation Holes & Interpolation Continuity
- Modify: `src/dxf.rs` (Increase Delaunay max edge limit to 250m for wide pit crest-to-toe and pit floors, preserve polylines vertex densification).
- Modify: `src/grid_engine.rs` (Refine point-in-mesh bounding & barycentric epsilon margin).
- Test: `tests/test_dxf_polyline.rs` & `tests/test_grid_engine.rs`

### Task 2: Optimize HTML Viewer Size (Lossless Raster Heatmap / Fast Compression)
- Modify: `src/html_exporter.rs`
  - Ganti embedding jutaan JSON `[x, y, dz]` text mentah dengan representasi compact grid PNG Base64 Data URL atau RLE/Uint8Array yang dirender instan ke Canvas.
  - Vektor garis CAD tetap dipertahankan sebagai polyline CAD crisp.
- Test: `tests/test_html_exporter.rs`

### Task 3: Green-Gate Verification & E2E Validation
- Verify `cargo test --tests`
- Verify generated sample files
