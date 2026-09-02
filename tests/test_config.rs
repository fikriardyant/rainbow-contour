use rainbow_contour::config::EngineConfig;

#[test]
fn test_config_default_values() {
    let cfg = EngineConfig::default();
    assert_eq!(cfg.company_name, "PT PAMA PERSADA NUSANTARA");
    assert_eq!(cfg.default_title, "PIT ALPHA CUT & FILL");
    assert_eq!(cfg.drawn_by, "Fikri Ardyantoro");
    assert_eq!(cfg.topo_date, "28 July 2026");
    assert_eq!(cfg.grid_step, 1.0);
    assert_eq!(cfg.max_tin_edge, 300.0);
    assert_eq!(cfg.weeding_min_dist, 0.5);
    assert_eq!(cfg.supplement_max_dist, 10.0);
    assert_eq!(cfg.default_outdir, "./output");
    assert_eq!(cfg.contour_levels.len(), 21);
}

#[test]
fn test_config_parse_custom_content() {
    let content = r#"
# Custom Mine Config
TOPO_PATH = /data/survey/topo.dxf
DESIGN_PATH = /data/design/pit_south.dxf
COMPANY_NAME = PT ADARO INDONESIA
RAINBOW_TITLE = PIT TUTUPAN NORTH
DRAWN_BY = Surveyor Team
TOPO_DATE = 15 August 2026
DESIGN_NAME = Plan EOM August 2026
GRID_STEP = 2.0
MAX_TIN_EDGE = 450.0
WEEDING_MIN_DIST = 1.0
SUPPLEMENT_MAX_DIST = 25.0
CONTOUR_LEVELS = -10.0, -5.0, 0.0, 5.0, 10.0
DEFAULT_OUTDIR = ./custom_output
"#;

    let cfg = EngineConfig::parse_content(content);
    assert_eq!(cfg.topo_path, "/data/survey/topo.dxf");
    assert_eq!(cfg.design_path, "/data/design/pit_south.dxf");
    assert_eq!(cfg.company_name, "PT ADARO INDONESIA");
    assert_eq!(cfg.default_title, "PIT TUTUPAN NORTH");
    assert_eq!(cfg.drawn_by, "Surveyor Team");
    assert_eq!(cfg.topo_date, "15 August 2026");
    assert_eq!(cfg.design_name, "Plan EOM August 2026");
    assert_eq!(cfg.grid_step, 2.0);
    assert_eq!(cfg.max_tin_edge, 450.0);
    assert_eq!(cfg.weeding_min_dist, 1.0);
    assert_eq!(cfg.supplement_max_dist, 25.0);
    assert_eq!(cfg.default_outdir, "./custom_output");
    assert_eq!(cfg.contour_levels, vec![-10.0, -5.0, 0.0, 5.0, 10.0]);
}

#[test]
fn test_config_to_dat_roundtrip() {
    let cfg1 = EngineConfig::default();
    let dat_str = cfg1.to_dat_string();
    let cfg2 = EngineConfig::parse_content(&dat_str);

    assert_eq!(cfg1.company_name, cfg2.company_name);
    assert_eq!(cfg1.default_title, cfg2.default_title);
    assert_eq!(cfg1.drawn_by, cfg2.drawn_by);
    assert_eq!(cfg1.topo_date, cfg2.topo_date);
    assert_eq!(cfg1.grid_step, cfg2.grid_step);
    assert_eq!(cfg1.max_tin_edge, cfg2.max_tin_edge);
    assert_eq!(cfg1.contour_levels, cfg2.contour_levels);
}
