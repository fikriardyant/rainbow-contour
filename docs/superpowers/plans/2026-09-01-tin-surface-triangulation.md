# 3D Polyline / Point Delaunay Triangulation & TIN Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add TIN surface generation capability (2D Delaunay Triangulation with 3D Z-interpolation) to `rainbow-contour` so it can seamlessly parse and mesh raw 3D Polylines / Vertices / Points from mining CAD DXF files alongside existing `3DFACE` meshes.

**Architecture:** 
1. Extend `src/dxf.rs` to extract 3D points from `POLYLINE`/`VERTEX`, `LWPOLYLINE`, `LINE`, and `POINT` entities in addition to `3DFACE`.
2. Add `delaunator` crate (industry-standard, extremely fast O(N log N) 2D Delaunay triangulation).
3. Implement `triangulate_points(&[Point3D]) -> Vec<Triangle3D>` in `src/tin.rs` (or `src/dxf.rs`), with spatial deduplication / downsampling for dense survey points.
4. Auto-fallback / auto-triangulate in `parse_dxf_mesh` or pipeline when no `3DFACE` entities are present.

**Tech Stack:** Rust 2021, `delaunator = "1.1.0"`, `rayon`.

---

### Task 1: Add `delaunator` dependency and 3D Polyline Parsing in `dxf.rs`

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/dxf.rs`
- Test: `tests/test_dxf_polyline.rs`

- [ ] **Step 1: Add `delaunator` to `Cargo.toml`**
- [ ] **Step 2: Write failing test `tests/test_dxf_polyline.rs`**
- [ ] **Step 3: Implement DXF polyline point extraction and Delaunay TIN meshing**
- [ ] **Step 4: Run test to verify it passes**

---

### Task 2: Integrate TIN Triangulation into Pipeline & Memory-Efficient Mesh Construction

**Files:**
- Modify: `src/dxf.rs` / `src/lib.rs` / `src/main.rs`
- Test: `tests/test_tin_triangulation.rs`

- [ ] **Step 1: Write integration test for mixed 3DFACE and Polyline inputs**
- [ ] **Step 2: Update `main.rs` CLI progress logging and pipeline fallback**
- [ ] **Step 3: Run all cargo tests**

---

### Task 3: Verify with Real-World Mining Data (`Topo_Design_trial`)

**Files:**
- Test against `/home/cells/Documents/Topo_Design_trial/TP_WK33.Dxf` and `PAMA_PIT_SPE_PNL2_SCBD_SOUTH_R02_260607_DESIGN.dxf`

- [ ] **Step 1: Run benchmark and trial run on real pit data**
- [ ] **Step 2: Validate cut/fill output and generated viewer**
