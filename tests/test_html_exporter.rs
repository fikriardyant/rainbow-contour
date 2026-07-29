use rainbow_contour::grid_engine::GridPointDelta;
use rainbow_contour::html_exporter::{generate_html_viewer, KopInfo};
use rainbow_contour::volume::VolumeSummary;

#[test]
fn test_generate_html_viewer_contains_neobrutalism_css() {
    let grid = vec![GridPointDelta {
        x: 0.0,
        y: 0.0,
        z_topo: 10.0,
        z_design: 15.0,
        delta_z: 5.0,
    }];
    let summary = VolumeSummary {
        cut_m3: 0.0,
        fill_m3: 5.0,
        net_m3: 5.0,
        cell_area_m2: 1.0,
    };

    let kop = KopInfo {
        title: "Pit A July 2026",
        company: "PT PAMA PERSADA NUSANTARA",
        drawn_by: "Fikri Ardyantoro",
        date_created: "29 July 2026",
        topo_date: "28 July 2026",
        design_name: "Plan EOM July 2026",
    };
    let html = generate_html_viewer(&kop, &grid, &summary);
    assert!(html.contains("PETA RAINBOW CONTOUR"));
    assert!(html.contains("map-frame"));
    assert!(html.contains("html2canvas"));
    assert!(html.contains("jspdf"));
}
