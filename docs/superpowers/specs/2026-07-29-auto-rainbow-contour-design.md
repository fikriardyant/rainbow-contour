# Design Spec: Auto Rainbow Contour Engine & Standalone Visualizer

**Date**: 2026-07-29  
**Author**: Kuda (Hermes Agent) & Fikri Ardyantoro  
**Status**: Approved / Draft  
**Target Path**: `docs/superpowers/specs/2026-07-29-auto-rainbow-contour-design.md`

---

## 1. Executive Summary & Overview

**Auto Rainbow Contour Engine** adalah tools tambang berkinerja tinggi untuk menghitung beda elevasi ($\Delta Z = Z_{\text{design}} - Z_{\text{topo}}$) serta mengalkulasi estimasi volume **Cut & Fill** ($m^3$) dari file DXF berukuran jumbo (600–800 MB+). 

Sistem ini didesain untuk dijalankan secara **portabel di lingkungan kantor tanpa butuh hak akses admin / install Python**, mendukung sistem operasi **Windows (.bat) & Linux (.sh)** melalui binary kompilasi **Rust** yang ringan dan super cepat, serta memproduksi laporan interaktif **Standalone HTML Viewer** (Neobrutalism UI) dan **Export DXF Rainbow Isoline**.

---

## 2. Goals & Non-Goals

### Goals
- **Skalabilitas & Performa**: Sanggup membaca file DXF Topo & Design hingga >800 MB / jutaan entitas dalam hitungan detik tanpa OOM (Out of Memory) atau freeze browser.
- **Portabilitas Lintas OS**: Berjalan lancar di Windows dan Linux menggunakan script wrapper (`.bat` / `.sh`) tanpa dependensi Python atau runtime eksternal.
- **Support Multiple Boundary**: Sanggup mengevaluasi beberapa polygon boundary sekaligus dalam 1 kali kalkulasi dengan skala gradien warna Cut/Fill seragam.
- **Dual Output Deliverables**:
  1. `rainbow_unified_report.html`: Standalone HTML viewer interaktif dengan visualisasi Heatmap/Contour, tabel ringkasan volume per boundary, dan fitur export PDF Kop A4/A3.
  2. `rainbow_unified_result.dxf`: File DXF baru berisikan garis kontur vektor isoline dengan warna layer menyesuaikan gradien delta elevasi.

### Non-Goals
- Tidak memerlukan server backend cloud atau koneksi internet (100% offline & local processing).
- Tidak menggantikan software CAD 3D full-suite (Surpac/Civil 3D) untuk pemodelan geologi kompleks, melainkan difokuskan untuk evaluasi progress Cut/Fill cepat.

---

## 3. System Architecture & Component Design

Sistem terbagi atas 3 komponen utama:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        USER WORKFLOW & INTERFACE                       │
│   Windows: run_rainbow.bat  │  Linux: ./run_rainbow.sh                 │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        RUST NATIVE CLI ENGINE                          │
│   Executable: rainbow-contour (.exe / ELF binary)                      │
│                                                                        │
│  ├── 1. Fast ASCII DXF Parser (Zero-copy parallel parsing via rayon)   │
│  ├── 2. Spatial KD-Tree Indexer (rstar / scipy-equivalent KD-Tree)     │
│  ├── 3. Grid Elevation Interpolator & Boundary Masker                  │
│  ├── 4. ΔZ Calculator (ΔZ = Z_design - Z_topo)                         │
│  ├── 5. Marching Squares Isoline Generator (Cut/Fill Color Mapping)    │
│  └── 6. Grid Prism Volume Integrator (Per-boundary & Total Combined)    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                  ┌─────────────────┴─────────────────┐
                  ▼                                   ▼
