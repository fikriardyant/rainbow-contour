# Implementation Plan - Issue #1: Ongrade Range, Custom Colors, & Inverted Cut/Fill Fix

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Memperbaiki definisi Cut vs Fill agar sesuai kaidah tambang/survey (Topo > Design = CUT/Gali, Topo < Design = FILL/Timbun), menambahkan toleransi ongrade (misal `-0.5` s/d `+0.5` m) di `config.dat`, palet warna kontras tinggi yang dapat dikonfigurasi, dan legenda dinamis.

**Architecture:** 
1. **Definisi Standar Tambang**:
   - `delta_z = z_topo - z_design` (atau bila tetap `z_design - z_topo`, maka `z_topo > z_design` adalah area galian/CUT).
   - Standar konsisten: kita definisikan `delta_z = z_topo - z_design`:
     - `delta_z > ongrade_max` => Tanah asli (Topo) lebih tinggi dari rencana (Design) = **CUT (Gali)** (warna merah/hangat).
     - `delta_z < ongrade_min` => Tanah asli lebih rendah dari rencana = **FILL (Timbun)** (warna biru/dingin).
     - `ongrade_min <= delta_z <= ongrade_max` => **ON GRADE** (toleransi pas, warna hijau elektrik).
2. **Ekstensi `EngineConfig` di `src/config.rs`**:
   - Parsing `ONGRADE_MIN`, `ONGRADE_MAX` (default `-0.50`, `0.50`).
   - Parsing 13 slot warna HEX (`COLOR_ONGRADE`, `COLOR_CUT_*`, `COLOR_FILL_*`).
3. **Volume Engine di `src/volume.rs`**:
   - `cut += (z_topo - z_design) * area` untuk titik di atas `ongrade_max`.
   - `fill += (z_design - z_topo) * area` untuk titik di bawah `ongrade_min`.
   - Menghitung luas zona `ongrade_area_m2`.
4. **HTML Viewer & Legend di `src/html_exporter.rs`**:
   - Menyelaraskan mapping ID warna dengan palet HEX dari config.
   - Legenda kop otomatis menampilkan label `CUT (+0.5m to > +16m)` dan `FILL (-0.5m to < -16m)`.

**Tech Stack:** Rust (1.80+), serde, HTML5 Canvas 2D, Neobrutalism UI, config.dat parser.

---

### Global Constraints
- Format `config.dat` tetap backward-compatible.
- Seluruh unit & integration test harus lulus dengan `cargo test --bins --tests`.

---

### Task 1: Fix Cut & Fill Logic in Grid Engine & Volume Calculation

**Files:**
- Modify: `src/grid_engine.rs:150-170`
- Modify: `src/volume.rs:1-32`
- Test: `tests/test_volume.rs`

- [ ] **Step 1: Write test for true mining cut/fill definition & ongrade tolerance in `tests/test_volume.rs`**

```rust
use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::volume::calculate_volume_with_tolerance;

#[test]
fn test_cut_fill_definition_and_ongrade() {
    // Topo = 100, Design = 90 -> Perlu digali 10m (CUT)
    // Topo = 100, Design = 110 -> Perlu ditimbun 10m (FILL)
    // Topo = 100, Design = 100.2 -> Selisih 0.2m (ONGRADE toleransi [-0.5, 0.5])
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 100.0, z_design: 90.0, delta_z: 10.0 },   // CUT 10m
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 100.0, z_design: 110.0, delta_z: -10.0 }, // FILL 10m
        GridPointDelta { x: 2.0, y: 0.0, z_topo: 100.0, z_design: 99.8, delta_z: 0.2 },    // ONGRADE
    ];
    let vol = calculate_volume_with_tolerance(&grid, 1.0, -0.5, 0.5);
    assert_eq!(vol.cut_m3, 10.0);
    assert_eq!(vol.fill_m3, 10.0);
    assert_eq!(vol.ongrade_area_m2, 1.0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_volume test_cut_fill_definition_and_ongrade -- --exact`
Expected: FAIL

- [ ] **Step 3: Update `src/grid_engine.rs` & `src/volume.rs`**

Di `src/grid_engine.rs`:
```rust
// Mining standard: delta_z = z_topo - z_design
// > 0 = Topo higher than Design (CUT / galian)
// < 0 = Topo lower than Design (FILL / timbunan)
delta_z: zt - zd,
```

Di `src/volume.rs`:
```rust
pub fn calculate_volume_with_tolerance(
    grid: &[GridPointDelta],
    step: f64,
    ongrade_min: f64,
    ongrade_max: f64,
) -> VolumeSummary {
    let cell_area = step * step;
    let mut cut = 0.0;
    let mut fill = 0.0;
    let mut ongrade_area = 0.0;

    for pt in grid {
        if pt.delta_z > ongrade_max {
            cut += (pt.delta_z) * cell_area;
        } else if pt.delta_z < ongrade_min {
            fill += (pt.delta_z.abs()) * cell_area;
        } else {
            ongrade_area += cell_area;
        }
    }

    VolumeSummary {
        cut_m3: cut,
        fill_m3: fill,
        net_m3: fill - cut,
        cell_area_m2: cell_area,
        ongrade_area_m2: ongrade_area,
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_volume test_cut_fill_definition_and_ongrade -- --exact`
Expected: PASS

---

### Task 2: Extend EngineConfig with Ongrade Settings & Color Palette

**Files:**
- Modify: `src/config.rs`
- Test: `tests/test_config.rs`

- [ ] **Step 1: Write failing test in `tests/test_config.rs`**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement config parsing, default high-contrast colors, and dat string serialization**
- [ ] **Step 4: Run test to verify it passes**

---

### Task 3: Dynamic High-Contrast Palette & Dynamic Legend in HTML Exporter

**Files:**
- Modify: `src/html_exporter.rs`
- Test: `tests/test_html_exporter.rs`

- [ ] **Step 1: Write test for custom color & ongrade legend in `tests/test_html_exporter.rs`**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement palette injection & legend update in `src/html_exporter.rs`**
- [ ] **Step 4: Run test to verify it passes**

---

### Task 4: Connect main.rs Pipeline & Verify E2E / GATES

**Files:**
- Modify: `src/main.rs`
- Modify: `config.dat`
- Test: `cargo test --bins --tests`
- Verify: `GATES.md`
