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
        company: "PT PAMA PERSADA NUSANTARA",
        drawn_by: "Fikri Ardyantoro",
        date_created: "29 July 2026",
        topo_date: "28 July 2026",
        design_name: "Plan EOM July 2026",
        logo_data_uri: None,
    };
    let html = generate_html_viewer(&kop, &grid, &summary, &[]);
    assert!(html.contains("PETA RAINBOW CONTOUR"));
    assert!(html.contains("pdf-kop-container"));
    assert!(html.contains("html2canvas"));
    assert!(html.contains("jspdf"));
    assert!(html.contains("5.00 m³"));
}

#[test]
fn test_html_exporter_custom_colors_and_ongrade() {
    let mut config = EngineConfig::default();
    config.ongrade_min = -0.5;
    config.ongrade_max = 0.5;
    config.color_ongrade = "#00FF66".to_string();
    config.color_cut_deep = "#7F0000".to_string();

    let kop = KopInfo {
        title: "Pit A",
        company: "PT PAMA PERSADA NUSANTARA",
        drawn_by: "Fikri Ardyantoro",
        date_created: "3 September 2026",
        topo_date: "3 September 2026",
        design_name: "Plan EOM",
        logo_data_uri: None,
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
    assert!(html.contains("0-2m"));
}

#[test]
fn test_html_exporter_contains_interactive_cad_hud_and_zoom() {
    let kop = KopInfo {
        title: "Pit A",
        company: "PT PAMA PERSADA NUSANTARA",
        drawn_by: "Fikri Ardyantoro",
        date_created: "3 September 2026",
        topo_date: "3 September 2026",
        design_name: "Plan EOM",
        logo_data_uri: None,
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

