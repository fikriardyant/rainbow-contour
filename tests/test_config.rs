use rainbow_contour::config::EngineConfig;

#[test]
fn test_config_default_values() {
    let cfg = EngineConfig::default();
    assert_eq!(cfg.company_name, "PT PAMA PERSADA NUSANTARA");
    assert_eq!(cfg.default_title, "PIT ALPHA CUT & FILL");
    assert_eq!(cfg.drawn_by, "Fikri Ardyantoro");
    assert_eq!(cfg.topo_date, "28 July 2026");
    assert_eq!(cfg.grid_step, 0.5);
    assert_eq!(cfg.max_tin_edge, 300.0);
    assert_eq!(cfg.weeding_min_dist, 0.5);
    assert_eq!(cfg.supplement_max_dist, 10.0);
    assert_eq!(cfg.default_outdir, "./output");
    assert_eq!(cfg.contour_levels.len(), 21);
    assert_eq!(cfg.ongrade_min, -0.5);
    assert_eq!(cfg.ongrade_max, 0.5);
    assert_eq!(cfg.color_ongrade, "#00E676");
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
ONGRADE_MIN = -0.3
ONGRADE_MAX = 0.3
COLOR_ONGRADE = #00FF00
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
    assert_eq!(cfg.ongrade_min, -0.3);
    assert_eq!(cfg.ongrade_max, 0.3);
    assert_eq!(cfg.color_ongrade, "#00FF00");
}

#[test]
fn test_config_ongrade_and_custom_colors() {
    let custom_dat = r#"
ONGRADE_MIN=-0.5
ONGRADE_MAX=0.5
COLOR_ONGRADE=#00FF66
COLOR_CUT_DEEP=#7F0000
COLOR_CUT_HIGH=#FF0000
COLOR_CUT_MID=#FF6600
COLOR_CUT_LOW=#FFCC00
COLOR_CUT_NEAR=#FFEE55
COLOR_CUT_TO_GRADE=#FFFF99
COLOR_FILL_TO_GRADE=#99FFFF
COLOR_FILL_NEAR=#00FFFF
COLOR_FILL_LOW=#0099FF
COLOR_FILL_MID=#0033FF
COLOR_FILL_HIGH=#6600FF
COLOR_FILL_DEEP=#330066
"#;
    let config = EngineConfig::parse_content(custom_dat);
    assert_eq!(config.ongrade_min, -0.5);
    assert_eq!(config.ongrade_max, 0.5);
    assert_eq!(config.color_ongrade, "#00FF66");
    assert_eq!(config.color_cut_deep, "#7F0000");
    assert_eq!(config.color_fill_deep, "#330066");
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
    assert_eq!(cfg1.ongrade_min, cfg2.ongrade_min);
    assert_eq!(cfg1.ongrade_max, cfg2.ongrade_max);
    assert_eq!(cfg1.color_ongrade, cfg2.color_ongrade);
    assert_eq!(cfg1.color_cut_deep, cfg2.color_cut_deep);
}
