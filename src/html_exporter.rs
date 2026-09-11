use crate::config::{hex_to_rgba, EngineConfig};
use crate::dxf::StyledPolyline;
use crate::grid_engine::GridPointDelta;
use crate::volume::VolumeSummary;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct KopInfo<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub company: &'a str,
    pub district: &'a str,
    pub department: &'a str,
    pub project_name: &'a str,
    pub drawn_by: &'a str,
    pub reviewed_by: &'a str,
    pub approved_by: &'a str,
    pub date_created: &'a str,
    pub topo_date: &'a str,
    pub design_name: &'a str,
    pub coordinate_system: &'a str,
    pub logo_data_uri: Option<&'a str>,
}

impl<'a> Default for KopInfo<'a> {
    fn default() -> Self {
        Self {
            title: "PETA KONTUR CUT & FILL",
            subtitle: "ISOPACH DIFFERENCE (TOPO - DESIGN)",
            company: "PT PAMAPERSADA NUSANTARA",
            district: "DISTRIK KPCS • SANGATTA",
            department: "ENGINEERING & MINE PLANNING DEPT.",
            project_name: "PIT ALPHA",
            drawn_by: "Fikri Ardyantoro",
            reviewed_by: "Reviewer",
            approved_by: "Approver",
            date_created: "11 September 2026",
            topo_date: "28 July 2026",
            design_name: "design.dxf",
            coordinate_system: "UTM ZONE 50S (WGS84)",
            logo_data_uri: None,
        }
    }
}

fn format_number_with_commas(val: f64) -> String {
    let rounded = format!("{:.2}", val);
    let parts: Vec<&str> = rounded.split('.').collect();
    let int_part = parts[0];
    let dec_part = parts.get(1).unwrap_or(&"00");

    let is_negative = int_part.starts_with('-');
    let raw_digits = if is_negative { &int_part[1..] } else { int_part };

    let mut result = String::new();
    let num_digits = raw_digits.len();

    for (i, c) in raw_digits.chars().enumerate() {
        result.push(c);
        let rem = num_digits - i - 1;
        if rem > 0 && rem % 3 == 0 {
            result.push(',');
        }
    }

    if is_negative {
        format!("-{}.{}", result, dec_part)
    } else {
        format!("{}.{}", result, dec_part)
    }
}

#[derive(Serialize)]
struct CompactStyledLine<'a> {
    c: &'a str,
    pts: Vec<[f64; 2]>,
}

#[derive(Serialize)]
struct HeatmapRasterMetadata {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    step: f64,
    cols: usize,
    rows: usize,
    rle: Vec<u32>, // [color_id, count, color_id, count, ...]
}

pub fn delta_z_to_color_id(dz: f64, ongrade_min: f64, ongrade_max: f64) -> u32 {
    // Cut (Topo > Design, dz > 0)
    if dz > 16.0 {
        1
    } else if dz > 12.0 {
        2
    } else if dz > 8.0 {
        3
    } else if dz > 4.0 {
        4
    } else if dz > 2.0 {
        5
    } else if dz > ongrade_max {
        6
    } else if dz >= ongrade_min && dz <= ongrade_max {
        7 // ON GRADE
    } else if dz >= -2.0 {
        8
    } else if dz >= -4.0 {
        9
    } else if dz >= -8.0 {
        10
    } else if dz >= -12.0 {
        11
    } else if dz >= -16.0 {
        12
    } else {
        13
    }
}

pub fn generate_html_viewer(
    kop: &KopInfo,
    grid: &[GridPointDelta],
    summary: &VolumeSummary,
    design_lines: &[StyledPolyline],
) -> String {
    let def_config = EngineConfig::default();
    generate_html_viewer_with_config(kop, grid, summary, design_lines, &def_config)
}

