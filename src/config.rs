use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

pub fn format_system_time(time: SystemTime) -> String {
    let duration = match time.duration_since(UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return "1 January 1970".to_string(),
    };
    let total_secs = duration.as_secs();
    let days = (total_secs / 86400) as i64;

    // Civil day calculation from Unix epoch days (Howard Hinnant algorithm)
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };

    let month_idx = ((m - 1) as usize).min(11);
    format!("{} {} {}", d, MONTH_NAMES[month_idx], year)
}

pub fn get_current_date_string() -> String {
    format_system_time(SystemTime::now())
}

pub fn get_file_modified_date_string<P: AsRef<Path>>(path: P) -> String {
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            return format_system_time(modified);
        }
    }
    get_current_date_string()
}

pub fn open_file_in_default_browser<P: AsRef<Path>>(path: P) {
    let p = path.as_ref();
    let path_str = if let Ok(canonical) = p.canonicalize() {
        canonical.to_string_lossy().to_string()
    } else {
        p.to_string_lossy().to_string()
    };

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path_str])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&path_str)
            .spawn();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn();
    }
}

pub fn load_image_as_data_uri<P: AsRef<Path>>(path: P) -> Option<String> {
    let p = path.as_ref();
    if !p.exists() {
        return None;
    }
    let bytes = fs::read(p).ok()?;
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };

    // Standard base64 encoding without external crate dependency
    const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut b64 = String::with_capacity(bytes.len() * 4 / 3 + 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        b64.push(B64_CHARS[((n >> 18) & 63) as usize] as char);
        b64.push(B64_CHARS[((n >> 12) & 63) as usize] as char);

        if chunk.len() > 1 {
            b64.push(B64_CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            b64.push('=');
        }

        if chunk.len() > 2 {
            b64.push(B64_CHARS[(n & 63) as usize] as char);
        } else {
            b64.push('=');
        }
    }

    Some(format!("data:{};base64,{}", mime, b64))
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub topo_path: String,
    pub design_path: String,
    pub company_name: String,
    pub default_title: String,
    pub drawn_by: String,
    pub topo_date: String,
    pub design_name: String,
    pub company_logo_path: String,
    pub auto_open_browser: bool,
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
            topo_path: String::new(),
            design_path: String::new(),
            company_name: "PT PAMA PERSADA NUSANTARA".to_string(),
            default_title: "PIT ALPHA CUT & FILL".to_string(),
            drawn_by: "Fikri Ardyantoro".to_string(),
            topo_date: "28 July 2026".to_string(),
            design_name: String::new(),
            company_logo_path: "company_logo.png".to_string(),
            auto_open_browser: true,
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

        let topo_path = map.get("TOPO_PATH").or_else(|| map.get("TOPO")).cloned().unwrap_or(def.topo_path);
        let design_path = map.get("DESIGN_PATH").or_else(|| map.get("DESIGN")).cloned().unwrap_or(def.design_path);

        let company_name = map.get("COMPANY_NAME").or_else(|| map.get("COMPANY")).cloned().unwrap_or(def.company_name);
        let default_title = map.get("RAINBOW_TITLE").or_else(|| map.get("DEFAULT_TITLE")).or_else(|| map.get("TITLE")).cloned().unwrap_or(def.default_title);
        let drawn_by = map.get("DRAWN_BY").cloned().unwrap_or(def.drawn_by);
        let topo_date = map.get("TOPO_DATE").cloned().unwrap_or(def.topo_date);
        let design_name = map.get("DESIGN_NAME").cloned().unwrap_or(def.design_name);
        let company_logo_path = map.get("COMPANY_LOGO_PATH").or_else(|| map.get("COMPANY_LOGO")).or_else(|| map.get("LOGO")).cloned().unwrap_or(def.company_logo_path);
        let auto_open_browser = map.get("AUTO_OPEN_BROWSER").or_else(|| map.get("OPEN_BROWSER"))
            .map(|s| s.to_lowercase() != "false" && s != "0" && s.to_lowercase() != "no")
            .unwrap_or(def.auto_open_browser);

        let grid_step = map
            .get("GRID_STEP")
            .or_else(|| map.get("STEP"))
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

        let default_outdir = map.get("DEFAULT_OUTDIR").or_else(|| map.get("OUTDIR")).cloned().unwrap_or(def.default_outdir);

        Self {
            topo_path,
            design_path,
            company_name,
            default_title,
            drawn_by,
            topo_date,
            design_name,
            company_logo_path,
            auto_open_browser,
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
# Edit parameter di bawah untuk mengubah default input, kalkulasi & kop peta
# ======================================================================

# --- DEFAULT INPUT DXF (Biarkan kosong jika ingin ditanyakan saat dijalankan) ---
TOPO_PATH={}
DESIGN_PATH={}

# --- KOP & METADATA PETA ---
COMPANY_NAME={}
RAINBOW_TITLE={}
DRAWN_BY={}
TOPO_DATE={}
DESIGN_NAME={}
COMPANY_LOGO_PATH={}
AUTO_OPEN_BROWSER={}

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
            self.topo_path,
            self.design_path,
            self.company_name,
            self.default_title,
            self.drawn_by,
            self.topo_date,
            self.design_name,
            self.company_logo_path,
            self.auto_open_browser,
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
