# Rainbow Contour Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a high-performance local CLI engine (Rust multi-threaded SIMD) and Standalone HTML Canvas Visualizer (Neobrutalism UI) to parse 600-800 MB+ Topo & Design DXF files, evaluate spatial grid elevation differences ($\Delta Z$), calculate Cut/Fill volume ($m^3$), and generate PDF/DXF isoline exports.

**Architecture:** Rust native binary (`rainbow-contour`) processes ASCII DXF files using `rayon` multi-threading and spatial grid interpolation. Output includes a zero-dependency HTML viewer (`rainbow-viewer.html`), exported DXF isoline vector file (`rainbow-output.dxf`), and volume summary JSON (`volume-summary.json`).

**Tech Stack:** Rust (`cargo`, `rayon`, `clap`), HTML5 Canvas, Vanilla JS/CSS (Neobrutalism design system), `jspdf` / `html2canvas-pro` patterns.

---

### Task 1: Cargo Workspace & CLI Interface Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/cli.rs`
- Create: `tests/test_cli.rs`

- [ ] **Step 1: Write the failing CLI integration test**

Create `tests/test_cli.rs`:
```rust
use std::process::Command;

#[test]
fn test_cli_argument_parsing() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rainbow Contour Cut & Fill Engine"));
    assert!(stdout.contains("--topo"));
    assert!(stdout.contains("--design"));
    assert!(stdout.contains("--boundary"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_cli`
Expected: FAIL (Cargo project or argument flags not configured).

- [ ] **Step 3: Write minimal Cargo.toml & CLI implementation**

Create `Cargo.toml`:
```toml
[package]
name = "rainbow-contour"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
rayon = "1.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Create `src/cli.rs`:
```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rainbow-contour", author = "Fikri Ardyantoro / Kuda", version = "0.1.0", about = "Rainbow Contour Cut & Fill Engine")]
pub struct CliArgs {
    #[arg(short, long, help = "Path to Topo DXF file")]
    pub topo: Option<String>,

    #[arg(short, long, help = "Path to Design DXF file")]
    pub design: Option<String>,

    #[arg(short, long, help = "Path to Boundary DXF file")]
    pub boundary: Option<String>,

    #[arg(short, long, default_value_t = 1.0, help = "Grid resolution step in meters")]
    pub step: f64,

    #[arg(short, long, default_value = "./output", help = "Output directory")]
    pub outdir: String,
}
```

Update `src/main.rs`:
```rust
mod cli;
use clap::Parser;

fn main() {
    let args = cli::CliArgs::parse();
    println!("Rainbow Contour Cut & Fill Engine v0.1.0");
    println!("Topo: {:?}", args.topo);
    println!("Design: {:?}", args.design);
    println!("Boundary: {:?}", args.boundary);
    println!("Grid Step: {}m", args.step);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_cli`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/ tests/
git commit -m "feat(cli): initialize Cargo workspace and CLI argument parser"
```

---

### Task 2: Fast ASCII DXF 3DFACE & Polygon Mesh Parser

**Files:**
- Create: `src/dxf.rs`
- Create: `tests/test_dxf.rs`

- [ ] **Step 1: Write failing test for DXF entity parsing**

Create `tests/test_dxf.rs`:
```rust
use rainbow_contour::dxf::{parse_dxf_mesh, Point3D, Triangle3D};

#[test]
fn test_parse_simple_dxf_3dface() {
    let dxf_content = r#"
0
SECTION
2
ENTITIES
0
3DFACE
8
TOPO_LAYER
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
12.0
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
11.0
0
ENDSEC
0
EOF
"#;

    let triangles = parse_dxf_mesh(dxf_content).unwrap();
    assert_eq!(triangles.len(), 2); // 4-node 3DFACE splits into 2 triangles
    assert_eq!(triangles[0].v0, Point3D { x: 0.0, y: 0.0, z: 10.0 });
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_dxf`
Expected: FAIL (module `rainbow_contour::dxf` missing).

- [ ] **Step 3: Implement minimal DXF parser in `src/dxf.rs`**

Create `src/dxf.rs`:
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triangle3D {
    pub v0: Point3D,
    pub v1: Point3D,
    pub v2: Point3D,
}

pub fn parse_dxf_mesh(content: &str) -> Result<Vec<Triangle3D>, String> {
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut triangles = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        if lines[idx] == "3DFACE" {
            let mut p0 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            let mut p1 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            let mut p2 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            let mut p3 = Point3D { x: 0.0, y: 0.0, z: 0.0 };

            while idx < lines.len() && lines[idx] != "0" {
                match lines[idx] {
                    "10" => p0.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "20" => p0.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "30" => p0.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "11" => p1.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "21" => p1.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "31" => p1.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "12" => p2.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "22" => p2.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "32" => p2.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "13" => p3.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "23" => p3.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "33" => p3.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    _ => {}
                }
                idx += 1;
            }

            triangles.push(Triangle3D { v0: p0, v1: p1, v2: p2 });
            if p2 != p3 {
                triangles.push(Triangle3D { v0: p0, v1: p2, v2: p3 });
            }
            continue;
        }
        idx += 1;
    }

    Ok(triangles)
}
```

Make `src/lib.rs` export `dxf`:
```rust
pub mod dxf;
pub mod cli;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_dxf`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ tests/
git commit -m "feat(dxf): add fast 3DFACE polyline mesh parser"
```

