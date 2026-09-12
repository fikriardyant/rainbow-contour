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

pub fn hex_to_rgba(hex: &str) -> [u8; 4] {
    let cleaned = hex.trim().trim_start_matches('#');
    if cleaned.len() == 6 {
        let r = u8::from_str_radix(&cleaned[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&cleaned[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
        [r, g, b, 255]
    } else if cleaned.len() == 8 {
        let r = u8::from_str_radix(&cleaned[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&cleaned[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
        let a = u8::from_str_radix(&cleaned[6..8], 16).unwrap_or(255);
        [r, g, b, a]
    } else {
        [0, 0, 0, 255]
    }
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

    // Status Wizard Setup
    pub first_run: bool,

    // Identitas Unit / Distrik / Departemen
    pub district_name: String,
    pub department_name: String,

    // Metadata & Validasi Peta
    pub project_name: String,
    pub map_subtitle: String,
    pub reviewed_by: String,
    pub approved_by: String,
    pub coordinate_system: String,

    // Grid Anotasi
    pub grid_interval: f64,
    pub subtick_interval: f64,

    // Toleransi Ongrade
    pub ongrade_min: f64,
    pub ongrade_max: f64,

    // Palet Warna Kontur & Heatmap (HEX)
    pub color_cut_deep: String,      // > 16m
    pub color_cut_high: String,      // 12..16m
    pub color_cut_mid: String,       // 8..12m
    pub color_cut_low: String,       // 4..8m
    pub color_cut_near: String,      // 2..4m
    pub color_cut_minor: String,     // 1..2m
    pub color_cut_to_grade: String,  // 0.2..1m
    pub color_ongrade: String,       // -0.2..0.2m
    pub color_fill_to_grade: String, // -1..-0.2m
    pub color_fill_minor: String,    // -2..-1m
    pub color_fill_near: String,     // -4..-2m
    pub color_fill_low: String,      // -8..-4m
    pub color_fill_mid: String,      // -12..-8m
    pub color_fill_high: String,     // -16..-12m
    pub color_fill_deep: String,     // < -16m

    // Label Teks Legenda
    pub label_cut_deep: String,
    pub label_cut_high: String,
    pub label_cut_mid: String,
    pub label_cut_low: String,
    pub label_cut_near: String,
    pub label_cut_minor: String,
    pub label_cut_to_grade: String,
    pub label_ongrade: String,
    pub label_fill_to_grade: String,
    pub label_fill_minor: String,
    pub label_fill_near: String,
    pub label_fill_low: String,
    pub label_fill_mid: String,
    pub label_fill_high: String,
    pub label_fill_deep: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            topo_path: String::new(),
            design_path: String::new(),
            company_name: "PT MINING NUSANTARA PRIMA".to_string(),
            default_title: "PIT ALPHA CUT & FILL".to_string(),
            drawn_by: "Mine Engineer".to_string(),
            topo_date: "28 July 2026".to_string(),
            design_name: String::new(),
            company_logo_path: "company_logo.png".to_string(),
            auto_open_browser: true,
            grid_step: 0.5,
            max_tin_edge: 300.0,
            weeding_min_dist: 0.5,
            supplement_max_dist: 10.0,
            contour_levels: vec![
                -20.0, -18.0, -16.0, -14.0, -12.0, -10.0, -8.0, -6.0, -4.0, -2.0,
                0.0,
                2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0,
            ],
            default_outdir: "./output".to_string(),

            // Default First-Run & Kop Metadata
            first_run: false,
            district_name: "DISTRIK NUSANTARA".to_string(),
            department_name: "ENGINEERING & MINE PLANNING DEPT.".to_string(),
            project_name: "PIT ALPHA".to_string(),
            map_subtitle: "ISOPACH DIFFERENCE (TOPO - DESIGN)".to_string(),
            reviewed_by: "Reviewer".to_string(),
            approved_by: "Approver".to_string(),
            coordinate_system: "UTM ZONE 50S (WGS84)".to_string(),
            grid_interval: 200.0,
            subtick_interval: 50.0,

            // Default ongrade tolerance [-0.2, +0.2]
            ongrade_min: -0.2,
            ongrade_max: 0.2,

            // High-Contrast Vivid 15-Band Palette
            color_cut_deep: "#7F0000".to_string(),
            color_cut_high: "#B71C1C".to_string(),
            color_cut_mid: "#D50000".to_string(),
            color_cut_low: "#FF3D00".to_string(),
            color_cut_near: "#FF9100".to_string(),
            color_cut_minor: "#FFE600".to_string(),
            color_cut_to_grade: "#76FF03".to_string(),
            color_ongrade: "#00E676".to_string(),
            color_fill_to_grade: "#00BFA5".to_string(),
            color_fill_minor: "#00ACC1".to_string(),
            color_fill_near: "#00B0FF".to_string(),
            color_fill_low: "#2979FF".to_string(),
            color_fill_mid: "#0039CB".to_string(),
            color_fill_high: "#4A148C".to_string(),
            color_fill_deep: "#311B92".to_string(),

            // Default Clean CAD Legend Labels
            label_cut_deep: "> 16m".to_string(),
            label_cut_high: "12-16m".to_string(),
            label_cut_mid: "8-12m".to_string(),
            label_cut_low: "4-8m".to_string(),
            label_cut_near: "2-4m".to_string(),
            label_cut_minor: "1-2m".to_string(),
            label_cut_to_grade: "0.2-1m".to_string(),
            label_ongrade: "±0.2m".to_string(),
            label_fill_to_grade: "0.2-1m".to_string(),
            label_fill_minor: "1-2m".to_string(),
            label_fill_near: "2-4m".to_string(),
            label_fill_low: "4-8m".to_string(),
            label_fill_mid: "8-12m".to_string(),
            label_fill_high: "12-16m".to_string(),
            label_fill_deep: "> 16m".to_string(),
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

        let ongrade_min = map
            .get("ONGRADE_MIN")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.ongrade_min);

        let ongrade_max = map
            .get("ONGRADE_MAX")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.ongrade_max);

        let color_cut_deep = map.get("COLOR_CUT_DEEP").cloned().unwrap_or(def.color_cut_deep);
        let color_cut_high = map.get("COLOR_CUT_HIGH").cloned().unwrap_or(def.color_cut_high);
        let color_cut_mid = map.get("COLOR_CUT_MID").cloned().unwrap_or(def.color_cut_mid);
        let color_cut_low = map.get("COLOR_CUT_LOW").cloned().unwrap_or(def.color_cut_low);
        let color_cut_near = map.get("COLOR_CUT_NEAR").cloned().unwrap_or(def.color_cut_near);
        let color_cut_minor = map.get("COLOR_CUT_MINOR").cloned().unwrap_or(def.color_cut_minor);
        let color_cut_to_grade = map.get("COLOR_CUT_TO_GRADE").cloned().unwrap_or(def.color_cut_to_grade);
        let color_ongrade = map.get("COLOR_ONGRADE").cloned().unwrap_or(def.color_ongrade);
        let color_fill_to_grade = map.get("COLOR_FILL_TO_GRADE").cloned().unwrap_or(def.color_fill_to_grade);
        let color_fill_minor = map.get("COLOR_FILL_MINOR").cloned().unwrap_or(def.color_fill_minor);
        let color_fill_near = map.get("COLOR_FILL_NEAR").cloned().unwrap_or(def.color_fill_near);
        let color_fill_low = map.get("COLOR_FILL_LOW").cloned().unwrap_or(def.color_fill_low);
        let color_fill_mid = map.get("COLOR_FILL_MID").cloned().unwrap_or(def.color_fill_mid);
        let color_fill_high = map.get("COLOR_FILL_HIGH").cloned().unwrap_or(def.color_fill_high);
        let color_fill_deep = map.get("COLOR_FILL_DEEP").cloned().unwrap_or(def.color_fill_deep);

        let label_cut_deep = map.get("LABEL_CUT_DEEP").cloned().unwrap_or(def.label_cut_deep);
        let label_cut_high = map.get("LABEL_CUT_HIGH").cloned().unwrap_or(def.label_cut_high);
        let label_cut_mid = map.get("LABEL_CUT_MID").cloned().unwrap_or(def.label_cut_mid);
        let label_cut_low = map.get("LABEL_CUT_LOW").cloned().unwrap_or(def.label_cut_low);
        let label_cut_near = map.get("LABEL_CUT_NEAR").cloned().unwrap_or(def.label_cut_near);
        let label_cut_minor = map.get("LABEL_CUT_MINOR").cloned().unwrap_or(def.label_cut_minor);
        let label_cut_to_grade = map.get("LABEL_CUT_TO_GRADE").cloned().unwrap_or(def.label_cut_to_grade);
        let label_ongrade = map.get("LABEL_ONGRADE").cloned().unwrap_or(def.label_ongrade);
        let label_fill_to_grade = map.get("LABEL_FILL_TO_GRADE").cloned().unwrap_or(def.label_fill_to_grade);
        let label_fill_minor = map.get("LABEL_FILL_MINOR").cloned().unwrap_or(def.label_fill_minor);
        let label_fill_near = map.get("LABEL_FILL_NEAR").cloned().unwrap_or(def.label_fill_near);
        let label_fill_low = map.get("LABEL_FILL_LOW").cloned().unwrap_or(def.label_fill_low);
        let label_fill_mid = map.get("LABEL_FILL_MID").cloned().unwrap_or(def.label_fill_mid);
        let label_fill_high = map.get("LABEL_FILL_HIGH").cloned().unwrap_or(def.label_fill_high);
        let label_fill_deep = map.get("LABEL_FILL_DEEP").cloned().unwrap_or(def.label_fill_deep);

        let first_run = map.get("FIRST_RUN")
            .map(|s| {
                let sl = s.to_lowercase();
                sl == "y" || sl == "yes" || sl == "true" || sl == "1"
            })
            .unwrap_or(def.first_run);

        let district_name = map.get("DISTRICT_NAME")
            .or_else(|| map.get("DISTRIK"))
            .or_else(|| map.get("DISTRICT"))
            .cloned()
            .unwrap_or(def.district_name);

        let department_name = map.get("DEPARTMENT_NAME")
            .or_else(|| map.get("DEPT"))
            .or_else(|| map.get("DEPARTMENT"))
            .cloned()
            .unwrap_or(def.department_name);

        let project_name = map.get("PROJECT_NAME")
            .or_else(|| map.get("PROJECT"))
            .cloned()
            .unwrap_or(def.project_name);

        let map_subtitle = map.get("MAP_SUBTITLE")
            .or_else(|| map.get("SUBTITLE"))
            .cloned()
            .unwrap_or(def.map_subtitle);

        let reviewed_by = map.get("REVIEWED_BY")
            .or_else(|| map.get("CHECKED_BY"))
            .or_else(|| map.get("REVIEWER"))
            .or_else(|| map.get("CHECKED"))
            .cloned()
            .unwrap_or(def.reviewed_by);

        let approved_by = map.get("APPROVED_BY")
            .or_else(|| map.get("APPROVER"))
            .or_else(|| map.get("APPROVED"))
            .cloned()
            .unwrap_or(def.approved_by);

        let coordinate_system = map.get("COORDINATE_SYSTEM")
            .or_else(|| map.get("PROJECTION"))
            .or_else(|| map.get("CRS"))
            .cloned()
            .unwrap_or(def.coordinate_system);

        let grid_interval = map.get("GRID_INTERVAL")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.grid_interval);

        let subtick_interval = map.get("SUBTICK_INTERVAL")
            .and_then(|s| s.parse().ok())
            .unwrap_or(def.subtick_interval);

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
            first_run,
            district_name,
            department_name,
            project_name,
            map_subtitle,
            reviewed_by,
            approved_by,
            coordinate_system,
            grid_interval,
            subtick_interval,
            ongrade_min,
            ongrade_max,
            color_cut_deep,
            color_cut_high,
            color_cut_mid,
            color_cut_low,
            color_cut_near,
            color_cut_minor,
            color_cut_to_grade,
            color_ongrade,
            color_fill_to_grade,
            color_fill_minor,
            color_fill_near,
            color_fill_low,
            color_fill_mid,
            color_fill_high,
            color_fill_deep,
            label_cut_deep,
            label_cut_high,
            label_cut_mid,
            label_cut_low,
            label_cut_near,
            label_cut_minor,
            label_cut_to_grade,
            label_ongrade,
            label_fill_to_grade,
            label_fill_minor,
            label_fill_near,
            label_fill_low,
            label_fill_mid,
            label_fill_high,
            label_fill_deep,
        }
    }

    pub fn to_dat_string(&self) -> String {
        let levels_str = self
            .contour_levels
            .iter()
            .map(|l| format!("{:.1}", l))
            .collect::<Vec<String>>()
            .join(",");

        let first_run_str = if self.first_run { "y" } else { "n" };

        format!(
            r#"# ======================================================================
# RAINBOW CONTOUR ENGINE CONFIGURATION (config.dat)
# Edit parameter di bawah untuk mengubah default input, kalkulasi & kop peta
# ======================================================================

# --- WIZARD SETUP STATUS (n = tanya setup saat run pertama, y = setup selesai) ---
FIRST_RUN={}

# --- IDENTITAS PERUSAHAAN & SITE ---
COMPANY_NAME={}
DISTRICT_NAME={}
DEPARTMENT_NAME={}
COMPANY_LOGO_PATH={}

# --- METADATA & VALIDASI PETA (Civil 3D) ---
PROJECT_NAME={}
RAINBOW_TITLE={}
MAP_SUBTITLE={}
DRAWN_BY={}
REVIEWED_BY={}
APPROVED_BY={}
COORDINATE_SYSTEM={}
TOPO_DATE={}
DESIGN_NAME={}
AUTO_OPEN_BROWSER={}

# --- DEFAULT INPUT DXF (Biarkan kosong jika ingin ditanyakan saat dijalankan) ---
TOPO_PATH={}
DESIGN_PATH={}

# --- GRID ANOTASI PETA (Meter) ---
GRID_INTERVAL={:.1}
SUBTICK_INTERVAL={:.1}

# --- PARAMETER PERHITUNGAN GRID & SURFACE ---
GRID_STEP={:.2}
MAX_TIN_EDGE={:.1}
WEEDING_MIN_DIST={:.2}
SUPPLEMENT_MAX_DIST={:.1}

# --- TOLERANSI ONGRADE (Meter) ---
ONGRADE_MIN={:.2}
ONGRADE_MAX={:.2}

# --- LABEL LEGENDA CUT / FILL / ONGRADE ---
LABEL_CUT_DEEP={}
LABEL_CUT_HIGH={}
LABEL_CUT_MID={}
LABEL_CUT_LOW={}
LABEL_CUT_NEAR={}
LABEL_CUT_MINOR={}
LABEL_CUT_TO_GRADE={}
LABEL_ONGRADE={}
LABEL_FILL_TO_GRADE={}
LABEL_FILL_MINOR={}
LABEL_FILL_NEAR={}
LABEL_FILL_LOW={}
LABEL_FILL_MID={}
LABEL_FILL_HIGH={}
LABEL_FILL_DEEP={}

# --- PALET WARNA KONTUR (HEX) ---
COLOR_CUT_DEEP={}
COLOR_CUT_HIGH={}
COLOR_CUT_MID={}
COLOR_CUT_LOW={}
COLOR_CUT_NEAR={}
COLOR_CUT_MINOR={}
COLOR_CUT_TO_GRADE={}
COLOR_ONGRADE={}
COLOR_FILL_TO_GRADE={}
COLOR_FILL_MINOR={}
COLOR_FILL_NEAR={}
COLOR_FILL_LOW={}
COLOR_FILL_MID={}
COLOR_FILL_HIGH={}
COLOR_FILL_DEEP={}

# --- ISOLINE CONTOUR LEVELS (Meter) ---
CONTOUR_LEVELS={}

# --- DEFAULT OUTPUT ---
DEFAULT_OUTDIR={}
"#,
            first_run_str,
            self.company_name,
            self.district_name,
            self.department_name,
            self.company_logo_path,
            self.project_name,
            self.default_title,
            self.map_subtitle,
            self.drawn_by,
            self.reviewed_by,
            self.approved_by,
            self.coordinate_system,
            self.topo_date,
            self.design_name,
            self.auto_open_browser,
            self.topo_path,
            self.design_path,
            self.grid_interval,
            self.subtick_interval,
            self.grid_step,
            self.max_tin_edge,
            self.weeding_min_dist,
            self.supplement_max_dist,
            self.ongrade_min,
            self.ongrade_max,
            self.label_cut_deep,
            self.label_cut_high,
            self.label_cut_mid,
            self.label_cut_low,
            self.label_cut_near,
            self.label_cut_minor,
            self.label_cut_to_grade,
            self.label_ongrade,
            self.label_fill_to_grade,
            self.label_fill_minor,
            self.label_fill_near,
            self.label_fill_low,
            self.label_fill_mid,
            self.label_fill_high,
            self.label_fill_deep,
            self.color_cut_deep,
            self.color_cut_high,
            self.color_cut_mid,
            self.color_cut_low,
            self.color_cut_near,
            self.color_cut_minor,
            self.color_cut_to_grade,
            self.color_ongrade,
            self.color_fill_to_grade,
            self.color_fill_minor,
            self.color_fill_near,
            self.color_fill_low,
            self.color_fill_mid,
            self.color_fill_high,
            self.color_fill_deep,
            levels_str,
            self.default_outdir
        )
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        fs::write(path, self.to_dat_string())
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