┌───────────────────────────────────┐   ┌───────────────────────────────────┐
│     STANDALONE HTML REPORT        │   │        EXPORTED VECTOR DXF        │
│   rainbow_unified_report.html     │   │     rainbow_unified_result.dxf     │
│  - Interactive Canvas / Plotly    │   │  - Color-coded Isoline Contours   │
│  - Neobrutalism Layout & Legend   │   │  - Boundary Polygon Layers        │
│  - Per-Boundary Volume Summary    │   │  - CAD Compatible Elevation Z     │
│  - A4/A3 Kop PDF Export           │   │                                   │
└───────────────────────────────────┘   └───────────────────────────────────┘
```

---

## 4. Input Requirements & Command Line Interface Flow

### 4.1 Input Files
1. **Topo DXF**: File DXF topografi aktual (Point, 3DFace, Polyline3D, Mesh).
2. **Design DXF**: File DXF rencana tambang / pit design / ramp design.
3. **Boundary DXF**: File DXF berisi satu atau beberapa Closed LWPOLYLINE / POLYLINE yang menentukan batas area perhitungan.

### 4.2 Interactive CLI Prompt Flow (`run_rainbow.bat` / `run_rainbow.sh`)
Ketika script di-run, CMD/Terminal menampilkan dialog interaktif:
1. User menginput path file Topo, Design, dan Boundary.
2. User memilih resolusi Grid Sampling (Default: 1.0 meter).
3. Engine Rust mengeksekusi komputasi dengan progress bar.
4. Terminal menyajikan tabel ringkasan Volume Cut ($m^3$), Fill ($m^3$), dan Net ($m^3$) per Boundary serta Total Combined.
5. Menghasilkan file output dan otomatis membuka `rainbow_unified_report.html` di browser default.

---

## 5. Mathematical & Algorithmic Specifications

### 5.0 Hardware Execution Strategy (CPU Multi-Threading & GPU Acceleration)
- **Default Engine (CPU SIMD + Rayon Multi-Core)**: Fast, 100% portable across any office PC without requiring NVIDIA GPU drivers. Text parsing & grid sampling are chunked and parallelized across all CPU cores. Processing time for 800 MB DXF is ~2-4 seconds.
- **Hardware Acceleration Option (`--gpu` / CUDA / wgpu)**: Optional flag allowing GPU-accelerated 2D grid matrix interpolation for ultra-dense grids (e.g., 0.1m step resolution with >100M grid points) utilizing NVIDIA GeForce / CUDA / DirectX12 compute pipelines when present, with seamless fallback to CPU if GPU driver is absent.

### 5.1 Spatial Grid Interpolation & Masking
* **Bounding Box Evaluation**: Dihitung dari polygon Boundary.
* **Ray-Casting Point-in-Polygon**: Memastikan titik grid $(x_i, y_j)$ berada di dalam salah satu polygon Boundary.
* **Elevation Lookup ($Z$)**: Menggunakan Triangulated Surface Linear Interpolation (atau KD-Tree Nearest Inverse Distance Weighting untuk Point Cloud) untuk mendapatkan $Z_{\text{topo}}(x,y)$ dan $Z_{\text{design}}(x,y)$.

### 5.2 Delta Elevation Calculation ($\Delta Z$)
$$\Delta Z = Z_{\text{design}} - Z_{\text{topo}}$$
* $\Delta Z > 0 \rightarrow$ **Fill Requirement** (Elevasi Design lebih tinggi dari Topo)
* $\Delta Z = 0 \rightarrow$ **On Grade / Match**
* $\Delta Z < 0 \rightarrow$ **Cut Requirement** (Elevasi Topo harus digali hingga mencapai Design)

### 5.3 Isoline Contour Generation (Marching Squares)
Garis kontur isoline di-generate menggunakan algoritma **Marching Squares** pada matrik 2D $\Delta Z$ dengan interval kontur yang dapat disesuaikan (misal tiap delta 0.5m atau 1.0m).

#### Uniform Color Palette Mapping:
- **Deep Cut ($\Delta Z \le -5.0\text{m}$)**: `#8B0000` (Dark Red / Magenta)
- **Moderate Cut ($-5.0\text{m} < \Delta Z \le -2.0\text{m}$)**: `#FF0000` (Red)
- **Minor Cut ($-2.0\text{m} < \Delta Z \le -0.5\text{m}$)**: `#FF7F00` (Orange)
- **On Grade / Match ($-0.5\text{m} < \Delta Z < +0.5\text{m}$)**: `#FFFF00` (Yellow)
- **Minor Fill ($+0.5\text{m} \le \Delta Z < +2.0\text{m}$)**: `#00FF00` (Bright Green)
- **Moderate Fill ($+2.0\text{m} \le \Delta Z < +5.0\text{m}$)**: `#008000` (Green)
- **Deep Fill ($\Delta Z \ge +5.0\text{m}$)**: `#0000FF` (Deep Blue)

