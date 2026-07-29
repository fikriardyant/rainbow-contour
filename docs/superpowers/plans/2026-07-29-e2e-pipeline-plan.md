# End-to-End Pipeline Wiring & Sample DXF Test Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire `main.rs` to read Topo, Design, and Boundary DXF files, run spatial grid elevation differencing and volume calculations, and output `rainbow-viewer.html` and `volume-summary.json` to the target output directory. Create synthetic DXF test fixtures to verify end-to-end execution.

**Architecture:** `main.rs` uses `cli::CliArgs`, loads file content into memory, invokes `parse_dxf_mesh` on Topo and Design DXF files, parses Boundary 2D/3D polyline vertices, calls `compute_grid_delta`, calculates Cut/Fill volumes via `calculate_volume`, exports JSON & HTML artifacts, and verifies file generation.

---

### Task 1: Add Boundary DXF Polyline Parser & Pipeline Wiring in `src/dxf.rs` & `src/main.rs`

**Files:**
- Modify: `src/dxf.rs`
- Modify: `src/main.rs`
- Create: `tests/test_e2e_pipeline.rs`

- [ ] **Step 1: Write failing E2E integration test with synthetic DXF files**

Create `tests/test_e2e_pipeline.rs`:
```rust
use std::fs;
use std::process::Command;

#[test]
fn test_end_to_end_pipeline_with_synthetic_dxf() {
    let temp_dir = std::env::temp_dir().join("rainbow_test_e2e");
    let _ = fs::create_dir_all(&temp_dir);

    let topo_dxf = temp_dir.join("topo.dxf");
    let design_dxf = temp_dir.join("design.dxf");
    let boundary_dxf = temp_dir.join("boundary.dxf");
    let out_dir = temp_dir.join("output");

    let sample_topo = r#"0
SECTION
2
ENTITIES
0
3DFACE
10
0.0
20
0.0
30
10.0
11
10.0
21
0.0
31
10.0
12
10.0
22
10.0
32
10.0
13
0.0
23
10.0
33
10.0
0
ENDSEC
0
EOF"#;

    let sample_design = r#"0
SECTION
2
ENTITIES
0
3DFACE
10
0.0
20
0.0
30
15.0
11
10.0
21
0.0
31
15.0
12
10.0
22
10.0
32
15.0
13
0.0
23
10.0
33
15.0
0
ENDSEC
0
EOF"#;

    let sample_boundary = r#"0
SECTION
2
ENTITIES
0
POLYLINE
0
VERTEX
10
0.0
20
0.0
30
0.0
0
VERTEX
10
5.0
20
0.0
30
0.0
0
VERTEX
10
5.0
20
5.0
30
0.0
0
VERTEX
10
0.0
20
5.0
30
0.0
0
SEQEND
0
ENDSEC
0
EOF"#;

    fs::write(&topo_dxf, sample_topo).unwrap();
    fs::write(&design_dxf, sample_design).unwrap();
    fs::write(&boundary_dxf, sample_boundary).unwrap();

    let status = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--topo",
            topo_dxf.to_str().unwrap(),
            "--design",
            design_dxf.to_str().unwrap(),
            "--boundary",
            boundary_dxf.to_str().unwrap(),
            "--step",
            "1.0",
            "--outdir",
            out_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute cargo run");

    assert!(status.success());
    assert!(out_dir.join("rainbow-viewer.html").exists());
    assert!(out_dir.join("volume-summary.json").exists());
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_e2e_pipeline`
Expected: FAIL (Pipeline not yet wired in `main.rs`).

- [ ] **Step 3: Implement Boundary Polyline Parser & `main.rs` wiring**

Add boundary polyline parser in `src/dxf.rs`:
```rust
pub fn parse_dxf_boundary(content: &str) -> Result<Vec<Point3D>, String> {
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut points = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        if lines[idx] == "VERTEX" || lines[idx] == "LWPOLYLINE" {
            let mut p = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            while idx < lines.len() && lines[idx] != "0" {
                match lines[idx] {
                    "10" => p.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "20" => p.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "30" => p.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    _ => {}
                }
                idx += 1;
            }
            points.push(p);
            continue;
        }
        idx += 1;
    }

    if points.is_empty() {
        Ok(vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 100.0, y: 0.0, z: 0.0 },
            Point3D { x: 100.0, y: 100.0, z: 0.0 },
            Point3D { x: 0.0, y: 100.0, z: 0.0 },
        ])
    } else {
        Ok(points)
    }
}
```

Update `src/main.rs`:
```rust
mod cli;
use clap::Parser;
use rainbow_contour::dxf::{parse_dxf_boundary, parse_dxf_mesh};
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::generate_html_viewer;
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

    println!(
        "Computing spatial grid delta (step = {}m)...",
        args.step
    );
    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, args.step);
    let volume = calculate_volume(&grid, args.step);

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

    println!(
        "SUCCESS! Artifacts saved to {}",
        out_dir.display()
    );
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_e2e_pipeline`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ tests/
git commit -m "feat(pipeline): wire end-to-end DXF parsing, grid engine, volume calculation, and HTML/JSON exports"
```
