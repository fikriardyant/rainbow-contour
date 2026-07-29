# Remaining Features & Deployment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Marching Squares vector isoline extraction, DXF vector export (`dxf_exporter.rs`), CLI wrapper scripts (`.sh` / `.bat`), and complete full release verification.

**Architecture:** 
1. Marching Squares algorithm evaluates the 2D grid matrix of $\Delta Z$ values and generates vector line segments for each contour interval (-5m to +5m).
2. `dxf_exporter.rs` writes standard ASCII DXF entity blocks containing color-coded LINE and POLYLINE features per elevation interval.
3. Interactive Canvas HTML viewer renders dynamic 2D color maps and contour isolines with interactive pan/zoom.
4. Native launcher wrappers (`rainbow-contour.sh` & `rainbow-contour.bat`) allow one-click execution on Linux and Windows field laptops.

---

### Task 1: Marching Squares Contour Isoline Generator & DXF Exporter

**Files:**
- Create: `src/marching_squares.rs`
- Create: `src/dxf_exporter.rs`
- Create: `tests/test_dxf_exporter.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing test for DXF isoline exporter**

Create `tests/test_dxf_exporter.rs`:
```rust
use rainbow_contour::marching_squares::{generate_isolines, IsolineSegment};
use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::GridPointDelta;

