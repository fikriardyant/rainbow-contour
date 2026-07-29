use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::volume::calculate_volume;

#[test]
fn test_volume_calculation() {
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 10.0, z_design: 15.0, delta_z: 5.0 },
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 12.0, z_design: 10.0, delta_z: -2.0 },
    ];

    let summary = calculate_volume(&grid, 1.0);
    assert_eq!(summary.fill_m3, 5.0);
    assert_eq!(summary.cut_m3, 2.0);
    assert_eq!(summary.net_m3, 3.0);
}
