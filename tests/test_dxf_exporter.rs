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
