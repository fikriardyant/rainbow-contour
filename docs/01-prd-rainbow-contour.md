# PRD: Rainbow Contour (Auto Cut & Fill Difference Map Generator)

- **Document ID**: PRD-01-RAINBOW-CONTOUR
- **Project**: Rainbow Contour (`rainbow-contour`)
- **Author**: Fikri Ardyantoro / Kuda
- **Date**: 2026-07-29
- **Status**: Approved / Draft

---

## 1. Executive Summary & Purpose

In open-pit mining operations (PAMA / Contractor workflows), mine engineers must compare high-density Topo survey data against monthly Pit Design models to generate **Cut & Fill elevation difference maps (Rainbow Contour)** and calculate volume differences ($m^3$). 

Currently, engineers rely on heavy commercial GIS/CAD software (Civil 3D, Surpac, Datamine), which is slow to launch, consumes high workstation memory, and is inaccessible on standard field laptops.

**Rainbow Contour** is a hybrid CLI + Standalone Web Visualizer tool that ingests 600-800 MB+ DXF files (Topo, Design, and Boundaries), performs high-speed grid elevation differencing ($\Delta Z = Z_{\text{design}} - Z_{\text{topo}}$) using a local multi-threaded Rust engine, and generates an interactive, zero-dependency HTML viewer complete with Neobrutalism UI, Cut/Fill volume tables, A4/A3 PDF reporting, and DXF contour line export.

---

## 2. Target Audience & Operational Value

* **Target Users**: Mine Engineers, Surveyors, Mining Contractors (PAMA Persada).
* **Operational Benefit**:
  * **Zero Cost / Zero Overhead**: Eliminates reliance on extra CAD licenses for quick map generation.
  * **High Throughput**: Processes 800 MB+ ASCII DXF files in seconds using all CPU cores (Rust Rayon + SIMD) with optional CUDA acceleration.
  * **Uniform Scale Multi-Boundary Comparison**: Processes multiple pit boundaries simultaneously using a single unified elevation gradient legend.

---

## 3. High-Level Requirements

### 3.1 Input Specifications
* **Topo DXF**: 3DFACE / TIN / Polyline mesh (up to 800 MB).
* **Design DXF**: Target pit surface design.
* **Boundary DXF**: Closed 2D/3D polylines defining pit boundaries.

### 3.2 Computational Requirements
* **Grid Resolution**: Configurable sampling grid (default 1m x 1m, slider 0.5m - 5m).
* **Delta Evaluation**: $\Delta Z = Z_{\text{design}} - Z_{\text{topo}}$.
  * $\Delta Z > 0$: **Fill** (Blue / Cyan / Green gradient)
  * $\Delta Z = 0$: **Match / Grade** (Yellow)
  * $\Delta Z < 0$: **Cut** (Orange / Red / Magenta gradient)
* **Volume Estimation**: Prism summation ($\sum \Delta Z \times \text{cell\_area}$) per boundary polygon.
* **Contour Isoline Generation**: Marching Squares algorithm to extract vector isolines for DXF export.

### 3.3 Output & Visualizer Requirements
* **Interactive Canvas**: HTML5 Canvas rendering elevation gradient + vector lines, pan/zoom support.
* **Volume Table**: Tabular breakdown of Cut ($m^3$), Fill ($m^3$), and Net ($m^3$) per boundary area.
* **PDF Export**: Instant A4/A3 Landscape PDF generation with standard mining header (Kop) and Legend.
* **DXF Export**: Generated DXF file with color-coded elevation layers.

---

## 4. Non-Functional Requirements

* **Performance**: Sub-5 second processing for 800 MB DXF on standard 8-core CPU workstation.
* **Portability**: Standalone native binary (`rainbow-contour` executable) requiring no Python environment or admin rights.
* **UI Style**: Clean Neobrutalism UI (`border-4 border-slate-900`, high contrast, crisp typography).
