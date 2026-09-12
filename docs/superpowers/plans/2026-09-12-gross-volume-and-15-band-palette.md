# Gross Volume Calculation & 15-Band Tri-Green Transition Palette Implementation Plan

> **For implementer:** Use TDD throughout. Write failing test first. Watch it fail. Then implement.

**Goal:** Eliminate the volume cutoff deadband so Cut/Fill matches MineScape & Civil 3D 1:1, and upgrade the heatmap & legend to a 15-band palette featuring a 3-tier green transition zone around zero (`-1 to -0.2m`, `-0.2 to +0.2m`, `+0.2 to +1m`) with explicit numerical elevation ranges.

**Architecture:** 
- `src/volume.rs` provides standard gross volume accumulation ($\Delta Z > 0 \to \text{Cut}$, $\Delta Z < 0 \to \text{Fill}$) without threshold exclusion.
- `src/config.rs` expands palette and label fields to 15 bands and updates defaults.
- `src/html_exporter.rs` maps $\Delta Z$ to 15 color IDs, generates a 16-color JS palette array, and formats the HTML Kop legend table.
- `src/main.rs` integrates the gross volume call and passes the updated config to export.

**Tech Stack:** Rust 2021, Cargo, Serde, Tailwind CSS / HTML Canvas rasterizer.

---

### Task 1: Update Volume Calculation in `src/volume.rs` to Standard Gross Volume

**Files:**
- Modify: `src/volume.rs`
- Modify: `tests/test_volume.rs`

**Step 1: Write the failing test**
Update `tests/test_volume.rs` to assert that `calculate_volume` computes all $\Delta Z > 0$ as Cut and all $\Delta Z < 0$ as Fill with zero volume discarded, including values within $[-0.5, +0.5]$.
Also assert that `calculate_volume_with_tolerance` or `calculate_volume` accurately accounts for near-grade values without silent volume dropping.

```rust
#[test]
fn test_gross_volume_includes_near_grade() {
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 100.3, z_design: 100.0, delta_z: 0.3 },   // Cut 0.3
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 99.8, z_design: 100.0, delta_z: -0.2 },   // Fill 0.2
        GridPointDelta { x: 2.0, y: 0.0, z_topo: 105.0, z_design: 100.0, delta_z: 5.0 },   // Cut 5.0
    ];
    let step = 1.0; // area = 1.0 m2
    let summary = calculate_volume(&grid, step);
    assert!((summary.cut_m3 - 5.3).abs() < 1e-6);
    assert!((summary.fill_m3 - 0.2).abs() < 1e-6);
    assert!((summary.net_m3 - (0.2 - 5.3)).abs() < 1e-6);
}
```

**Step 2: Run test — confirm it passes or fails**
Command: `cargo test --test test_volume`

**Step 3: Update `src/volume.rs`**
Ensure `calculate_volume` is the canonical function and update `calculate_volume_with_tolerance` so it does not discard volume from `cut_m3` and `fill_m3`.

**Step 4: Run test — confirm it passes**
Command: `cargo test --test test_volume`

**Step 5: Commit**
`git add src/volume.rs tests/test_volume.rs && git commit -m "feat(volume): ensure gross volume calculation without deadband exclusion"`

---

### Task 2: Extend `EngineConfig` in `src/config.rs` & `config.dat` for 15 Bands

**Files:**
- Modify: `src/config.rs`
- Modify: `config.dat`
- Modify: `tests/test_html_exporter.rs`

**Step 1: Write the failing test**
In `tests/test_html_exporter.rs`, assert `EngineConfig` has:
- `color_cut_minor` (default `#FFE600`) and `label_cut_minor` (default `1-2m`)
- `color_fill_minor` (default `#00E5FF`) and `label_fill_minor` (default `-2 - -1m`)
- Updated `color_cut_to_grade` (`#76FF03`) and `label_cut_to_grade` (`0.2-1m`)
- Updated `color_ongrade` (`#00E676`) and `label_ongrade` (`-0.2 - 0.2m`)
- Updated `color_fill_to_grade` (`#1B5E20`) and `label_fill_to_grade` (`-1 - -0.2m`)

**Step 2: Run test — confirm it fails**
Command: `cargo test --test test_html_exporter`

