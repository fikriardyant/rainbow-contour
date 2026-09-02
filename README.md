# 🌈 Rainbow Contour Engine

> High-performance Rust CLI & A4 Kop PDF generator for Cut & Fill Difference Maps in Mining Engineering.

![Rainbow Contour A4 Map Preview](./screenshots/preview.png)

---

## ⚡ Features

- **TIN Surface & 3D Polyline Meshing**: Native Delaunay 2D Triangulation with 3D $Z$-interpolation supporting `3DFACE`, `POLYLINE`/`VERTEX`, `LWPOLYLINE` (2D/3D with elevations), `LINE`, and `POINT` entities directly from CAD.
- **Seamless RLE Raster Heatmap Engine**: High-speed, lossless Run-Length Encoded raster bitmap renderer (< 500KB - 1.5MB HTML size) with zero memory lag and seamless anti-gap rendering.
- **Auto-Detect Pit Outer Boundary**: Automatically detects and clips grid deltas to the outermost closed crest boundary if no external boundary file is supplied.
- **Marching Squares Contours**: Generates vector isoline contours at $2\text{m}$ interval gradients ($\pm 20\text{m}$).
- **Cut & Fill Volume Calculation**: Computes exact Cut ($m^3$), Fill ($m^3$), and Net ($m^3$) volumes based on cell area.
- **Centralized `config.dat` Engine**: Reads and saves default project, survey, and calculation parameters directly from `config.dat`.
- **A4 Landscape Mine Plan Kop**: Official mine engineering layout with double technical border, coordinate grid frame (UTM & Geographic), North Arrow, multi-level color legend, and metadata block.
- **Direct PDF Export**: One-click high-resolution PDF download using `html2canvas-pro` + `jsPDF` (no browser print dialog required).
- **Vector DXF Export**: Exports color-coded vector isolines with native CAD ACI colors directly to `.dxf` format for Civil 3D / Surpac.

---

## ⚙️ Configuration (`config.dat`)

Seluruh parameter default, input file, kalkulasi, dan Kop peta diatur melalui file `config.dat` di root project. Anda dapat langsung mengedit file ini dengan text editor apapun:

```ini
# ======================================================================
# RAINBOW CONTOUR ENGINE CONFIGURATION (config.dat)
# Edit parameter di bawah untuk mengubah default input, kalkulasi & kop peta
# ======================================================================

# --- DEFAULT INPUT DXF (Biarkan kosong jika ingin ditanyakan saat dijalankan) ---
TOPO_PATH=
DESIGN_PATH=

# --- KOP & METADATA PETA ---
COMPANY_NAME=PT PAMA PERSADA NUSANTARA
RAINBOW_TITLE=PIT ALPHA CUT & FILL
DRAWN_BY=Fikri Ardyantoro
TOPO_DATE=28 July 2026
DESIGN_NAME=

# --- PARAMETER PERHITUNGAN GRID & SURFACE ---
GRID_STEP=1.00
MAX_TIN_EDGE=300.0
WEEDING_MIN_DIST=0.50
SUPPLEMENT_MAX_DIST=10.0

# --- ISOLINE CONTOUR LEVELS (Meter) ---
CONTOUR_LEVELS=-20.0,-18.0,-16.0,-14.0,-12.0,-10.0,-8.0,-6.0,-4.0,-2.0,0.0,2.0,4.0,6.0,8.0,10.0,12.0,14.0,16.0,18.0,20.0

# --- DEFAULT OUTPUT ---
DEFAULT_OUTDIR=./output
```

---

## 🚀 Quick Start & Tutorial

### 1. Build & Install
```bash
git clone https://github.com/fikriardyant/Rainbow-Contour.git
cd Rainbow-Contour
cargo build --release
```

### 2. Jalankan Mode Interaktif (Prompt CLI)
Cukup jalankan binary tanpa argumen. Sistem akan membaca default dari `config.dat`:
```bash
cargo run --release
```
Alur prompt interaktif:
1. `[1/3] Enter Topo DXF file path`: Masukkan path file Topo (contoh: `/path/to/topo.dxf`).
2. `[2/3] Enter Design DXF file path`: Masukkan path file Design (contoh: `/path/to/design.dxf`).
3. `[3/3] Enter Boundary DXF path (opt)`: Tekan `Enter` untuk auto-detect batas pit terluar dari file Design, atau masukkan path boundary jika ada.
4. `Enter Topo Survey Date [28 July 2026]`: Tekan `Enter` untuk memakai default `config.dat`.
5. `Enter Design Name [design_filename]`: Otomatis mengambil nama file design sebagai default. Tekan `Enter` untuk konfirmasi.

### 3. Jalankan Mode Otomatis / Direct CLI Flags (One-Liner)
Bisa langsung menjalankan pipeline dengan parameter penuh (akan meng-override nilai di `config.dat`):
```bash
cargo run --release -- \
  --topo /path/to/topo.dxf \
  --design /path/to/design.dxf \
  --company "PT PAMA PERSADA NUSANTARA" \
  --rainbow-title "PIT ALPHA CUT & FILL" \
  --drawn-by "Fikri Ardyantoro" \
  --topo-date "28 July 2026" \
  --design-name "Plan EOM July 2026" \
  --step 1.0 \
  --outdir "./output"
```

---

## 📸 Screenshots

| A4 Landscape Kop Map | Cut & Fill Volume Summary |
| :---: | :---: |
| ![A4 Kop Map](./screenshots/preview.png) | ![Volume Summary](./screenshots/summary.png) |

---

## 🎨 Legend Elevation Delta Scale ($\Delta Z$)

| Color | Delta Z Range | Category |
| :--- | :--- | :--- |
| 🟤 Dark Red | $> +16\text{m}$ | Heavy Cut |
| 🔴 Red | $+12\text{m} \text{ to } +16\text{m}$ | Cut |
| 🟠 Orange | $+4\text{m} \text{ to } +12\text{m}$ | Moderate Cut |
| 🟡 Yellow | $0\text{m} \text{ to } +4\text{m}$ | Minor Cut |
| 🟢 Green | $0\text{m}$ | **ON GRADE** |
| 🩵 Light Blue | $0\text{m} \text{ to } -4\text{m}$ | Minor Fill |
| 🔵 Blue | $-4\text{m} \text{ to } -12\text{m}$ | Moderate Fill |
| 🟣 Dark Purple | $< -16\text{m}$ | Heavy Fill |

---

## 📁 Output Deliverables

Setiap eksekusi menghasilkan 3 file siap pakai di folder output (`--outdir` atau `DEFAULT_OUTDIR`):
1. **`rainbow-viewer.html`** — Standalone A4 Landscape Mine Map Viewer dengan tombol 1-klik download PDF Kop.
2. **`rainbow-output.dxf`** — File vektor AutoCAD DXF multi-layer isoline kontur berwarna sesuai elevasi (kompatibel Civil 3D, Surpac, Minescape).
3. **`volume-summary.json`** — Ringkasan data kuantitatif volume Cut, Fill, Net, dan area cell dalam format JSON.

---

## 🛠️ Tech Stack

- **Engine**: Rust (Edition 2021)
- **TIN & Spatial**: Delaunator 2D Triangulation, Barycentric Z Interpolation, Marching Squares
- **Layout & PDF**: HTML5 Canvas, Tailwind CSS, `html2canvas-pro`, `jsPDF`
- **CAD Support**: 3DFACE, POLYLINE, VERTEX, LWPOLYLINE (2D/3D), LINE, POINT
