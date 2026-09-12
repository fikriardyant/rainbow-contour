use std::fs;
use std::path::Path;
use rainbow_contour::config::EngineConfig;
use rainbow_contour::dxf::{parse_dxf_mesh_with_params, parse_dxf_styled_polylines, auto_detect_closed_boundary};
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::{generate_html_viewer_with_config, KopInfo};
use rainbow_contour::volume::calculate_volume_with_tolerance;

#[test]
fn test_bodydam_intan_minescape_parity_and_15_bands() {
    let topo_path = "trial_intan/TOPO_INTAN_WEEK0726_14022026.dxf";
    let design_path = "trial_intan/Design Bodydam Intan.dxf";

    if !Path::new(topo_path).exists() || !Path::new(design_path).exists() {
        println!("Bodydam Intan DXF files not found, skipping parity test.");
        return;
    }

    let topo_content = fs::read_to_string(topo_path).expect("read topo");
    let design_content = fs::read_to_string(design_path).expect("read design");

    let config = EngineConfig::default();

    let topo_mesh = parse_dxf_mesh_with_params(
        &topo_content,
        0.05,
        1.0,
        300.0,
    ).expect("parse topo");

    let design_mesh = parse_dxf_mesh_with_params(
        &design_content,
        0.05,
        1.0,
        300.0,
    ).expect("parse design");

    let design_lines = parse_dxf_styled_polylines(&design_content);
    let boundary = auto_detect_closed_boundary(&design_lines).unwrap_or_default();

    let step = 0.1;
    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, step);

    let volume = calculate_volume_with_tolerance(
        &grid,
        step,
        config.ongrade_min,
        config.ongrade_max,
    );

    println!("=== BODYDAM INTAN GROSS VOLUME ===");
    println!("CUT : {:.2} m³ (MineScape: 8,569 m³)", volume.cut_m3);
    println!("FILL: {:.2} m³ (MineScape: 10,431 m³)", volume.fill_m3);
    println!("Level [-0.2, +0.2] Area: {:.2} m²", volume.ongrade_area_m2);

    // Parity check vs MineScape (Cut 8,569 m³, Fill 10,431 m³) within 1% tolerance
    let minescape_cut = 8569.0;
    let minescape_fill = 10431.0;
    let cut_diff_pct = ((volume.cut_m3 - minescape_cut) / minescape_cut).abs() * 100.0;
    let fill_diff_pct = ((volume.fill_m3 - minescape_fill) / minescape_fill).abs() * 100.0;

    assert!(cut_diff_pct < 1.0, "Cut diff {:.2}% exceeds 1% tolerance", cut_diff_pct);
    assert!(fill_diff_pct < 1.0, "Fill diff {:.2}% exceeds 1% tolerance", fill_diff_pct);

    // Verify HTML viewer generation with 15-band palette
    let kop = KopInfo {
        title: "BODYDAM INTAN CUT & FILL",
        company: "PT MINING NUSANTARA PRIMA",
        drawn_by: "Mine Engineer",
        date_created: "12 September 2026",
        topo_date: "14 February 2026",
        design_name: "Design Bodydam Intan",
        logo_data_uri: None,
        ..Default::default()
    };

    let html = generate_html_viewer_with_config(&kop, &grid, &volume, &design_lines, &config);

    // Check volume summary numbers in HTML
    assert!(html.contains("8,579.88 m³"));
    assert!(html.contains("10,407.07 m³"));

    // Check 15-band legend entries
    assert!(html.contains("-0.2 - 0.2m")); // On Grade level center
    assert!(html.contains("0.2-1m"));      // Cut to Grade
    assert!(html.contains("-1 - -0.2m"));  // Fill to Grade
    assert!(html.contains("1-2m"));        // Cut Minor (Yellow)
    assert!(html.contains("-2 - -1m"));    // Fill Minor (Cyan)

    // Check palette colors
    assert!(html.contains("#76FF03"));     // Lime Green
    assert!(html.contains("#00E676"));     // Emerald Green
    assert!(html.contains("#1B5E20"));     // Dark Forest Green
    assert!(html.contains("#FFE600"));     // Yellow
    assert!(html.contains("#00E5FF"));     // Cyan
}