pub fn generate_html_viewer_with_config(
    kop: &KopInfo,
    grid: &[GridPointDelta],
    summary: &VolumeSummary,
    design_lines: &[StyledPolyline],
    config: &EngineConfig,
) -> String {
    // Dynamically compute bounding box coordinates from actual grid delta points
    let (min_x, max_x, min_y, max_y) = if !grid.is_empty() {
        let min_x = grid.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let max_x = grid.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = grid.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_y = grid.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        (min_x, max_x, min_y, max_y)
    } else {
        (273000.0, 275000.0, 9693500.0, 9695500.0)
    };

    let step = if config.grid_step > 0.001 {
        config.grid_step
    } else if grid.len() > 1 {
        let mut min_diff = f64::INFINITY;
        for i in 0..grid.len().min(100) {
            for j in (i + 1)..grid.len().min(100) {
                let dx = (grid[i].x - grid[j].x).abs();
                let dy = (grid[i].y - grid[j].y).abs();
                if dx > 0.0001 && dx < min_diff {
                    min_diff = dx;
                }
                if dy > 0.0001 && dy < min_diff {
                    min_diff = dy;
                }
            }
        }
        if min_diff.is_finite() && min_diff >= 0.001 {
            (min_diff * 1000.0).round() / 1000.0
        } else {
            1.0
        }
    } else {
        1.0
    };

    let cols = if max_x > min_x { ((max_x - min_x) / step).round() as usize + 1 } else { 1 };
    let rows = if max_y > min_y { ((max_y - min_y) / step).round() as usize + 1 } else { 1 };

    let logo_html = if let Some(uri) = kop.logo_data_uri {
        if !uri.is_empty() {
            format!(r#"<img src="{}" alt="Company Logo" class="max-w-[200px] max-h-[60px] w-auto h-auto object-contain" />"#, uri)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let subtitle_html = if kop.subtitle.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div style="font-size: 7.5px; font-weight: bold; color: #475569; margin-top: 2px;">{}</div>"#, kop.subtitle)
    };

    let mut grid_bytes = vec![0u8; cols * rows];
    for p in grid {
        let gx = ((p.x - min_x) / step).round() as usize;
        let gy = ((p.y - min_y) / step).round() as usize;
        if gx < cols && gy < rows {
            grid_bytes[gy * cols + gx] = delta_z_to_color_id(p.delta_z, config.ongrade_min, config.ongrade_max) as u8;
        }
    }

    let mut rle: Vec<u32> = Vec::new();
    if !grid_bytes.is_empty() {
        let mut cur_val = grid_bytes[0] as u32;
        let mut cur_cnt = 0u32;
        for &b in &grid_bytes {
            let val = b as u32;
            if val == cur_val && cur_cnt < 65535 {
                cur_cnt += 1;
            } else {
                rle.push(cur_val);
                rle.push(cur_cnt);
                cur_val = val;
                cur_cnt = 1;
            }
        }
        rle.push(cur_val);
        rle.push(cur_cnt);
    }

    let raster_meta = HeatmapRasterMetadata {
        min_x,
        max_x,
        min_y,
        max_y,
        step,
        cols,
        rows,
        rle,
    };
    let raster_json = serde_json::to_string(&raster_meta).unwrap_or_else(|_| "{}".to_string());

    let compact_design_lines: Vec<CompactStyledLine> = design_lines
        .iter()
        .map(|pl| CompactStyledLine {
            c: &pl.color_hex,
            pts: pl.points.iter().map(|pt| [(pt.x * 100.0).round() / 100.0, (pt.y * 100.0).round() / 100.0]).collect(),
        })
        .collect();
    let design_lines_json = serde_json::to_string(&compact_design_lines).unwrap_or_else(|_| "[]".to_string());

    // Calculate Viewport Dimensions (isometric 1:1 metric aspect ratio matching frame 728px : 702px = 1.037)
    let span_x = (max_x - min_x).max(10.0);
    let span_y = (max_y - min_y).max(10.0);
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    let frame_aspect = 728.0 / 702.0; // ~1.037
    let mut target_span_x = span_x * 1.15;
    let mut target_span_y = span_y * 1.15;
    if target_span_x / target_span_y < frame_aspect {
        target_span_x = target_span_y * frame_aspect;
    } else {
        target_span_y = target_span_x / frame_aspect;
    }

    let map_min_x = center_x - target_span_x / 2.0;
    let map_max_x = center_x + target_span_x / 2.0;
    let map_min_y = center_y - target_span_y / 2.0;
    let map_max_y = center_y + target_span_y / 2.0;

    // Grid ticks calculation
    let raw_interval = (target_span_x / 6.0).max(10.0);
    let grid_interval = if config.grid_interval > 0.0 {
        config.grid_interval
    } else if raw_interval <= 50.0 {
        50.0
    } else if raw_interval <= 100.0 {
        100.0
    } else if raw_interval <= 200.0 {
        200.0
    } else if raw_interval <= 500.0 {
        500.0
    } else if raw_interval <= 1000.0 {
        1000.0
    } else {
        (raw_interval / 500.0).round() * 500.0
    };
    let subtick_interval = if config.subtick_interval > 0.0 {
        config.subtick_interval
    } else {
        (grid_interval / 4.0).max(10.0)
    };

    let first_easting = ((map_min_x / grid_interval).floor() * grid_interval) as i64;
    let last_easting = ((map_max_x / grid_interval).ceil() * grid_interval) as i64;
    let mut easting_ticks = Vec::new();
    let mut curr_e = first_easting;
    while curr_e <= last_easting {
        let e_f = curr_e as f64;
        if e_f >= map_min_x && e_f <= map_max_x {
            easting_ticks.push(curr_e);
        }
        curr_e += grid_interval as i64;
    }

    let first_northing = ((map_min_y / grid_interval).floor() * grid_interval) as i64;
    let last_northing = ((map_max_y / grid_interval).ceil() * grid_interval) as i64;
    let mut northing_ticks = Vec::new();
    let mut curr_n = first_northing;
    while curr_n <= last_northing {
        let n_f = curr_n as f64;
        if n_f >= map_min_y && n_f <= map_max_y {
            northing_ticks.push(curr_n);
        }
        curr_n += grid_interval as i64;
    }

    // Scale calculation
    let scale_denom = (target_span_x / 0.19136).round() as i64;
    let (bar_dist_m, bar_label) = if target_span_x < 500.0 {
        (50.0, "50m")
    } else if target_span_x < 2000.0 {
        (200.0, "200m")
    } else {
        (500.0, "500m")
    };
    let bar_width_px = ((bar_dist_m / target_span_x) * 728.0).round().max(70.0).min(180.0) as i64;
    let scale_box_w = (bar_width_px + 36).max(160);
    let scale_box_h = 44;
    let bar_start_x = (scale_box_w - bar_width_px) / 2;
    let seg_w = bar_width_px as f64 / 4.0;

    let scale_bar_svg = format!(
        r##"<svg width="{w}" height="{h}" viewBox="0 0 {w} {h}" style="display: block;">
            <text x="{cx}" y="11" font-size="8.5" font-weight="bold" font-family="Arial, sans-serif" text-anchor="middle" fill="#000000" letter-spacing="0.2">SCALE 1 : {denom} &bull; GRID {grid:.0}m</text>
            <rect x="{bx}" y="17" width="{sw:.2}" height="7" fill="#000000" stroke="#000000" stroke-width="0.8" />
            <rect x="{bx1:.2}" y="17" width="{sw:.2}" height="7" fill="#ffffff" stroke="#000000" stroke-width="0.8" />
            <rect x="{bx2:.2}" y="17" width="{sw:.2}" height="7" fill="#000000" stroke="#000000" stroke-width="0.8" />
            <rect x="{bx3:.2}" y="17" width="{sw:.2}" height="7" fill="#ffffff" stroke="#000000" stroke-width="0.8" />
            <text x="{bx}" y="36" font-size="8" font-family="Arial, sans-serif" font-weight="bold" text-anchor="middle" fill="#000000">0</text>
            <text x="{bx_end}" y="36" font-size="8" font-family="Arial, sans-serif" font-weight="bold" text-anchor="middle" fill="#000000">{label}</text>
        </svg>"##,
        w = scale_box_w,
        h = scale_box_h,
        cx = scale_box_w as f64 / 2.0,
        denom = scale_denom,
        grid = grid_interval,
        bx = bar_start_x,
        sw = seg_w,
        bx1 = bar_start_x as f64 + seg_w,
        bx2 = bar_start_x as f64 + 2.0 * seg_w,
        bx3 = bar_start_x as f64 + 3.0 * seg_w,
        bx_end = bar_start_x + bar_width_px,
        label = bar_label,
    );

    // Generate SVG Grid Lines and Ticks
    let mut svg_grid_elements = Vec::new();

    // Easting Major Lines & Ticks
    for &e in &easting_ticks {
        let pct = (e as f64 - map_min_x) / (map_max_x - map_min_x) * 100.0;
        svg_grid_elements.push(format!(
            r##"<line x1="{:.4}%" y1="0%" x2="{:.4}%" y2="100%" stroke="#000000" stroke-width="0.5" stroke-dasharray="3 3" opacity="0.25" />"##,
            pct, pct
        ));
        svg_grid_elements.push(format!(
            r##"<line x1="{:.4}%" y1="0" x2="{:.4}%" y2="8" stroke="#000000" stroke-width="1.2" />"##,
            pct, pct
        ));
        svg_grid_elements.push(format!(
            r##"<line x1="{:.4}%" y1="100%" x2="{:.4}%" y2="calc(100% - 8px)" stroke="#000000" stroke-width="1.2" />"##,
            pct, pct
        ));
    }

    // Easting Sub-ticks
    let mut sub_e = ((map_min_x / subtick_interval).floor() * subtick_interval) as i64;
    while (sub_e as f64) <= map_max_x {
        let e_f = sub_e as f64;
        if e_f >= map_min_x && e_f <= map_max_x && !easting_ticks.contains(&sub_e) {
            let pct = (e_f - map_min_x) / (map_max_x - map_min_x) * 100.0;
            svg_grid_elements.push(format!(
                r##"<line x1="{:.4}%" y1="0" x2="{:.4}%" y2="4" stroke="#000000" stroke-width="0.8" />"##,
                pct, pct
            ));
            svg_grid_elements.push(format!(
                r##"<line x1="{:.4}%" y1="100%" x2="{:.4}%" y2="calc(100% - 4px)" stroke="#000000" stroke-width="0.8" />"##,
                pct, pct
            ));
        }
        sub_e += subtick_interval as i64;
    }

    // Northing Major Lines & Ticks
    for &n in &northing_ticks {
        let pct = (map_max_y - n as f64) / (map_max_y - map_min_y) * 100.0;
        svg_grid_elements.push(format!(
            r##"<line x1="0%" y1="{:.4}%" x2="100%" y2="{:.4}%" stroke="#000000" stroke-width="0.5" stroke-dasharray="3 3" opacity="0.25" />"##,
            pct, pct
        ));
        svg_grid_elements.push(format!(
            r##"<line x1="0" y1="{:.4}%" x2="8" y2="{:.4}%" stroke="#000000" stroke-width="1.2" />"##,
            pct, pct
        ));
        svg_grid_elements.push(format!(
            r##"<line x1="100%" y1="{:.4}%" x2="calc(100% - 8px)" y2="{:.4}%" stroke="#000000" stroke-width="1.2" />"##,
            pct, pct
        ));
    }

    // Northing Sub-ticks
    let mut sub_n = ((map_min_y / subtick_interval).floor() * subtick_interval) as i64;
    while (sub_n as f64) <= map_max_y {
        let n_f = sub_n as f64;
        if n_f >= map_min_y && n_f <= map_max_y && !northing_ticks.contains(&sub_n) {
            let pct = (map_max_y - n_f) / (map_max_y - map_min_y) * 100.0;
            svg_grid_elements.push(format!(
                r##"<line x1="0" y1="{:.4}%" x2="4" y2="{:.4}%" stroke="#000000" stroke-width="0.8" />"##,
                pct, pct
            ));
            svg_grid_elements.push(format!(
                r##"<line x1="100%" y1="{:.4}%" x2="calc(100% - 4px)" y2="{:.4}%" stroke="#000000" stroke-width="0.8" />"##,
                pct, pct
            ));
        }
        sub_n += subtick_interval as i64;
    }

    let grid_svg = svg_grid_elements.join("\n        ");

    // Generate 4-sided coordinate labels as precision SVGs
    let mut top_labels = Vec::new();
    let mut bottom_labels = Vec::new();
    for &e in &easting_ticks {
        let x_px = (e as f64 - map_min_x) / (map_max_x - map_min_x) * 728.0;
        top_labels.push(format!(
            r##"<text x="{:.1}" y="18" text-anchor="middle" font-size="9.5" font-weight="bold" font-family="Arial, sans-serif" fill="#000000" letter-spacing="0.3">{}</text>"##,
            x_px, e
        ));
        bottom_labels.push(format!(
            r##"<text x="{:.1}" y="18" text-anchor="middle" font-size="9.5" font-weight="bold" font-family="Arial, sans-serif" fill="#000000" letter-spacing="0.3">{}</text>"##,
            x_px, e
        ));
    }

    let mut left_labels = Vec::new();
    let mut right_labels = Vec::new();
    for &n in &northing_ticks {
        let y_px = (map_max_y - n as f64) / (map_max_y - map_min_y) * 702.0;
        left_labels.push(format!(
            r##"<text x="24" y="{:.1}" transform="rotate(-90 24 {:.1})" text-anchor="middle" font-size="9.5" font-weight="bold" font-family="Arial, sans-serif" fill="#000000" letter-spacing="0.3">{}</text>"##,
            y_px, y_px, n
        ));
        right_labels.push(format!(
            r##"<text x="25" y="{:.1}" transform="rotate(-90 25 {:.1})" text-anchor="middle" font-size="9.5" font-weight="bold" font-family="Arial, sans-serif" fill="#000000" letter-spacing="0.3">{}</text>"##,
            y_px, y_px, n
        ));
    }

    let top_labels_html = format!(
        r#"<svg width="728" height="28" viewBox="0 0 728 28" style="display: block;">{}</svg>"#,
        top_labels.join("")
    );
    let bottom_labels_html = format!(
        r#"<svg width="728" height="28" viewBox="0 0 728 28" style="display: block;">{}</svg>"#,
        bottom_labels.join("")
    );
    let left_labels_html = format!(
        r#"<svg width="49" height="702" viewBox="0 0 49 702" style="display: block;">{}</svg>"#,
        left_labels.join("")
    );
    let right_labels_html = format!(
        r#"<svg width="49" height="702" viewBox="0 0 49 702" style="display: block;">{}</svg>"#,
        right_labels.join("")
    );

    // Build paletteRGBA javascript array
    let p_empty = [0, 0, 0, 0];
    let p1 = hex_to_rgba(&config.color_cut_deep);
    let p2 = hex_to_rgba(&config.color_cut_high);
    let p3 = hex_to_rgba(&config.color_cut_mid);
    let p4 = hex_to_rgba(&config.color_cut_low);
    let p5 = hex_to_rgba(&config.color_cut_near);
    let p6 = hex_to_rgba(&config.color_cut_to_grade);
    let p7 = hex_to_rgba(&config.color_ongrade);
    let p8 = hex_to_rgba(&config.color_fill_to_grade);
    let p9 = hex_to_rgba(&config.color_fill_near);
    let p10 = hex_to_rgba(&config.color_fill_low);
    let p11 = hex_to_rgba(&config.color_fill_mid);
    let p12 = hex_to_rgba(&config.color_fill_high);
    let p13 = hex_to_rgba(&config.color_fill_deep);

    let palette_js = format!(
        "[\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}],\n  [{},{},{},{}]\n]",
        p_empty[0], p_empty[1], p_empty[2], p_empty[3],
        p1[0], p1[1], p1[2], p1[3],
        p2[0], p2[1], p2[2], p2[3],
        p3[0], p3[1], p3[2], p3[3],
        p4[0], p4[1], p4[2], p4[3],
        p5[0], p5[1], p5[2], p5[3],
        p6[0], p6[1], p6[2], p6[3],
        p7[0], p7[1], p7[2], p7[3],
        p8[0], p8[1], p8[2], p8[3],
        p9[0], p9[1], p9[2], p9[3],
        p10[0], p10[1], p10[2], p10[3],
        p11[0], p11[1], p11[2], p11[3],
        p12[0], p12[1], p12[2], p12[3],
        p13[0], p13[1], p13[2], p13[3],
    );

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>PETA RAINBOW CONTOUR • {title}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/html2canvas-pro@1.5.8/dist/html2canvas-pro.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/jspdf/2.5.1/jspdf.umd.min.js"></script>
    <style>
        @page {{ size: A4 landscape; margin: 0; }}
        *, *::before, *::after {{
            box-sizing: border-box;
            -webkit-print-color-adjust: exact !important;
            print-color-adjust: exact !important;
        }}

        @media print {{
            .no-print {{ display: none !important; }}
            body {{ padding: 0 !important; margin: 0 !important; background: #ffffff !important; }}
            #pdf-kop-container {{
                box-shadow: none !important;
                margin: 0 !important;
            }}
        }}

        .font-cad-mono {{
            font-family: 'Consolas', 'Courier New', monospace !important;
        }}

        /* Strict Fixed Sheet Geometry (A4 Landscape 1123px x 794px) */
        #pdf-kop-container {{
            width: 1123px !important;
            height: 794px !important;
            min-width: 1123px !important;
            min-height: 794px !important;
            max-width: 1123px !important;
            max-height: 794px !important;
            background-color: #ffffff;
            padding: 16px;
            position: relative;
            display: flex;
            box-sizing: border-box;
            overflow: hidden;
        }}

        #sheet-inner-border {{
            width: 1091px;
            height: 762px;
            border: 2px solid #000000;
            display: flex;
            position: relative;
            background: #ffffff;
            box-sizing: border-box;
            overflow: hidden;
        }}

        /* Left Map Area */
        #map-area {{
            width: 826px;
            height: 758px;
            position: relative;
            background: #ffffff;
        }}

        #top-coord-bar {{
            position: absolute;
            left: 49px;
            top: 0;
            width: 728px;
            height: 28px;
        }}

        #bottom-coord-bar {{
            position: absolute;
            left: 49px;
            bottom: 0;
            width: 728px;
            height: 28px;
        }}

        #left-coord-bar {{
            position: absolute;
            left: 0;
            top: 28px;
            width: 49px;
            height: 702px;
        }}

        #right-coord-bar {{
            position: absolute;
            right: 0;
            top: 28px;
            width: 49px;
            height: 702px;
        }}

        #map-frame {{
            position: absolute;
            left: 49px;
            top: 28px;
            width: 728px;
            height: 702px;
            border: 1.5px solid #000000;
            background: #ffffff;
            overflow: hidden;
        }}

        #contourCanvas {{
            width: 100% !important;
            height: 100% !important;
            display: block !important;
        }}

        /* North Arrow */
        #north-arrow {{
            position: absolute;
            top: 12px;
            left: 12px;
            width: 62px;
            height: 76px;
            background: rgba(255, 255, 255, 0.95);
            border: 1.5px solid #000000;
            display: flex;
            align-items: center;
            justify-content: center;
            box-sizing: border-box;
            z-index: 10;
        }}

        /* Scale Bar Box */
        #scale-bar-box {{
            position: absolute;
            bottom: 12px;
            left: 12px;
            background: rgba(255, 255, 255, 0.95);
            border: 1.5px solid #000000;
            padding: 4px 6px;
            display: flex;
            align-items: center;
            justify-content: center;
            box-sizing: border-box;
            z-index: 10;
        }}

        /* Right Civil 3D Kop Sidebar */
        #kop-sidebar {{
            width: 265px;
            height: 758px;
            border-left: 2px solid #000000;
            display: flex;
            flex-direction: column;
            background: #ffffff;
            font-size: 10px;
            color: #000000;
            overflow: hidden;
            box-sizing: border-box;
        }}

        .kop-section {{
            width: 100%;
            border-bottom: 1.5px solid #000000;
            position: relative;
            box-sizing: border-box;
        }}

        .kop-header-bar {{
            background: #e2e8f0;
            border-bottom: 1px solid #000000;
            padding: 3px 6px 4px 6px;
            font-size: 8.5px;
            font-weight: 900;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            color: #000000;
            text-align: center;
            line-height: 1.15 !important;
        }}

        .swatch {{
            width: 11px;
            height: 11px;
            border: 1px solid #000000;
            flex-shrink: 0;
            display: inline-block;
        }}
    </style>
    <script>
        async function downloadPDF() {{
            try {{
                const element = document.getElementById('pdf-kop-container');
                const html2canvasFn = window.html2canvasPro || window.html2canvas;
                if (!html2canvasFn) {{
                    alert('Library html2canvas belum terload. Mohon cek koneksi internet.');
                    return;
                }}
                
                const btn = document.getElementById('export-btn');
                const originalText = btn.innerHTML;
                btn.innerHTML = '<span class="animate-pulse">Generating PDF...</span>';
                btn.disabled = true;

                if (typeof resetView === 'function') {{
                    resetView();
                }} else if (typeof drawContour === 'function') {{
                    drawContour();
                }}

                const hud = document.getElementById('cad-hud-controls');
                if (hud) hud.style.display = 'none';

                const canvas = await html2canvasFn(element, {{
                    scale: 2,
                    useCORS: true,
                    allowTaint: true,
                    backgroundColor: '#ffffff',
                    logging: false,
                    width: 1123,
                    height: 794,
                    onclone: (clonedDoc) => {{
                        clonedDoc.body.style.margin = '0';
                        clonedDoc.body.style.padding = '0';
                        clonedDoc.body.style.background = '#ffffff';
                        const toolbar = clonedDoc.querySelector('.no-print');
                        if (toolbar) toolbar.remove();
                        const clonedContainer = clonedDoc.getElementById('pdf-kop-container');
                        if (clonedContainer) {{
                            clonedContainer.style.margin = '0';
                            clonedContainer.style.boxShadow = 'none';
                        }}
                    }}
                }});

                if (hud) hud.style.display = 'flex';

                const imgData = canvas.toDataURL('image/png', 1.0);
                const {{ jsPDF }} = window.jspdf;
                const pdf = new jsPDF({{
                    orientation: 'landscape',
                    unit: 'mm',
                    format: 'a4'
                }});

                pdf.addImage(imgData, 'PNG', 0, 0, 297, 210, undefined, 'FAST');
                pdf.save('rainbow-contour-kop.pdf');

                btn.innerHTML = originalText;
                btn.disabled = false;
            }} catch (err) {{
                console.error('PDF export error:', err);
                const hud = document.getElementById('cad-hud-controls');
                if (hud) hud.style.display = 'flex';
                alert('Gagal membuat PDF: ' + err.message);
                const btn = document.getElementById('export-btn');
                btn.innerHTML = 'DOWNLOAD PDF (A4 LANDSCAPE)';
                btn.disabled = false;
            }}
        }}
    </script>
