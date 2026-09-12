use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::volume::{calculate_volume, calculate_volume_with_tolerance};

#[test]
fn test_volume_calculation_backward_compatible() {
    // Topo > Design = CUT (delta_z = +5.0)
    // Topo < Design = FILL (delta_z = -2.0)
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 15.0, z_design: 10.0, delta_z: 5.0 },
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 8.0, z_design: 10.0, delta_z: -2.0 },
    ];

    let summary = calculate_volume(&grid, 1.0);
    assert_eq!(summary.cut_m3, 5.0);
    assert_eq!(summary.fill_m3, 2.0);
    assert_eq!(summary.net_m3, 2.0 - 5.0);
}

#[test]
fn test_cut_fill_definition_and_ongrade() {
    // Topo = 100, Design = 90 -> Perlu digali 10m (CUT) -> delta_z = +10.0
    // Topo = 100, Design = 110 -> Perlu ditimbun 10m (FILL) -> delta_z = -10.0
    // Topo = 100, Design = 99.8 -> Selisih 0.2m (Cut tipis tapi masuk ONGRADE area [-0.5, 0.5])
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 100.0, z_design: 90.0, delta_z: 10.0 },   // CUT 10m
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 100.0, z_design: 110.0, delta_z: -10.0 }, // FILL 10m
        GridPointDelta { x: 2.0, y: 0.0, z_topo: 100.0, z_design: 99.8, delta_z: 0.2 },    // CUT 0.2m (Near grade)
    ];
    let vol = calculate_volume_with_tolerance(&grid, 1.0, -0.5, 0.5);
    // Gross Volume: 10.0 + 0.2 = 10.2 m³ (no volume dropped)
    assert!((vol.cut_m3 - 10.2).abs() < 1e-6);
    assert_eq!(vol.fill_m3, 10.0);
    assert_eq!(vol.ongrade_area_m2, 1.0);
}

#[test]
fn test_gross_volume_matches_cad_and_minescape_logic() {
    let grid = vec![
        GridPointDelta { x: 0.0, y: 0.0, z_topo: 100.3, z_design: 100.0, delta_z: 0.3 },   // Cut 0.3m
        GridPointDelta { x: 1.0, y: 0.0, z_topo: 99.8, z_design: 100.0, delta_z: -0.2 },   // Fill 0.2m
        GridPointDelta { x: 2.0, y: 0.0, z_topo: 105.0, z_design: 100.0, delta_z: 5.0 },   // Cut 5.0m
    ];
    let step = 1.0;
    let summary = calculate_volume(&grid, step);
    assert!((summary.cut_m3 - 5.3).abs() < 1e-6);
    assert!((summary.fill_m3 - 0.2).abs() < 1e-6);
    assert!((summary.net_m3 - (0.2 - 5.3)).abs() < 1e-6);

    let summary_tol = calculate_volume_with_tolerance(&grid, step, -0.2, 0.2);
    assert!((summary_tol.cut_m3 - 5.3).abs() < 1e-6);
    assert!((summary_tol.fill_m3 - 0.2).abs() < 1e-6);
    assert_eq!(summary_tol.ongrade_area_m2, 1.0); // 1 cell (-0.2) is in [-0.2, 0.2]
}
