# 🌈 Rainbow Contour Engine

> High-performance CLI & A4 Kop PDF generator for Cut & Fill Difference Maps in Mining Engineering.

![Rainbow Contour A4 Map Preview](./screenshots/preview.png)

---

## ⚡ Features

- **Double-Click Executable (`.bat` / `.sh`)**: Langsung jalankan tanpa perlu install Rust / Cargo di komputer user.
- **TIN Surface & 3D Polyline Meshing**: Native Delaunay 2D Triangulation dengan 3D $Z$-interpolation untuk `3DFACE`, `POLYLINE`/`VERTEX`, `LWPOLYLINE` (2D/3D elevations), `LINE`, dan `POINT` langsung dari CAD.
- **Seamless RLE Raster Heatmap Engine**: Lossless Run-Length Encoded raster bitmap viewer (< 500KB - 1.5MB) tanpa memory lag dan anti-bolong/celah pixel.
- **Auto-Detect Pit Outer Boundary**: Otomatis mendeteksi closed crest limit terluar dari file Design DXF jika tanpa boundary eksplisit.
- **Marching Squares Contours**: Menghasilkan garis kontur isoline vektor tiap gradien interval $2\text{m}$ ($\pm 20\text{m}$).
- **Cut & Fill Volume Calculation**: Kalkulasi akurat volume Cut ($m^3$), Fill ($m^3$), dan Net ($m^3$) berbasis cell area.
- **Centralized `config.dat` Engine**: Simpan & ubah parameter kalkulasi, default survey, dan Kop peta langsung dari file teks `config.dat`.
- **A4 Landscape Mine Plan Kop**: Standar layout engineering tambang dengan double border, frame koordinat UTM & Geografis, North Arrow, legenda multi-level, dan tabel volume.
- **Direct PDF Export**: Tombol 1-klik download PDF resolusi tinggi via `html2canvas-pro` + `jsPDF` tanpa popup print browser.
- **Vector DXF Export**: Ekspor garis kontur 3DPolyline native ACI colors langsung ke `.dxf` untuk AutoCAD / Civil 3D / Surpac.

---

## ⚙️ Konfigurasi (`config.dat`)

Seluruh parameter default, path file DXF, kalkulasi, dan Kop peta diatur melalui file `config.dat` di root folder:

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

## 🚀 Cara Menjalankan (Quick Start)

### 🪟 Pengguna Windows:
1. Download package / binary release `rainbow-contour`.
2. Klik ganda (Double-Click) file **`rainbow-contour.bat`**.
3. Masukkan path file DXF pada prompt interaktif yang muncul di terminal.

### 🐧 Pengguna Linux / macOS:
1. Buka terminal di folder project.
2. Jalankan launcher script:
   ```bash
   ./rainbow-contour.sh
   ```

### 💻 Developer Mode (Menggunakan Cargo):
Jika ingin meng-compile dari source:
```bash
cargo run --release
```

---

## 📝 Alur Prompt Interaktif CLI

Saat `rainbow-contour.bat` atau `./rainbow-contour.sh` dijalankan:
1. `[1/3] Enter Topo DXF file path`: Masukkan path file Topo (contoh: `D:/Survey/topo.dxf`).
2. `[2/3] Enter Design DXF file path`: Masukkan path file Design (contoh: `D:/Design/pit_south.dxf`).
3. `[3/3] Enter Boundary DXF path (opt)`: Tekan `Enter` untuk auto-detect batas pit terluar, atau ketik path file boundary jika ada.
4. `Enter Topo Survey Date [28 July 2026]`: Tekan `Enter` untuk memakai nilai default `config.dat`.
5. `Enter Design Name [pit_south]`: Otomatis mendeteksi nama file design sebagai default. Tekan `Enter` untuk konfirmasi.

> **Tips:** Jika `TOPO_PATH` dan `DESIGN_PATH` sudah diisi di `config.dat`, program akan langsung memproses otomatis tanpa memunculkan prompt input file!

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

Setiap eksekusi menghasilkan 3 file di folder output (`./output` atau `DEFAULT_OUTDIR`):
1. **`rainbow-viewer.html`** — A4 Landscape Kop Viewer interaktif dengan tombol 1-klik `DOWNLOAD PDF KOP`.
2. **`rainbow-output.dxf`** — Garis kontur vektor 3D AutoCAD DXF warna native ACI (langsung bisa dibuka di AutoCAD, Civil 3D, Surpac, Minescape).
3. **`volume-summary.json`** — Ringkasan data kuantitatif volume Cut, Fill, dan Net dalam format JSON.