</head>
<body class="bg-slate-200 min-h-screen flex flex-col items-center justify-start p-6 font-sans antialiased text-slate-800">

    <!-- Top Action Toolbar (Outside PDF Container) -->
    <div class="no-print w-[1123px] flex items-center justify-between mb-3 bg-white p-3 border-2 border-slate-900 shadow-[4px_4px_0px_rgba(15,23,42,1)]">
        <div class="flex items-center gap-2">
            <span class="inline-block w-3 h-3 bg-red-600 rounded-full animate-ping"></span>
            <span class="font-black text-xs uppercase tracking-widest text-slate-900">PETA KONTUR CUT &amp; FILL • CIVIL 3D STYLE</span>
        </div>
        <button id="export-btn" onclick="downloadPDF()" class="bg-yellow-300 hover:bg-yellow-400 border-2 border-slate-900 px-4 py-2 font-black text-xs uppercase shadow-[2px_2px_0px_rgba(15,23,42,1)] active:translate-x-0.5 active:translate-y-0.5 flex items-center justify-center gap-2 cursor-pointer transition-colors">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
            DOWNLOAD PDF (A4 LANDSCAPE)
        </button>
    </div>

    <!-- Exact A4 Landscape Container (1123px x 794px @ 96dpi) -->
    <div id="pdf-kop-container" class="shadow-[8px_8px_0px_rgba(15,23,42,1)]">
        <div id="sheet-inner-border">

            <!-- ================= LEFT: MAP AREA ================= -->
            <div id="map-area">
                
                <!-- Top Coordinate Bar -->
                <div id="top-coord-bar">
                    {top_labels_html}
                </div>

                <!-- Bottom Coordinate Bar -->
                <div id="bottom-coord-bar">
                    {bottom_labels_html}
                </div>

                <!-- Left Coordinate Bar -->
                <div id="left-coord-bar">
                    {left_labels_html}
                </div>

                <!-- Right Coordinate Bar -->
                <div id="right-coord-bar">
                    {right_labels_html}
                </div>

                <!-- Main Map Frame -->
                <div id="map-frame">
                    <canvas id="contourCanvas" class="w-full h-full block cursor-grab active:cursor-grabbing"></canvas>

                    <!-- SVG Grid Overlay -->
                    <svg class="absolute inset-0 w-full h-full pointer-events-none">
                        {grid_svg}
                    </svg>

                    <!-- North Arrow -->
                    <div id="north-arrow">
                        <svg width="58" height="72" viewBox="0 0 58 72" style="display: block;">
                            <text x="29" y="11" font-size="11.5" font-weight="bold" font-family="'Times New Roman', serif" text-anchor="middle" fill="#000000">N</text>
                            
                            <!-- North Arrowhead -->
                            <polygon points="29,15 29,41 23,37" fill="#000000" />
                            <polygon points="29,15 35,37 29,41" fill="#ffffff" stroke="#000000" stroke-width="0.8" />
                            
                            <!-- South Arrowhead -->
                            <polygon points="29,61 29,41 34,44" fill="#000000" />
                            <polygon points="29,61 24,44 29,41" fill="#ffffff" stroke="#000000" stroke-width="0.8" />
                            
                            <!-- West Arrowhead -->
                            <polygon points="9,41 29,41 26,46" fill="#000000" />
                            <polygon points="9,41 26,36 29,41" fill="#ffffff" stroke="#000000" stroke-width="0.8" />
                            
                            <!-- East Arrowhead -->
                            <polygon points="49,41 29,41 32,36" fill="#000000" />
                            <polygon points="49,41 32,46 29,41" fill="#ffffff" stroke="#000000" stroke-width="0.8" />
                            
                            <!-- Cardinal W, E, S -->
                            <text x="3.5" y="43.5" font-size="7" font-weight="bold" font-family="'Times New Roman', serif" text-anchor="middle" fill="#000000">W</text>
                            <text x="54.5" y="43.5" font-size="7" font-weight="bold" font-family="'Times New Roman', serif" text-anchor="middle" fill="#000000">E</text>
                            <text x="29" y="69.5" font-size="7" font-weight="bold" font-family="'Times New Roman', serif" text-anchor="middle" fill="#000000">S</text>
                            
                            <!-- Center Ring & Dot -->
                            <circle cx="29" cy="41" r="5" fill="none" stroke="#000000" stroke-width="1.2" />
                            <circle cx="29" cy="41" r="1.5" fill="#000000" />
                        </svg>
                    </div>

                    <!-- Scale Bar Box -->
                    <div id="scale-bar-box">
                        {scale_bar_svg}
                    </div>

                    <!-- Floating CAD Navigation HUD (Zoom / Pan Controls) -->
                    <div id="cad-hud-controls" class="absolute bottom-3 right-3 flex items-center gap-1.5 bg-slate-900/90 border-2 border-slate-700 p-1.5 shadow-lg backdrop-blur-sm z-20">
                        <button type="button" onclick="zoomIn()" title="Zoom In (+)" class="w-7 h-7 bg-white hover:bg-yellow-300 border border-slate-900 text-slate-900 font-black text-sm flex items-center justify-center cursor-pointer active:translate-y-0.5 transition-all">+</button>
                        <button type="button" onclick="zoomOut()" title="Zoom Out (-)" class="w-7 h-7 bg-white hover:bg-yellow-300 border border-slate-900 text-slate-900 font-black text-sm flex items-center justify-center cursor-pointer active:translate-y-0.5 transition-all">−</button>
                        <button type="button" onclick="resetView()" title="Fit View (Extents)" class="px-2 h-7 bg-yellow-300 hover:bg-yellow-400 border border-slate-900 text-slate-900 font-black text-[10px] uppercase flex items-center justify-center cursor-pointer active:translate-y-0.5 transition-all">FIT</button>
                        <span id="zoomReadout" class="font-cad-mono text-[10px] text-yellow-400 font-bold px-1.5 select-none">100%</span>
                    </div>

                </div>
            </div>

            <!-- ================= RIGHT: CIVIL 3D KOP ================= -->
            <div id="kop-sidebar">
                
                <!-- 1. Company Header (~46px) -->
                <div class="kop-section flex flex-col items-center justify-center text-center p-1.5" style="height: 46px;">
                    <div style="font-size: 10.5px; font-weight: 900; letter-spacing: 0.2px;">{company}</div>
                    <div style="font-size: 8.5px; font-weight: bold; color: #1e293b; margin-top: 1px;">{district}</div>
                    <div style="font-size: 7px; font-weight: bold; color: #64748b; letter-spacing: 0.4px; margin-top: 1px;">{department}</div>
                </div>

                <!-- 2. Logo Box (~76px) -->
                <div class="kop-section flex items-center justify-center p-2 bg-white" style="height: 76px;">
                    {logo_html}
                </div>

                <!-- 3. MAP TITLE (SWAPPED TO TOP OF INFO, ~65px) -->
                <div class="kop-section flex flex-col items-center justify-center text-center px-2 py-1.5 bg-amber-50" style="height: 65px;">
                    <div style="font-size: 7px; font-weight: bold; color: #b45309; letter-spacing: 0.8px; margin-bottom: 2px;">MAP TITLE</div>
                    <div style="font-size: 11.5px; font-weight: 900; line-height: 1.15; color: #000000; letter-spacing: 0.2px;">{title}</div>
                    {subtitle_html}
                </div>

                <!-- 4. MAP INFORMATION (SWAPPED UNDER TITLE, ~110px) -->
                <div class="kop-section flex flex-col" style="height: 110px; box-sizing: border-box; overflow: hidden;">
                    <div class="kop-header-bar" style="height: 20px; line-height: 20px; box-sizing: border-box;">MAP INFORMATION</div>
                    <div class="flex flex-col justify-around flex-1 px-2.5 py-1 text-[8.5px]" style="box-sizing: border-box;">
                        <div class="flex items-center justify-between border-b border-slate-200 py-1">
                            <span class="font-bold text-slate-500 uppercase">PROJECT:</span>
                            <span class="font-black text-black text-right">{project}</span>
                        </div>
                        <div class="flex items-center justify-between border-b border-slate-200 py-1">
                            <span class="font-bold text-slate-500 uppercase">TOPO DATE:</span>
                            <span class="font-black text-black text-right">{topo_date}</span>
                        </div>
                        <div class="flex items-center justify-between border-b border-slate-200 py-1">
                            <span class="font-bold text-slate-500 uppercase">DESIGN REF:</span>
                            <span class="font-black text-black text-right">{design_name}</span>
                        </div>
                        <div class="flex items-center justify-between py-1">
                            <span class="font-bold text-slate-500 uppercase">PROJECTION:</span>
                            <span class="font-black text-black text-right">{coord_sys}</span>
                        </div>
                    </div>
                </div>

                <!-- 5. VALIDATION (~98px) -->
                <div class="kop-section flex flex-col" style="height: 98px; box-sizing: border-box; overflow: hidden;">
                    <div class="kop-header-bar" style="height: 20px; line-height: 20px; box-sizing: border-box;">VALIDATION</div>
                    <div class="flex flex-col flex-1" style="box-sizing: border-box;">
                        <!-- Header Row -->
                        <div class="flex items-center text-center font-bold text-[8px] bg-slate-100 border-b border-black" style="height: 18px;">
                            <div class="w-[28%] border-r border-black h-full flex items-center justify-center">ROLE</div>
                            <div class="w-[44%] border-r border-black h-full flex items-center justify-center">NAME</div>
                            <div class="w-[28%] h-full flex items-center justify-center">SIGN</div>
                        </div>
                        <!-- DRAWN Row -->
                        <div class="flex items-center text-center text-[8.5px] border-b border-slate-300 flex-1">
                            <div class="w-[28%] font-bold bg-slate-50 border-r border-slate-300 h-full flex items-center justify-center">DRAWN</div>
                            <div class="w-[44%] font-black border-r border-slate-300 h-full flex items-center justify-center truncate px-1">{drawn_by}</div>
                            <div class="w-[28%] h-full bg-white"></div>
                        </div>
                        <!-- REVIEWED Row -->
                        <div class="flex items-center text-center text-[8.5px] border-b border-slate-300 flex-1">
                            <div class="w-[28%] font-bold bg-slate-50 border-r border-slate-300 h-full flex items-center justify-center">REVIEWED</div>
                            <div class="w-[44%] font-black border-r border-slate-300 h-full flex items-center justify-center truncate px-1">{reviewed_by}</div>
                            <div class="w-[28%] h-full bg-white"></div>
                        </div>
                        <!-- APPROVED Row -->
                        <div class="flex items-center text-center text-[8.5px] flex-1">
                            <div class="w-[28%] font-bold bg-slate-50 border-r border-slate-300 h-full flex items-center justify-center">APPROVED</div>
                            <div class="w-[44%] font-black border-r border-slate-300 h-full flex items-center justify-center truncate px-1">{approved_by}</div>
                            <div class="w-[28%] h-full bg-white"></div>
                        </div>
                    </div>
                </div>

                <!-- 6. LEGEND (flex-1, classic unified table) -->
                <div class="kop-section flex flex-col flex-1" style="box-sizing: border-box; overflow: hidden;">
                    <div class="kop-header-bar" style="height: 20px; line-height: 20px; box-sizing: border-box;">LEGEND</div>
                    <div class="px-2.5 py-2 flex flex-col flex-1" style="box-sizing: border-box;">
                        <table class="w-full h-full text-[9px] font-bold" style="border-collapse: collapse;">
                            <!-- Cut Rows -->
                            <tr>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_cut_deep}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_cut_deep}</span>
                                </td>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_cut_high}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_cut_high}</span>
                                </td>
                            </tr>
                            <tr>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_cut_mid}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_cut_mid}</span>
                                </td>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_cut_low}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_cut_low}</span>
                                </td>
                            </tr>
                            <tr>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_cut_near}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_cut_near}</span>
                                </td>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_cut_to_grade}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_cut_to_grade}</span>
                                </td>
                            </tr>

                            <!-- On Grade Row -->
                            <tr>
                                <td colspan="2" style="vertical-align: middle;">
                                    <div class="w-full bg-emerald-50 border border-emerald-400 py-1 px-2 flex items-center gap-2 rounded-[2px]">
                                        <span class="swatch" style="background-color: {c_ongrade};"></span>
                                        <span class="text-emerald-900 font-black text-[9px]">{l_ongrade}</span>
                                    </div>
                                </td>
                            </tr>

                            <!-- Fill Rows -->
                            <tr>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_fill_to_grade}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_fill_to_grade}</span>
                                </td>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_fill_near}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_fill_near}</span>
                                </td>
                            </tr>
                            <tr>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_fill_low}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_fill_low}</span>
                                </td>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_fill_mid}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_fill_mid}</span>
                                </td>
                            </tr>
                            <tr>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_fill_high}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_fill_high}</span>
                                </td>
                                <td style="width: 50%; vertical-align: middle;">
                                    <span class="swatch" style="background-color: {c_fill_deep}; margin-right: 5px;"></span><span style="vertical-align: middle;">{l_fill_deep}</span>
                                </td>
                            </tr>
                        </table>
                    </div>
                </div>

                <!-- 7. VOLUME SUMMARY (Compact, docked at bottom) -->
                <div class="kop-section flex flex-col flex-shrink-0 bg-slate-50 border-b-0 overflow-hidden" style="height: 92px; flex-shrink: 0; box-sizing: border-box;">
                    <div class="kop-header-bar" style="height: 20px; line-height: 20px; box-sizing: border-box;">VOLUME SUMMARY</div>
                    <div class="flex flex-col justify-between flex-1 px-3.5 py-2 text-[10px]" style="box-sizing: border-box;">
                        <div class="flex items-center justify-between">
                            <span class="font-bold text-red-700 uppercase">CUT:</span>
                            <span class="font-black text-red-700 font-cad-mono text-[12px]">{cut_vol} m³</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="font-bold text-blue-700 uppercase">FILL:</span>
                            <span class="font-black text-blue-700 font-cad-mono text-[12px]">{fill_vol} m³</span>
                        </div>
                        <div class="flex items-center justify-between border-t border-slate-900 pt-1 mt-0.5">
                            <span class="font-black text-slate-900 uppercase">NET:</span>
                            <span class="font-black text-slate-900 font-cad-mono text-[12.5px]">{net_vol} m³</span>
                        </div>
                    </div>
                </div>

            </div>

        </div>
    </div>

    <!-- Ultra-Fast Canvas Renderer with RLE Raster Decompression & CAD Vector Layer -->
    <script>
        const rasterData = {raster_json};
        const designPolylines = {design_lines_json};
        const paletteRGBA = {palette_js};

        const mapMinX = {map_min_x};
        const mapMaxX = {map_max_x};
        const mapMinY = {map_min_y};
        const mapMaxY = {map_max_y};

        let offscreenCanvas = null;

        // Interactive CAD Viewport State
        let viewZoom = 1.0;
        let panOffsetX = 0;
        let panOffsetY = 0;
        let isDragging = false;
        let dragStartX = 0;
        let dragStartY = 0;
        let didDrag = false;

        function updateZoomUI() {{
            const readout = document.getElementById('zoomReadout');
            if (readout) {{
                readout.textContent = Math.round(viewZoom * 100) + '%';
            }}
        }}

        function zoomIn() {{
            viewZoom = Math.min(25.0, viewZoom * 1.25);
            updateZoomUI();
            drawContour();
        }}

        function zoomOut() {{
            viewZoom = Math.max(0.2, viewZoom / 1.25);
            updateZoomUI();
            drawContour();
        }}

        function resetView() {{
            viewZoom = 1.0;
            panOffsetX = 0;
            panOffsetY = 0;
            updateZoomUI();
            drawContour();
        }}

        function buildRasterImage() {{
            if (!rasterData || !rasterData.rle || rasterData.cols === 0 || rasterData.rows === 0) return null;
            if (offscreenCanvas) return offscreenCanvas;

            const cols = rasterData.cols;
            const rows = rasterData.rows;
            const rle = rasterData.rle;

            offscreenCanvas = document.createElement('canvas');
            offscreenCanvas.width = cols;
            offscreenCanvas.height = rows;
            const offCtx = offscreenCanvas.getContext('2d');
            const imgData = offCtx.createImageData(cols, rows);
            const buf = imgData.data;

            let pixelIdx = 0;
            const totalPixels = cols * rows;

            for (let i = 0; i < rle.length; i += 2) {{
                const colorId = rle[i];
                const count = rle[i + 1];
                const rgba = paletteRGBA[colorId] || [0,0,0,0];

                for (let k = 0; k < count && pixelIdx < totalPixels; k++) {{
                    const gy = Math.floor(pixelIdx / cols);
                    const gx = pixelIdx % cols;
                    const flippedY = rows - 1 - gy;
                    const destIdx = (flippedY * cols + gx) * 4;

                    buf[destIdx] = rgba[0];
                    buf[destIdx + 1] = rgba[1];
                    buf[destIdx + 2] = rgba[2];
                    buf[destIdx + 3] = rgba[3];
                    pixelIdx++;
                }}
            }}

            offCtx.putImageData(imgData, 0, 0);
            return offscreenCanvas;
        }}

        function drawContour() {{
            const canvas = document.getElementById('contourCanvas');
            if (!canvas) return;
            const ctx = canvas.getContext('2d');

            const dpr = window.devicePixelRatio || 2;
            const displayWidth = canvas.parentElement.clientWidth;
            const displayHeight = canvas.parentElement.clientHeight;

            canvas.width = displayWidth * dpr;
            canvas.height = displayHeight * dpr;
            ctx.scale(dpr, dpr);

            const width = displayWidth;
            const height = displayHeight;

            // Pure White CAD Paper
            ctx.fillStyle = '#ffffff';
            ctx.fillRect(0, 0, width, height);

            const minX = rasterData.min_x || 0;
            const maxX = rasterData.max_x || 100;
            const minY = rasterData.min_y || 0;
            const maxY = rasterData.max_y || 100;

            const dx = (maxX - minX) || 100;
            const dy = (maxY - minY) || 100;

            const baseScale = Math.min(width / (mapMaxX - mapMinX), height / (mapMaxY - mapMinY));
            const scale = baseScale * viewZoom;

            const mapCenterX = width / 2 + panOffsetX;
            const mapCenterY = height / 2 + panOffsetY;
            const viewMidX = (mapMinX + mapMaxX) / 2;
            const viewMidY = (mapMinY + mapMaxY) / 2;

            // 1. Draw Raster Contour Heatmap
            const rasterImg = buildRasterImage();
            if (rasterImg) {{
                ctx.imageSmoothingEnabled = false;
                const destW = dx * scale;
                const destH = dy * scale;
                const midX = (minX + maxX) / 2;
                const midY = (minY + maxY) / 2;

                const destX = mapCenterX + (midX - viewMidX) * scale - destW / 2;
                const destY = mapCenterY - (midY - viewMidY) * scale - destH / 2;

                ctx.drawImage(rasterImg, destX, destY, destW, destH);
            }}

            // 2. Draw Vector Design Lines
            if (designPolylines && designPolylines.length > 0) {{
                ctx.lineWidth = Math.max(1.0, Math.min(3.0, 1.2 * Math.sqrt(viewZoom)));

                for (const pl of designPolylines) {{
                    if (!pl.pts || pl.pts.length < 2) continue;
                    ctx.strokeStyle = pl.c || '#0f172a';
                    ctx.beginPath();
                    for (let i = 0; i < pl.pts.length; i++) {{
                        const px = mapCenterX + (pl.pts[i][0] - viewMidX) * scale;
                        const py = mapCenterY - (pl.pts[i][1] - viewMidY) * scale;
                        if (i === 0) ctx.moveTo(px, py);
                        else ctx.lineTo(px, py);
                    }}
                    ctx.stroke();
                }}
            }}
        }}

        window.addEventListener('DOMContentLoaded', () => {{
            const canvas = document.getElementById('contourCanvas');
            if (!canvas) return;

            canvas.addEventListener('wheel', (e) => {{
                e.preventDefault();
                const zoomFactor = e.deltaY < 0 ? 1.15 : 0.85;
                const newZoom = Math.min(25.0, Math.max(0.2, viewZoom * zoomFactor));
                viewZoom = newZoom;
                updateZoomUI();
                drawContour();
            }}, {{ passive: false }});

            canvas.addEventListener('mousedown', (e) => {{
                if (e.button !== 0) return;
                isDragging = true;
                didDrag = false;
                dragStartX = e.clientX - panOffsetX;
                dragStartY = e.clientY - panOffsetY;
            }});

            window.addEventListener('mousemove', (e) => {{
                if (!isDragging) return;
                didDrag = true;
                panOffsetX = e.clientX - dragStartX;
                panOffsetY = e.clientY - dragStartY;
                drawContour();
            }});

            window.addEventListener('mouseup', () => {{
                isDragging = false;
            }});

            window.addEventListener('resize', drawContour);
            drawContour();
        }});
    </script>