### 5.4 Volume Integration (Grid Prism Method)
Setiap cell grid memiliki luas area $A = \text{step}_x \times \text{step}_y$ (misal $1.0\text{m} \times 1.0\text{m} = 1.0\text{m}^2$).
$$\text{Volume Cut} = \sum_{\Delta Z < 0} |\Delta Z_{i,j}| \times A$$
$$\text{Volume Fill} = \sum_{\Delta Z > 0} \Delta Z_{i,j} \times A$$
$$\text{Net Volume} = \text{Volume Cut} - \text{Volume Fill}$$

---

## 6. Standalone HTML Report UI/UX (`rainbow_unified_report.html`)

- **Design System**: Neobrutalism Style (Border `border-4 border-slate-900`, High-contrast card, Shadow `shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]`).
- **Interactive Visualizer**: 
  - Render Heatmap 2D dengan kontur isoline overlay.
  - Controls: Pan, Zoom, Reset View, Toggle Boundary Visibility, Filter Range $\Delta Z$.
  - Cursor Tooltip: Menampilkan Koordinat $(X, Y)$, $Z_{\text{topo}}$, $Z_{\text{design}}$, dan $\Delta Z$.
- **Volume Summary Table**: Card Neobrutalism yang menampilkan daftar Boundary, Cut Volume ($m^3$), Fill Volume ($m^3$), dan Net Volume ($m^3$).
- **Print & Export Capabilities**:
  - **Export PDF Kop Laporan A4/A3**: Format halaman laporan yang rapi dengan Kop Tambang, Legend, dan Tabel Summary.
  - **Download DXF**: Link langsung untuk men-download `rainbow_unified_result.dxf`.

---

## 7. Edge Cases & Error Handling

1. **File DXF Rusak / Unsupported Entities**: Engine Rust memfilter entity yang tidak relevan (TEXT, DIMENSION) dan hanya memproses geometry spatial (3DFACE, POINT, LWPOLYLINE).
2. **Boundary Terbuka (Unclosed Polyline)**: Engine akan otomatis menyambungkan titik pertama dan terakhir LWPOLYLINE jika selisih koordinat $< 0.01\text{m}$.
3. **Area Non-Overlapping**: Jika area Topo dan Design tidak saling beririsan pada titik tertentu di dalam Boundary, cell tersebut di-flag sebagai `Out of Bounds` dan tidak dihitung ke volume.
4. **Memory Constraint**: Penggunaan memori Rust dijaga $\le 500\text{ MB}$ RAM bahkan untuk DXF sebesar $800\text{ MB}$ dengan teknik streaming text reader + Rayon parallel iterator.

---

## 8. Verification & Test Plan

- **Unit Tests**: Test parsing DXF entity, test point-in-polygon ray casting, test Marching Squares isoline output.
- **Integration Tests**: Menguji run 3 file DXF sampel (Topo, Design, Boundary) dan membandingkan volume Cut/Fill yang dihasilkan dengan kalkulasi manual/Surpac.
- **Performance Benchmark**: Memastikan file 800 MB DXF dapat diproses dalam waktu $< 10$ detik di CPU Intel i5/i7 standar.
