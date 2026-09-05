# Gates: Issue #3 Small Surface Adaptive Triangulation & Smooth Contour Generation

OWNS: src/dxf.rs, src/grid_engine.rs, src/marching_squares.rs, src/main.rs, tests/**

Scope: Fix oversized triangles on small designs (Issue #3) via configurable and adaptive TIN triangulation parameters, prevent void-spanning edges, implement true 2D Marching Squares with linear edge interpolation for smooth isolines, and apply adaptive grid sampling for small surfaces while leaving Kop surat and boundary logic strictly untouched.

- [x] G1: DXF parser accepts configurable parameters and applies adaptive densification for small designs
  CHECK: cargo test --test test_dxf_adaptive_triangulation test_dxf_adaptive_densification -- --exact
  EXPECT: test test_dxf_adaptive_densification ... ok

- [x] G2: Small surface triangulation generates fine, non-faceted triangles with bounded edge lengths
  CHECK: cargo test --test test_dxf_adaptive_triangulation test_small_surface_triangulation_bounds -- --exact
  EXPECT: test test_small_surface_triangulation_bounds ... ok

- [x] G3: Marching squares implements true 2D edge-crossing interpolation for smooth, non-jagged isolines
  CHECK: cargo test --test test_dxf_exporter test_marching_squares_linear_interpolation -- --exact
  EXPECT: test test_marching_squares_linear_interpolation ... ok

- [x] G4: Full test suite and regression checks pass without altering Kop surat or Issue #2 boundary logic
  CHECK: cargo test --bins --tests
  EXPECT: test test_launcher_scripts_exist ... ok