**Step 3: Implement in `src/config.rs` and `config.dat`**
- Add fields `color_cut_minor`, `label_cut_minor`, `color_fill_minor`, `label_fill_minor` to `EngineConfig`.
- Update `parse_content` to load `COLOR_CUT_MINOR`, `LABEL_CUT_MINOR`, `COLOR_FILL_MINOR`, `LABEL_FILL_MINOR`.
- Update `config.dat` template and comments.

**Step 4: Run test — confirm it passes**
Command: `cargo test --test test_html_exporter`

**Step 5: Commit**
`git add src/config.rs config.dat tests/test_html_exporter.rs && git commit -m "feat(config): add 15-band palette and tri-green transition configuration"`

---

### Task 3: Upgrade `src/html_exporter.rs` with 15-Band Classification and Legend Layout

**Files:**
- Modify: `src/html_exporter.rs`
- Modify: `tests/test_html_exporter.rs`

**Step 1: Write the failing test**
In `tests/test_html_exporter.rs`:
- Test `delta_z_to_color_id`:
  - `dz = 18.0` $\to 1$
  - `dz = 14.0` $\to 2$
  - `dz = 10.0` $\to 3$
  - `dz = 6.0` $\to 4$
  - `dz = 3.0` $\to 5$
  - `dz = 1.5` $\to 6$ (Cut Minor 1-2m)
  - `dz = 0.5` $\to 7$ (Cut to Grade 0.2-1m)
  - `dz = 0.0` $\to 8$ (Level -0.2 to +0.2m)
  - `dz = -0.5` $\to 9$ (Fill to Grade -1 to -0.2m)
  - `dz = -1.5` $\to 10$ (Fill Minor -2 to -1m)
  - `dz = -3.0` $\to 11$
  - `dz = -6.0` $\to 12$
  - `dz = -10.0` $\to 13$
  - `dz = -14.0` $\to 14$
  - `dz = -18.0` $\to 15$
- Test that generated HTML contains all 16 palette entries (including 0 transparent) and 15 legend swatches.

**Step 2: Run test — confirm it fails**
Command: `cargo test --test test_html_exporter`

**Step 3: Implement in `src/html_exporter.rs`**
- Update `delta_z_to_color_id(dz: f64) -> u32` (or with config if needed).
- Update `paletteRGBA` array construction with 16 elements.
- Re-architect Kop LEGEND table to render all 15 bands in a clean layout:
  - Rows 1-3: Cut High/Mid/Low/Near/Minor pairs
  - Row 4: Cut to Grade `0.2 - 1m`
  - Row 5: On Grade `-0.2 - 0.2m` (centered)
  - Row 6: Fill to Grade `-1 - -0.2m`
  - Rows 7-9: Fill Minor/Near/Low/Mid/High/Deep pairs

**Step 4: Run test — confirm it passes**
Command: `cargo test --test test_html_exporter`

**Step 5: Commit**
`git add src/html_exporter.rs tests/test_html_exporter.rs && git commit -m "feat(exporter): implement 15-band color classification and kop legend"`

---

### Task 4: Connect `src/main.rs` & Verify Pipeline

**Files:**
- Modify: `src/main.rs`

**Step 1: Check existing `main.rs` volume calculation call**
Change line 170 in `src/main.rs` to call `calculate_volume(&grid, config.grid_step)`.

**Step 2: Compile & run check**
Command: `cargo check`

**Step 3: Run full test suite**
Command: `cargo test`

**Step 4: Commit**
`git add src/main.rs && git commit -m "feat(main): wire gross volume calculation and 15-band export in pipeline"`

---

### Task 5: End-to-End Verification with Bodydam Intan Dataset & Visual Check

**Files:**
- Test: `tests/test_sample_output.rs` (or dedicated integration test)

**Step 1: Run integration tests**
Verify that Bodydam Intan calculates CUT ~8,580 m³ and FILL ~10,407 m³ and produces valid HTML output.

**Step 2: Inspect output HTML**
Check generated HTML with headless browser or DOM check to ensure canvas renders without errors and legend renders completely.

**Step 3: Commit & Final Wrap**
`git add . && git commit -m "test(integration): verify gross volume parity with MineScape and 15-band visual output"`
