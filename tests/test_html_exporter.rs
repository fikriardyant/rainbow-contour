use rainbow_contour::config::EngineConfig;
use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::html_exporter::{generate_html_viewer, generate_html_viewer_with_config, KopInfo};
use rainbow_contour::volume::VolumeSummary;

#[test]
fn test_generate_html_viewer_contains_neobrutalism_css() {
    let grid = vec![GridPointDelta {
        x: 0.0,
        y: 0.0,
        z_topo: 10.0,
        z_design: 15.0,
        delta_z: -5.0,
    }];
    let summary = VolumeSummary {
        cut_m3: 0.0,
        fill_m3: 5.0,
        net_m3: 5.0,
        cell_area_m2: 1.0,
        ongrade_area_m2: 0.0,
    };

    let kop = KopInfo {
        title: "Pit A July 2026",
        company: "PT MINING NUSANTARA PRIMA",
        drawn_by: "Mine Engineer",
        date_created: "29 July 2026",
        topo_date: "28 July 2026",
        design_name: "Plan EOM July 2026",
        logo_data_uri: None,
        ..Default::default()
    };
    let html = generate_html_viewer(&kop, &grid, &summary, &[]);
    assert!(html.contains("PETA RAINBOW CONTOUR"));
    assert!(html.contains("pdf-kop-container"));
    assert!(html.contains("html2canvas"));
    assert!(html.contains("jspdf"));
    assert!(html.contains("5.00 m³"));
}

#[test]
fn test_delta_z_to_color_id_15_bands() {
    use rainbow_contour::html_exporter::delta_z_to_color_id;
    let ongrade_min = -0.2;
    let ongrade_max = 0.2;

    // Cut bands (1..=7)
    assert_eq!(delta_z_to_color_id(18.0, ongrade_min, ongrade_max), 1); // > 16m
    assert_eq!(delta_z_to_color_id(14.0, ongrade_min, ongrade_max), 2); // 12..16m
    assert_eq!(delta_z_to_color_id(10.0, ongrade_min, ongrade_max), 3); // 8..12m
    assert_eq!(delta_z_to_color_id(6.0, ongrade_min, ongrade_max), 4);  // 4..8m
    assert_eq!(delta_z_to_color_id(3.0, ongrade_min, ongrade_max), 5);  // 2..4m
    assert_eq!(delta_z_to_color_id(1.5, ongrade_min, ongrade_max), 6);  // 1..2m (Yellow)
    assert_eq!(delta_z_to_color_id(0.5, ongrade_min, ongrade_max), 7);  // 0.2..1m (Lime Green)

    // Center / Level band (8)
    assert_eq!(delta_z_to_color_id(0.1, ongrade_min, ongrade_max), 8);  // -0.2..0.2m (Emerald Green)
    assert_eq!(delta_z_to_color_id(0.0, ongrade_min, ongrade_max), 8);
    assert_eq!(delta_z_to_color_id(-0.1, ongrade_min, ongrade_max), 8);

    // Fill bands (9..=15)
    assert_eq!(delta_z_to_color_id(-0.5, ongrade_min, ongrade_max), 9); // -1..-0.2m (Forest Green)
    assert_eq!(delta_z_to_color_id(-1.5, ongrade_min, ongrade_max), 10); // -2..-1m (Cyan)
    assert_eq!(delta_z_to_color_id(-3.0, ongrade_min, ongrade_max), 11); // -4..-2m
    assert_eq!(delta_z_to_color_id(-6.0, ongrade_min, ongrade_max), 12); // -8..-4m
    assert_eq!(delta_z_to_color_id(-10.0, ongrade_min, ongrade_max), 13); // -12..-8m
    assert_eq!(delta_z_to_color_id(-14.0, ongrade_min, ongrade_max), 14); // -16..-12m
    assert_eq!(delta_z_to_color_id(-20.0, ongrade_min, ongrade_max), 15); // < -16m
}

#[test]
fn test_html_exporter_custom_colors_and_ongrade() {
    let mut config = EngineConfig::default();
    config.ongrade_min = -0.5;
    config.ongrade_max = 0.5;
    config.color_ongrade = "#00FF66".to_string();
    config.color_cut_deep = "#7F0000".to_string();
    config.label_ongrade = "ON GRADE".to_string();

    let kop = KopInfo {
        title: "Pit A",
        company: "PT MINING NUSANTARA PRIMA",
        drawn_by: "Mine Engineer",
        date_created: "3 September 2026",
        topo_date: "3 September 2026",
        design_name: "Plan EOM",
        logo_data_uri: None,
        ..Default::default()
    };
    let summary = VolumeSummary {
        cut_m3: 100.0,
        fill_m3: 200.0,
        net_m3: 100.0,
        cell_area_m2: 1.0,
        ongrade_area_m2: 50.0,
    };

    let html = generate_html_viewer_with_config(&kop, &[], &summary, &[], &config);
    assert!(html.contains("ON GRADE"));
    assert!(html.contains("#00FF66"));
    assert!(html.contains("0.2-1m"));
    assert!(html.contains("1-2m"));
    assert!(html.contains("CUT (+)"));
    assert!(html.contains("FILL (-)"));
}

#[test]
fn test_denoise_raster_majority_eliminates_isolated_spikes() {
    use rainbow_contour::html_exporter::denoise_raster_majority;
    let cols = 5;
    let rows = 5;
    // 5x5 grid of all color 8 (Emerald Green), with 1 isolated color 10 (Cyan) at center (2, 2)
    let mut grid = vec![8u8; cols * rows];
    grid[2 * cols + 2] = 10; // outlier spike

    denoise_raster_majority(&mut grid, cols, rows);

    // Center pixel should be smoothed to dominant neighbor (8)
    assert_eq!(grid[2 * cols + 2], 8);
}

#[test]
fn test_html_exporter_contains_interactive_cad_hud_and_zoom() {
    let kop = KopInfo {
        title: "Pit A",
        company: "PT MINING NUSANTARA PRIMA",
        drawn_by: "Mine Engineer",
        date_created: "3 September 2026",
        topo_date: "3 September 2026",
        design_name: "Plan EOM",
        logo_data_uri: None,
        ..Default::default()
    };
    let summary = VolumeSummary {
        cut_m3: 0.0,
        fill_m3: 0.0,
        net_m3: 0.0,
        cell_area_m2: 1.0,
        ongrade_area_m2: 0.0,
    };
    let html = generate_html_viewer(&kop, &[], &summary, &[]);
    assert!(html.contains("cad-hud-controls"));
    assert!(html.contains("zoomIn()"));
    assert!(html.contains("zoomOut()"));
    assert!(html.contains("resetView()"));
    assert!(html.contains("zoomReadout"));
}

#[test]
fn test_html_exporter_compact_volume_summary() {
    let kop = KopInfo::default();
    let summary = VolumeSummary {
        cut_m3: 38253.75,
        fill_m3: 0.0,
        net_m3: -38253.75,
        cell_area_m2: 1.0,
        ongrade_area_m2: 0.0,
    };
    let html = generate_html_viewer(&kop, &[], &summary, &[]);
    // Verify compact volume summary layout (docked at bottom, no loose justify-around)
    assert!(html.contains("VOLUME SUMMARY"));
    assert!(html.contains("height: 92px; flex-shrink: 0;"));
    assert!(html.contains("38,253.75 m³"));
    assert!(html.contains("-38,253.75 m³"));
}