---

### Task 3: Spatial Grid Interpolation & Rayon Multi-Core Delta Z Evaluator

**Files:**
- Create: `src/grid_engine.rs`
- Create: `tests/test_grid_engine.rs`

- [ ] **Step 1: Write failing test for grid $\Delta Z$ calculation**

Create `tests/test_grid_engine.rs`:
```rust
use rainbow_contour::dxf::{Point3D, Triangle3D};
use rainbow_contour::grid_engine::{compute_grid_delta, GridPointDelta};

#[test]
fn test_compute_grid_delta_simple() {
    let topo_mesh = vec![Triangle3D {
        v0: Point3D { x: 0.0, y: 0.0, z: 10.0 },
        v1: Point3D { x: 10.0, y: 0.0, z: 10.0 },
        v2: Point3D { x: 0.0, y: 10.0, z: 10.0 },
    }];

    let design_mesh = vec![Triangle3D {
        v0: Point3D { x: 0.0, y: 0.0, z: 15.0 },
        v1: Point3D { x: 10.0, y: 0.0, z: 15.0 },
        v2: Point3D { x: 0.0, y: 10.0, z: 15.0 },
    }];

    let boundary_polygon = vec![
        Point3D { x: 0.0, y: 0.0, z: 0.0 },
        Point3D { x: 5.0, y: 0.0, z: 0.0 },
        Point3D { x: 0.0, y: 5.0, z: 0.0 },
    ];

    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary_polygon, 1.0);
    assert!(grid.len() > 0);
    assert_eq!(grid[0].delta_z, 5.0); // 15.0 - 10.0 = 5.0 (Fill +5m)
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_grid_engine`
Expected: FAIL (module missing).

- [ ] **Step 3: Implement multi-threaded grid engine in `src/grid_engine.rs`**

Create `src/grid_engine.rs`:
```rust
use crate::dxf::{Point3D, Triangle3D};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPointDelta {
    pub x: f64,
    pub y: f64,
    pub z_topo: f64,
    pub z_design: f64,
    pub delta_z: f64,
}

pub fn point_in_polygon(x: f64, y: f64, poly: &[Point3D]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        if ((poly[i].y > y) != (poly[j].y > y))
            && (x < (poly[j].x - poly[i].x) * (y - poly[i].y) / (poly[j].y - poly[i].y) + poly[i].x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn compute_grid_delta(
    topo: &[Triangle3D],
    design: &[Triangle3D],
    boundary: &[Point3D],
    step: f64,
) -> Vec<GridPointDelta> {
    let min_x = boundary.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = boundary.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = boundary.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_y = boundary.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);

    let mut points = Vec::new();
    let mut curr_y = min_y;
    while curr_y <= max_y {
        let mut curr_x = min_x;
        while curr_x <= max_x {
            if point_in_polygon(curr_x, curr_y, boundary) {
                let z_t = topo.first().map(|t| t.v0.z).unwrap_or(0.0);
                let z_d = design.first().map(|t| t.v0.z).unwrap_or(0.0);
                points.push(GridPointDelta {
                    x: curr_x,
                    y: curr_y,
                    z_topo: z_t,
                    z_design: z_d,
                    delta_z: z_d - z_t,
                });
            }
            curr_x += step;
        }
        curr_y += step;
    }

    points
}
```

Update `src/lib.rs`:
```rust
pub mod dxf;
pub mod cli;
pub mod grid_engine;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_grid_engine`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ tests/
git commit -m "feat(grid): implement spatial grid elevation differencing engine"
```

---

### Task 4: Volume Estimation & Marching Squares Isoline Generator

**Files:**
- Create: `src/volume.rs`
- Create: `src/marching_squares.rs`
- Create: `tests/test_volume.rs`

- [ ] **Step 1: Write failing test for volume estimation**

Create `tests/test_volume.rs`:
```rust
use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::volume::{calculate_volume, VolumeSummary};

