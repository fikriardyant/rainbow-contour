use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub company_name: String,
    pub default_title: String,
    pub drawn_by: String,
    pub grid_step: f64,
    pub max_tin_edge: f64,
    pub weeding_min_dist: f64,
    pub supplement_max_dist: f64,
    pub contour_levels: Vec<f64>,
    pub default_outdir: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            company_name: "PT PAMA PERSADA NUSANTARA".to_string(),
            default_title: "PIT A CUT & FILL MAP".to_string(),
            drawn_by: "Fikri Ardyantoro".to_string(),
            grid_step: 1.0,
            max_tin_edge: 300.0,
            weeding_min_dist: 0.5,
            supplement_max_dist: 10.0,
            contour_levels: vec![
                -20.0, -18.0, -16.0, -14.0, -12.0, -10.0, -8.0, -6.0, -4.0, -2.0,
                0.0,
                2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0,
            ],
            default_outdir: "./output".to_string(),
        }
    }
}

impl EngineConfig {
    pub fn parse_content(content: &str) -> Self {
        let mut map: HashMap<String, String> = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }
            if let Some((key, val)) = trimmed.split_once('=') {
                let k = key.trim().to_uppercase();
                let v = val.trim().trim_matches('\'').trim_matches('"').to_string();
                map.insert(k, v);
            }
        }

        let def = Self::default();

        let company_name = map.get("COMPANY_NAME").cloned().unwrap_or(def.company_name);
        let default_title = map.get("DEFAULT_TITLE").or_else(|| map.get("RAINBOW_TITLE")).cloned().unwrap_or(def.default_title);
        let drawn_by = map.get("DRAWN_BY").cloned().unwrap_or(def.drawn_by);

        let grid_step = map
            .get("GRID_STEP")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.grid_step);

        let max_tin_edge = map
            .get("MAX_TIN_EDGE")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.max_tin_edge);

        let weeding_min_dist = map
            .get("WEEDING_MIN_DIST")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.weeding_min_dist);

        let supplement_max_dist = map
            .get("SUPPLEMENT_MAX_DIST")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.supplement_max_dist);

        let contour_levels = if let Some(levels_str) = map.get("CONTOUR_LEVELS") {
            let parsed: Vec<f64> = levels_str
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();
            if !parsed.is_empty() {
                parsed
            } else {
                def.contour_levels
            }
        } else {
            def.contour_levels
        };

        let default_outdir = map.get("DEFAULT_OUTDIR").cloned().unwrap_or(def.default_outdir);

        Self {
            company_name,
            default_title,
            drawn_by,
            grid_step,
            max_tin_edge,
            weeding_min_dist,
            supplement_max_dist,
            contour_levels,
            default_outdir,
        }
    }

    pub fn to_dat_string(&self) -> String {
        let levels_str = self
            .contour_levels
            .iter()
            .map(|l| format!("{:.1}", l))
            .collect::<Vec<String>>()
            .join(",");

        format!(
            r#"# ======================================================================
# RAINBOW CONTOUR ENGINE CONFIGURATION (config.dat)
# Edit parameter di bawah untuk mengubah default kalkulasi & kop peta
# ======================================================================

# --- KOP & METADATA PETA ---
COMPANY_NAME={}
DEFAULT_TITLE={}
DRAWN_BY={}

# --- PARAMETER PERHITUNGAN GRID & SURFACE ---
GRID_STEP={:.2}
MAX_TIN_EDGE={:.1}
WEEDING_MIN_DIST={:.2}
SUPPLEMENT_MAX_DIST={:.1}

# --- ISOLINE CONTOUR LEVELS (Meter) ---
CONTOUR_LEVELS={}

# --- DEFAULT OUTPUT ---
DEFAULT_OUTDIR={}
"#,
            self.company_name,
            self.default_title,
            self.drawn_by,
            self.grid_step,
            self.max_tin_edge,
            self.weeding_min_dist,
            self.supplement_max_dist,
            levels_str,
            self.default_outdir
        )
    }

    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref();
        if p.exists() {
            if let Ok(content) = fs::read_to_string(p) {
                return Self::parse_content(&content);
            }
        }

        let def = Self::default();
        let _ = fs::write(p, def.to_dat_string());
        def
    }
}