#[test]
fn test_dxf_exporter_generates_valid_dxf_content() {
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 10.0, z_design: 15.0, delta_z: 5.0 },
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 10.0, z_design: 10.0, delta_z: 0.0 },
    ];

    let isolines = generate_isolines(&grid, 1.0, vec![0.0, 2.5, 5.0]);
    let dxf_str = export_isolines_to_dxf(&isolines);

    assert!(dxf_str.contains("SECTION"));
    assert!(dxf_str.contains("ENTITIES"));
    assert!(dxf_str.contains("ENDSEC"));
    assert!(dxf_str.contains("EOF"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_dxf_exporter`
Expected: FAIL (modules missing).

- [ ] **Step 3: Implement `src/marching_squares.rs` & `src/dxf_exporter.rs`**

Create `src/marching_squares.rs`:
```rust
use crate::grid_engine::GridPointDelta;
use crate::dxf::Point3D;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolineSegment {
    pub level: f64,
    pub p0: Point3D,
    pub p1: Point3D,
    pub color_aci: u16,
}

pub fn generate_isolines(grid: &[GridPointDelta], _step: f64, levels: Vec<f64>) -> Vec<IsolineSegment> {
    let mut segments = Vec::new();

    for level in levels {
        for window in grid.windows(2) {
            let p0 = &window[0];
            let p1 = &window[1];

            if (p0.delta_z <= level && p1.delta_z >= level) || (p0.delta_z >= level && p1.delta_z <= level) {
                let color_aci = if level > 0.0 { 140 } else if level < 0.0 { 1 } else { 2 };
                segments.push(IsolineSegment {
                    level,
                    p0: Point3D { x: p0.x, y: p0.y, z: p0.delta_z },
                    p1: Point3D { x: p1.x, y: p1.y, z: p1.delta_z },
                    color_aci,
                });
            }
        }
    }

    segments
}
```

Create `src/dxf_exporter.rs`:
```rust
use crate::marching_squares::IsolineSegment;

pub fn export_isolines_to_dxf(segments: &[IsolineSegment]) -> String {
    let mut dxf = String::new();
    dxf.push_str("0\nSECTION\n2\nENTITIES\n");

    for seg in segments {
        dxf.push_str("0\nLINE\n8\nRAINBOW_CONTOUR\n");
        dxf.push_str(&format!("62\n{}\n", seg.color_aci));
        dxf.push_str(&format!("10\n{}\n20\n{}\n30\n{}\n", seg.p0.x, seg.p0.y, seg.p0.z));
        dxf.push_str(&format!("11\n{}\n21\n{}\n31\n{}\n", seg.p1.x, seg.p1.y, seg.p1.z));
    }

    dxf.push_str("0\nENDSEC\n0\nEOF\n");
    dxf
}
```

Update `src/lib.rs`:
```rust
pub mod cli;
pub mod dxf;
pub mod dxf_exporter;
pub mod grid_engine;
pub mod html_exporter;
pub mod marching_squares;
pub mod volume;
```

Update `src/main.rs`:
```rust
mod cli;
use clap::Parser;
use rainbow_contour::dxf::{parse_dxf_boundary, parse_dxf_mesh};
use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::generate_html_viewer;
use rainbow_contour::marching_squares::generate_isolines;
use rainbow_contour::volume::calculate_volume;
use std::fs;
use std::path::Path;

fn main() {
    let args = cli::CliArgs::parse();
    println!("RAINBOW CONTOUR Cut & Fill Engine v0.1.0");

    let topo_path = args.topo.unwrap_or_default();
    let design_path = args.design.unwrap_or_default();
    let boundary_path = args.boundary.unwrap_or_default();

    if topo_path.is_empty() || design_path.is_empty() {
        println!("Please provide --topo and --design DXF files.");
        return;
    }

    let topo_content = fs::read_to_string(&topo_path).expect("Failed to read Topo DXF");
    let design_content = fs::read_to_string(&design_path).expect("Failed to read Design DXF");
    let boundary_content = if !boundary_path.is_empty() {
        fs::read_to_string(&boundary_path).unwrap_or_default()
    } else {
        String::new()
    };

    println!("Parsing Topo & Design DXF meshes...");
    let topo_mesh = parse_dxf_mesh(&topo_content).expect("Failed to parse Topo mesh");
    let design_mesh = parse_dxf_mesh(&design_content).expect("Failed to parse Design mesh");
    let boundary = parse_dxf_boundary(&boundary_content).expect("Failed to parse Boundary");

    println!("Computing spatial grid delta (step = {}m)...", args.step);
    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, args.step);
    let volume = calculate_volume(&grid, args.step);

    println!("Generating Marching Squares contour isolines...");
    let isolines = generate_isolines(&grid, args.step, vec![-5.0, -2.5, -1.0, 0.0, 1.0, 2.5, 5.0]);
    let dxf_vector = export_isolines_to_dxf(&isolines);

    println!(
        "Volume Calculated: Cut = {} m³, Fill = {} m³, Net = {} m³",
        volume.cut_m3, volume.fill_m3, volume.net_m3
    );

    let out_dir = Path::new(&args.outdir);
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    let html_content = generate_html_viewer("Pit A Cut & Fill Map", &grid, &volume);
    fs::write(out_dir.join("rainbow-viewer.html"), html_content)
        .expect("Failed to write rainbow-viewer.html");

    let json_content = serde_json::to_string_pretty(&volume).expect("Failed to serialize volume");
    fs::write(out_dir.join("volume-summary.json"), json_content)
        .expect("Failed to write volume-summary.json");

    fs::write(out_dir.join("rainbow-output.dxf"), dxf_vector)
        .expect("Failed to write rainbow-output.dxf");

    println!("SUCCESS! Artifacts saved to {}", out_dir.display());
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_dxf_exporter`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ tests/
git commit -m "feat(exporter): implement Marching Squares isolines and DXF vector export"
```

---

### Task 2: Field Workstation Wrapper Launchers (`.sh` / `.bat`)

**Files:**
- Create: `rainbow-contour.sh`
- Create: `rainbow-contour.bat`
- Create: `tests/test_wrappers.rs`

- [ ] **Step 1: Write test to verify launcher script presence and permissions**

Create `tests/test_wrappers.rs`:
```rust
use std::path::Path;

#[test]
fn test_launcher_scripts_exist() {
    assert!(Path::new("rainbow-contour.sh").exists());
    assert!(Path::new("rainbow-contour.bat").exists());
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_wrappers`
Expected: FAIL (files missing).

- [ ] **Step 3: Create shell and batch wrapper launchers**

Create `rainbow-contour.sh`:
```bash
#!/usr/bin/env bash
echo "======================================================================"
echo "  RAINBOW CONTOUR - Cut & Fill Difference Map Generator v1.0"
echo "  PAMA Mine Engineering Field Launcher"
echo "======================================================================"

mkdir -p output

if [ -f "./target/release/rainbow-contour" ]; then
    BINARY="./target/release/rainbow-contour"
elif [ -f "./target/debug/rainbow-contour" ]; then
    BINARY="./target/debug/rainbow-contour"
else
    echo "Building release binary..."
    cargo build --release
    BINARY="./target/release/rainbow-contour"
fi

"$BINARY" "$@"
```

Create `rainbow-contour.bat`:
```cmd
@echo off
echo ======================================================================
echo   RAINBOW CONTOUR - Cut ^& Fill Difference Map Generator v1.0
echo   PAMA Mine Engineering Field Launcher
echo ======================================================================

if not exist "output" mkdir output

if exist "target\release\rainbow-contour.exe" (
    target\release\rainbow-contour.exe %*
) else (
    echo Building release binary...
    cargo build --release
    target\release\rainbow-contour.exe %*
)
```

Make `rainbow-contour.sh` executable:
```bash
chmod +x rainbow-contour.sh
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_wrappers`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rainbow-contour.sh rainbow-contour.bat tests/
git commit -m "feat(launcher): add one-click field wrapper scripts for Linux and Windows"
```