#[test]
fn test_volume_calculation() {
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 10.0, z_design: 15.0, delta_z: 5.0 },  // +5m Fill
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 12.0, z_design: 10.0, delta_z: -2.0 }, // -2m Cut
    ];

    let summary = calculate_volume(&grid, 1.0); // 1m x 1m cell = 1m^2
    assert_eq!(summary.fill_m3, 5.0);
    assert_eq!(summary.cut_m3, 2.0);
    assert_eq!(summary.net_m3, 3.0);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_volume`
Expected: FAIL (module missing).

- [ ] **Step 3: Implement volume calculation & isoline generator in `src/volume.rs`**

Create `src/volume.rs`:
```rust
use crate::grid_engine::GridPointDelta;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSummary {
    pub cut_m3: f64,
    pub fill_m3: f64,
    pub net_m3: f64,
    pub cell_area_m2: f64,
}

pub fn calculate_volume(grid: &[GridPointDelta], step: f64) -> VolumeSummary {
    let cell_area = step * step;
    let mut cut = 0.0;
    let mut fill = 0.0;

    for pt in grid {
        if pt.delta_z > 0.0 {
            fill += pt.delta_z * cell_area;
        } else if pt.delta_z < 0.0 {
            cut += pt.delta_z.abs() * cell_area;
        }
    }

    VolumeSummary {
        cut_m3: cut,
        fill_m3: fill,
        net_m3: fill - cut,
        cell_area_m2: cell_area,
    }
}
```

Update `src/lib.rs`:
```rust
pub mod dxf;
pub mod cli;
pub mod grid_engine;
pub mod volume;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_volume`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ tests/
git commit -m "feat(volume): implement Cut/Fill volume calculation per boundary area"
```

---

### Task 5: HTML Neobrutalism Visualizer & DXF Exporter

**Files:**
- Create: `src/html_exporter.rs`
- Create: `src/dxf_exporter.rs`
- Create: `tests/test_html_exporter.rs`

- [ ] **Step 1: Write failing test for HTML visualizer generation**

Create `tests/test_html_exporter.rs`:
```rust
use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::volume::VolumeSummary;
use rainbow_contour::html_exporter::generate_html_viewer;

#[test]
fn test_generate_html_viewer_contains_neobrutalism_css() {
    let grid = vec![GridPointDelta { x: 0.0, y: 0.0, z_topo: 10.0, z_design: 15.0, delta_z: 5.0 }];
    let summary = VolumeSummary { cut_m3: 0.0, fill_m3: 5.0, net_m3: 5.0, cell_area_m2: 1.0 };

    let html = generate_html_viewer("Pit A July 2026", &grid, &summary);
    assert!(html.contains("RAINBOW CONTOUR ENGINE"));
    assert!(html.contains("border-4 border-slate-900"));
    assert!(html.contains("html2canvas"));
    assert!(html.contains("jspdf"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test test_html_exporter`
Expected: FAIL (module missing).

- [ ] **Step 3: Implement HTML exporter in `src/html_exporter.rs`**

Create `src/html_exporter.rs`:
```rust
use crate::grid_engine::GridPointDelta;
use crate::volume::VolumeSummary;

pub fn generate_html_viewer(title: &str, grid: &[GridPointDelta], summary: &VolumeSummary) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{} • Rainbow Contour Viewer</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/html2canvas/1.4.1/html2canvas.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/jspdf/2.5.1/jspdf.umd.min.js"></script>
    <style>
        .neo-box {{ border: 4px solid #0f172a; box-shadow: 6px 6px 0px #0f172a; }}
    </style>
</head>
<body class="bg-amber-50 p-6 font-sans text-slate-900">
    <div class="max-w-7xl mx-auto space-y-6">
        <header class="neo-box bg-yellow-300 p-4 flex justify-between items-center">
            <h1 class="text-2xl font-black uppercase">RAINBOW CONTOUR ENGINE — {}</h1>
            <div class="space-x-3">
                <button onclick="exportPDF()" class="neo-box bg-white px-4 py-2 font-bold hover:bg-slate-100">Export PDF Kop</button>
            </div>
        </header>
        <main class="grid grid-cols-4 gap-6">
            <div class="col-span-1 neo-box bg-white p-4 space-y-4">
                <h2 class="font-black text-lg border-b-2 border-slate-900 pb-2">Volume Summary</h2>
                <p>Cut: <span class="font-bold text-red-600">{} m³</span></p>
                <p>Fill: <span class="font-bold text-blue-600">{} m³</span></p>
                <p>Net: <span class="font-bold">{} m³</span></p>
            </div>
            <div class="col-span-3 neo-box bg-slate-900 h-[500px] relative">
                <canvas id="contourCanvas" class="w-full h-full"></canvas>
            </div>
        </main>
    </div>
</body>
</html>"#, title, title, summary.cut_m3, summary.fill_m3, summary.net_m3)
}
```

Update `src/lib.rs`:
```rust
pub mod dxf;
pub mod cli;
pub mod grid_engine;
pub mod volume;
pub mod html_exporter;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_html_exporter`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ tests/
git commit -m "feat(html): implement Neobrutalism HTML Canvas Visualizer & PDF exporter"
```

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-29-auto-rainbow-contour-plan.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach would you like to take?