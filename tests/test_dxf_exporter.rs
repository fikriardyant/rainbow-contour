use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::marching_squares::generate_isolines;

#[test]
fn test_dxf_exporter_generates_valid_dxf_content() {
    let grid = vec![
        GridPointDelta {
            x: 0.0,
            y: 0.0,
            z_topo: 10.0,
            z_design: 15.0,
            delta_z: 5.0,
        },
        GridPointDelta {
            x: 1.0,
            y: 0.0,
            z_topo: 10.0,
            z_design: 10.0,
            delta_z: 0.0,
        },
    ];

    let isolines = generate_isolines(&grid, 1.0, vec![0.0, 2.5, 5.0]);
    let dxf_str = export_isolines_to_dxf(&isolines);

    assert!(dxf_str.contains("SECTION"));
    assert!(dxf_str.contains("ENTITIES"));
    assert!(dxf_str.contains("ENDSEC"));
    assert!(dxf_str.contains("EOF"));
}

#[test]
fn test_marching_squares_linear_interpolation() {
    // 2x2 grid cell with diagonal gradient
    // (0,1): dz=10     (1,1): dz=10
    // (0,0): dz=0      (1,0): dz=0
    // Contour level = 5.0 should cross exactly at midpoints y = 0.5 on both vertical edges
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 0.0, z_design: 0.0, delta_z: 0.0 },
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 0.0, z_design: 0.0, delta_z: 0.0 },
        GridPointDelta { x: 0.0, y: 1.0, z_topo: 10.0, z_design: 0.0, delta_z: 10.0 },
        GridPointDelta { x: 1.0, y: 1.0, z_topo: 10.0, z_design: 0.0, delta_z: 10.0 },
    ];

    let isolines = generate_isolines(&grid, 1.0, vec![5.0]);
    assert!(!isolines.is_empty(), "Should generate at least one isoline segment");
    
    // Check that at least one segment crosses with linear interpolated y = 0.5
    let has_interpolated_point = isolines.iter().any(|seg| {
        (seg.p0.y - 0.5).abs() < 1e-4 || (seg.p1.y - 0.5).abs() < 1e-4
    });
    assert!(has_interpolated_point, "Isoline must be smoothly interpolated between points, not snapped to grid vertices");
}