</body>
</html>
"##,
        title = kop.title,
        subtitle_html = subtitle_html,
        company = kop.company,
        district = kop.district,
        department = kop.department,
        project = kop.project_name,
        drawn_by = kop.drawn_by,
        reviewed_by = kop.reviewed_by,
        approved_by = kop.approved_by,
        topo_date = kop.topo_date,
        design_name = kop.design_name,
        coord_sys = kop.coordinate_system,
        logo_html = logo_html,
        top_labels_html = top_labels_html,
        bottom_labels_html = bottom_labels_html,
        left_labels_html = left_labels_html,
        right_labels_html = right_labels_html,
        grid_svg = grid_svg,
        scale_bar_svg = scale_bar_svg,
        c_cut_deep = config.color_cut_deep,
        l_cut_deep = config.label_cut_deep,
        c_cut_high = config.color_cut_high,
        l_cut_high = config.label_cut_high,
        c_cut_mid = config.color_cut_mid,
        l_cut_mid = config.label_cut_mid,
        c_cut_low = config.color_cut_low,
        l_cut_low = config.label_cut_low,
        c_cut_near = config.color_cut_near,
        l_cut_near = config.label_cut_near,
        c_cut_to_grade = config.color_cut_to_grade,
        l_cut_to_grade = config.label_cut_to_grade,
        c_ongrade = config.color_ongrade,
        l_ongrade = config.label_ongrade,
        c_fill_to_grade = config.color_fill_to_grade,
        l_fill_to_grade = config.label_fill_to_grade,
        c_fill_near = config.color_fill_near,
        l_fill_near = config.label_fill_near,
        c_fill_low = config.color_fill_low,
        l_fill_low = config.label_fill_low,
        c_fill_mid = config.color_fill_mid,
        l_fill_mid = config.label_fill_mid,
        c_fill_high = config.color_fill_high,
        l_fill_high = config.label_fill_high,
        c_fill_deep = config.color_fill_deep,
        l_fill_deep = config.label_fill_deep,
        cut_vol = format_number_with_commas(summary.cut_m3),
        fill_vol = format_number_with_commas(summary.fill_m3),
        net_vol = format_number_with_commas(summary.net_m3),
        raster_json = raster_json,
        design_lines_json = design_lines_json,
        palette_js = palette_js,
        map_min_x = map_min_x,
        map_max_x = map_max_x,
        map_min_y = map_min_y,
        map_max_y = map_max_y,
    )
}
