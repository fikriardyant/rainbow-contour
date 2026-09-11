# Implementation Plan: config.dat Configuration Engine & Dynamic Design Name

**Goal:**
1. Menyimpan dan membaca seluruh konfigurasi kalkulasi & Kop peta dari file `config.dat` di root project.
2. Default `Design Name` otomatis mengekstrak nama file design DXF yang diinput (file stem).

---

### Task 1: Create `src/config.rs` to Handle `config.dat`
- Key-Value parser format INI/DAT yang robust (support comments `#`, whitespace trimming, default value fallback).
- Auto-generate `config.dat` jika belum ada dengan nilai default yang persis sama dengan settingan saat ini:
  - `COMPANY_NAME=PT MINING NUSANTARA PRIMA`
  - `DEFAULT_TITLE=PIT A CUT & FILL MAP`
  - `DRAWN_BY=Fikri Ardyantoro`
  - `GRID_STEP=1.0`
  - `MAX_TIN_EDGE=300.0`
  - `WEEDING_MIN_DIST=0.5`
  - `SUPPLEMENT_MAX_DIST=10.0`
  - `CONTOUR_LEVELS=-20.0,-18.0,-16.0,-14.0,-12.0,-10.0,-8.0,-6.0,-4.0,-2.0,0.0,2.0,4.0,6.0,8.0,10.0,12.0,14.0,16.0,18.0,20.0`
  - `DEFAULT_OUTDIR=./output`

### Task 2: Update `src/main.rs` & `src/cli.rs`
- Integrasikan `config.rs` ke pipeline utama.
- Extract file stem dari `design_path` (misal `p2_scbd_r3cuwk33.dxf` -> `p2_scbd_r3cuwk33`) sebagai default prompt `Enter Design Name`.
- Gunakan nilai dari `config.dat` sebagai parameter kalkulasi (step, max_tin_edge, contour_levels) jika tidak di-override oleh CLI flags.

### Task 3: Testing & Verification
- Unit test `tests/test_config.rs` untuk parsing dan serialisasi `config.dat`.
- Test pipeline `tests/test_e2e_pipeline.rs`.
- Verifikasi `cargo test --tests`.
