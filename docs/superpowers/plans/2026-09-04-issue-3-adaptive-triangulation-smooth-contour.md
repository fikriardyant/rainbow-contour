# Implementation Plan - Issue #3: Fine Adaptive Surface Triangulation & Smooth Contour Isolines

**Goal:** Mengatasi issue #3 ("Triangle too big for small surface") dan tampilan patah-patah pada permukaan kecil dengan:
1. Menghubungkan parameter `max_tin_edge`, `weeding_min_dist`, dan `supplement_max_dist` dari `config.dat` ke `src/dxf.rs`.
2. Menerapkan adaptive densifikasi dan adaptive edge thresholding untuk surface berdimensi kecil agar kurva crest/toe rapat dan segitiga tidak menyeberang void.
3. Mengganti 1D window scanline di `src/marching_squares.rs` menjadi per-cell linear edge-crossing interpolation agar kontur mulus (tidak bergerigi/patah-patah siku).
4. Menyediakan adaptive grid step jika dimensi area sangat kecil tanpa mengubah layout Kop Surat atau logika boundary Issue #2.

---

### Task 1: Add parse_dxf_mesh_with_params & Adaptive Triangulation in `src/dxf.rs`
- Tambahkan `parse_dxf_mesh_with_params(content: &str, min_dist: f64, max_dist: f64, max_edge: f64)`.
- Jika bounding box polylines kecil (diagonal < 100m), lakukan adaptive downscale pada `supplement_dist` dan `max_edge`.
- Backward compatible `parse_dxf_mesh(content)` tetap memanggil fungsi baru dengan default.

### Task 2: Linear Edge-Crossing Interpolation in `src/marching_squares.rs`
- Buat interpolasi linear pada marching squares cell 2D sehingga koordinat $(x, y)$ isoline tepat berada di posisi proporsional $t = (level - z_a) / (z_b - z_a)$, bukan sekadar loncat antar titik grid.

### Task 3: Wire into `src/main.rs` & Regression Verification
- Hubungkan `config.max_tin_edge`, `config.weeding_min_dist`, dan `config.supplement_max_dist` ke pemanggilan `parse_dxf_mesh_with_params` di `src/main.rs`.
- Jalankan seluruh unit & integration test untuk memastikan seluruh gates green.
