use crate::config::{hex_to_rgba, EngineConfig};
use crate::dxf::StyledPolyline;
use crate::grid_engine::GridPointDelta;
use crate::volume::VolumeSummary;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct KopInfo<'a> {
    pub title: &'a str,
    pub company: &'a str,
    pub drawn_by: &'a str,
    pub date_created: &'a str,
    pub topo_date: &'a str,
    pub design_name: &'a str,
    pub logo_data_uri: Option<&'a str>,
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

    let mid_x = (min_x + max_x) / 2.0;
    let mid_y = (min_y + max_y) / 2.0;

    let mid_lon = 115.0 + (mid_x - 200000.0) / 111320.0;
    let mid_lat = -((10000000.0 - mid_y) / 110574.0).abs();

    let coord_top_left = format!("{:.0} mE", min_x);
    let coord_top_mid_geo = format!("{:.2}°E", mid_lon);
    let coord_top_mid = format!("{:.0} mE", mid_x);
    let coord_top_right = format!("{:.0} mE", max_x);

    let coord_left_top = format!("{:.0} mN", max_y);
    let coord_left_mid_geo = format!("{:.2}°S", mid_lat.abs());
    let coord_left_bottom = format!("{:.0} mN", min_y);

    let step = if grid.len() > 1 {
        let mut min_diff = f64::INFINITY;
        for i in 0..grid.len().min(100) {
            for j in (i + 1)..grid.len().min(100) {
                let dx = (grid[i].x - grid[j].x).abs();
                let dy = (grid[i].y - grid[j].y).abs();
                if dx > 0.001 && dx < min_diff {
                    min_diff = dx;
                }
                if dy > 0.001 && dy < min_diff {
                    min_diff = dy;
                }
            }
        }
        if min_diff.is_finite() && min_diff >= 0.1 {
            (min_diff * 100.0).round() / 100.0
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
            format!(r#"<div class="mb-2 flex items-center justify-center border-b border-slate-200 pb-1.5"><img src="{}" alt="Company Logo" class="max-w-[260px] max-h-[93px] w-auto h-auto object-contain" /></div>"#, uri)
        } else {
            String::new()
        }
    } else {
        String::new()
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
    <title>PETA RAINBOW CONTOUR • {}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/html2canvas/1.4.1/html2canvas.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/jspdf/2.5.1/jspdf.umd.min.js"></script>
    <style>
        @page {{ size: A4 landscape; margin: 0; }}
        *, *::before, *::after {{ box-sizing: border-box; }}

        /* Technical CAD Font Stack */
        .font-cad-mono {{
            font-family: 'Consolas', 'Courier New', monospace;
        }}
        .font-cad-title {{
            font-family: 'Arial Narrow', 'Arial', sans-serif;
            font-stretch: condensed;
        }}

        /* Strict Fixed Sheet Geometry */
        #pdf-kop-container {{
            width: 1123px !important;
            height: 794px !important;
            min-width: 1123px !important;
            min-height: 794px !important;
            max-width: 1123px !important;
            max-height: 794px !important;
            background-color: #ffffff;
            border: 3px solid #0f172a;
            position: relative;
            display: flex;
            padding: 12px;
            gap: 12px;
            box-sizing: border-box;
            overflow: hidden;
        }}

        /* Fix Tailwind Preflight vertical shift in html2canvas */
        img, svg, video, canvas, audio, iframe, embed, object {{
            display: inline-block !important;
            vertical-align: middle !important;
        }}

        #contourCanvas {{
            width: 100% !important;
            height: 100% !important;
            display: block !important;
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

                // Explicitly render canvas to raster snapshot before html2canvas capture
                if (typeof drawContour === 'function') {{
                    drawContour();
                }}

                const canvas = await html2canvasFn(element, {{
                    scale: 2,
                    useCORS: true,
                    allowTaint: true,
                    backgroundColor: '#ffffff',
                    logging: false,
                    scrollX: 0,
                    scrollY: 0,
                    windowWidth: 1123,
                    windowHeight: 794
                }});

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
    <div class="w-[1123px] flex items-center justify-between mb-3 bg-white p-3 border-2 border-slate-900 shadow-[4px_4px_0px_rgba(15,23,42,1)]">
        <div class="flex items-center gap-2">
            <span class="inline-block w-3 h-3 bg-red-600 rounded-full animate-ping"></span>
            <span class="font-black text-xs uppercase tracking-widest text-slate-900">PETA KONTUR CUT & FILL • SELESAI</span>
        </div>
        <button id="export-btn" onclick="downloadPDF()" class="bg-yellow-300 hover:bg-yellow-400 border-2 border-slate-900 px-4 py-2 font-black text-xs uppercase shadow-[2px_2px_0px_rgba(15,23,42,1)] active:translate-x-0.5 active:translate-y-0.5 flex items-center justify-center gap-2 cursor-pointer transition-colors">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
            DOWNLOAD PDF (A4 LANDSCAPE)
        </button>
    </div>

    <!-- Exact A4 Landscape Container (297mm x 210mm = 1123px x 794px @ 96dpi) -->
    <div id="pdf-kop-container" class="shadow-[8px_8px_0px_rgba(15,23,42,1)]">
        
        <!-- Left: Map Canvas & Ruler Box -->
        <div class="flex-1 h-full border-2 border-slate-900 relative flex flex-col bg-slate-900 overflow-hidden">
            <!-- Top Coordinate Bar -->
            <div class="h-6 bg-white border-b-2 border-slate-900 flex items-center justify-between px-3 text-[10px] font-bold text-slate-700 tracking-wider">
                <span>{}</span>
                <span class="text-slate-400">{}</span>
                <span>{}</span>
                <span>{}</span>
            </div>

            <div class="flex-1 flex relative overflow-hidden">
                <!-- Left Coordinate Labels: Rendered as SVG for 100% Matrix Precision in html2canvas -->
                <div class="w-10 bg-white border-r-2 border-slate-900 flex flex-col justify-around items-center py-2 shrink-0 select-none">
                    <svg width="36" height="100" class="overflow-visible">
                        <text x="-50" y="22" transform="rotate(-90)" fill="#0f172a" font-family="Consolas, monospace" font-size="10" font-weight="bold" text-anchor="middle">{}</text>
                    </svg>
                    <div class="bg-amber-100 border border-amber-300 px-1 py-0.5 rounded-[2px] flex items-center justify-center">
                        <svg width="30" height="90" class="overflow-visible">
                            <text x="-45" y="19" transform="rotate(-90)" fill="#78350f" font-family="Consolas, monospace" font-size="10" font-weight="900" text-anchor="middle">{}</text>
                        </svg>
                    </div>
                    <svg width="36" height="100" class="overflow-visible">
                        <text x="-50" y="22" transform="rotate(-90)" fill="#0f172a" font-family="Consolas, monospace" font-size="10" font-weight="bold" text-anchor="middle">{}</text>
                    </svg>
                </div>

                <!-- Main Canvas Area -->
                <div class="flex-1 h-full relative bg-slate-950">
                    <canvas id="contourCanvas" class="w-full h-full block"></canvas>
                </div>
            </div>

            <!-- Bottom Ruler Coordinate Bar -->
            <div class="h-5 bg-white border-t-2 border-slate-900 flex items-center justify-between px-3 text-[9px] font-bold text-slate-700 tracking-wider">
                <span>{}</span>
                <span class="text-slate-400">{}</span>
                <span>{}</span>
                <span>{}</span>
            </div>
        </div>

        <!-- Right: Official Mining KOP Sidebar -->
        <div class="w-[300px] h-full flex flex-col justify-between border-2 border-slate-900 p-3 bg-white font-sans text-xs">
            <!-- Header Logo & Identity -->
            <div class="border-b-2 border-slate-900 pb-2">
                {}
                <div class="font-black text-sm uppercase tracking-tight text-slate-900 text-center leading-tight">{}</div>
                <div class="text-[9px] text-slate-500 font-bold uppercase tracking-widest text-center mt-0.5">MINING OPERATIONS & ENGINEERING</div>
            </div>

            <!-- Map Title -->
            <div class="border-b-2 border-slate-900 py-2 text-center bg-yellow-100 -mx-3 px-3 border-t-2 border-slate-900">
                <div class="text-[9px] font-black uppercase text-yellow-800 tracking-widest">MAP TITLE</div>
                <div class="font-black text-base uppercase text-slate-900 tracking-tight leading-snug">{}</div>
                <div class="text-[9px] font-bold text-slate-600">CUT & FILL ISOPACH DIFFERENCE</div>
            </div>

            <!-- Metadata Info Table -->
            <div class="border-b-2 border-slate-900 py-1.5 text-[10px]">
                <table class="w-full" style="border-collapse: collapse;">
                    <tr style="height: 18px;">
                        <td class="font-semibold text-slate-500" style="width: 42%; vertical-align: middle;">Drawn By:</td>
                        <td class="font-bold text-slate-900 text-right" style="width: 58%; vertical-align: middle;">{}</td>
                    </tr>
                    <tr style="height: 18px;">
                        <td class="font-semibold text-slate-500" style="vertical-align: middle;">Date Created:</td>
                        <td class="font-bold text-slate-900 text-right" style="vertical-align: middle;">{}</td>
                    </tr>
                    <tr style="height: 18px;">
                        <td class="font-semibold text-slate-500" style="vertical-align: middle;">Survey/Topo Date:</td>
                        <td class="font-bold text-slate-900 text-right" style="vertical-align: middle;">{}</td>
                    </tr>
                    <tr style="height: 18px;">
                        <td class="font-semibold text-slate-500" style="width: 40%; vertical-align: middle;">Design Name:</td>
                        <td class="font-bold text-slate-900 text-right" style="width: 60%; vertical-align: middle; word-break: break-all;">{}</td>
                    </tr>
                </table>
            </div>

            <!-- Rainbow Legend Block -->
            <div class="border-b-2 border-slate-900 py-2">
                <h3 class="font-black text-[10px] uppercase text-slate-700 tracking-wider mb-1.5">KETERANGAN / LEGEND (DELTA Z)</h3>
                
                <table class="w-full text-[9px] font-bold" style="border-collapse: collapse;">
                    <!-- Cut Rows -->
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                    </tr>

                    <!-- On Grade Row -->
                    <tr>
                        <td colspan="2" style="background-color: #f1f5f9; border: 1px solid #0f172a; height: 20px; vertical-align: middle; padding: 0 6px;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 6px;"></span><span style="vertical-align: middle; color: #0f172a; font-weight: 900; font-size: 9px; line-height: 12px;">{}</span>
                        </td>
                    </tr>

                    <!-- Fill Rows -->
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: {}; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">{}</span>
                        </td>
                    </tr>
                </table>
            </div>

            <!-- Volume Summary Box (No Net) -->
            <table class="w-full bg-slate-100 border-2 border-slate-900 font-cad-mono" style="border-collapse: collapse;">
                <tr style="border-bottom: 1px solid #0f172a;">
                    <td colspan="2" class="font-black text-[10px] uppercase text-slate-900 font-cad-title" style="padding: 4px 8px 3px 8px; vertical-align: middle;">
                        VOLUME SUMMARY
                    </td>
                </tr>
                <tr style="height: 20px;">
                    <td class="font-bold text-red-600 font-cad-title text-xs" style="padding: 2px 8px; vertical-align: middle;">CUT:</td>
                    <td class="font-black text-slate-900 text-xs text-right" style="padding: 2px 8px; vertical-align: middle;">{} m³</td>
                </tr>
                <tr style="height: 20px;">
                    <td class="font-bold text-blue-600 font-cad-title text-xs" style="padding: 2px 8px; vertical-align: middle;">FILL:</td>
                    <td class="font-black text-slate-900 text-xs text-right" style="padding: 2px 8px; vertical-align: middle;">{} m³</td>
                </tr>
            </table>
        </div>
    </div>

    <!-- Ultra-Fast Canvas Renderer with RLE Raster Decompression & CAD Vector Layer -->
    <script>
        const rasterData = {};
        const designPolylines = {};
        
        const paletteRGBA = {};

        let offscreenCanvas = null;

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

            // Background
            ctx.fillStyle = '#020617';
            ctx.fillRect(0, 0, width, height);

            // Engineering Grid
            ctx.strokeStyle = '#1e293b';
            ctx.lineWidth = 1;
            for (let x = 0; x < width; x += 40) {{
                ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, height); ctx.stroke();
            }}
            for (let y = 0; y < height; y += 40) {{
                ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(width, y); ctx.stroke();
            }}

            const minX = rasterData.min_x || 0;
            const maxX = rasterData.max_x || 100;
            const minY = rasterData.min_y || 0;
            const maxY = rasterData.max_y || 100;

            const dx = (maxX - minX) || 100;
            const dy = (maxY - minY) || 100;

            const padding = 30;
            const availW = width - padding * 2;
            const availH = height - padding * 2;

            const scale = Math.min(availW / dx, availH / dy);
            const offsetX = (width - dx * scale) / 2;
            const offsetY = (height - dy * scale) / 2;

            // 1. Draw Seamless Raster Heatmap Image
            const rasterImg = buildRasterImage();
            if (rasterImg) {{
                ctx.imageSmoothingEnabled = false;
                const destW = dx * scale;
                const destH = dy * scale;
                const destX = offsetX;
                const destY = height - (offsetY + destH);

                ctx.drawImage(rasterImg, destX, destY, destW, destH);
            }}

            // 2. Draw Overlay Vector CAD Design Lines
            if (designPolylines && designPolylines.length > 0) {{
                ctx.lineWidth = 1.0;
                for (const pl of designPolylines) {{
                    if (!pl.pts || pl.pts.length < 2) continue;
                    ctx.strokeStyle = pl.c || '#ffffff';
                    ctx.beginPath();
                    for (let i = 0; i < pl.pts.length; i++) {{
                        const px = offsetX + (pl.pts[i][0] - minX) * scale;
                        const py = height - (offsetY + (pl.pts[i][1] - minY) * scale);
                        if (i === 0) ctx.moveTo(px, py);
                        else ctx.lineTo(px, py);
                    }}
                    ctx.stroke();
                }}
            }}
        }}

        window.addEventListener('resize', drawContour);
        window.addEventListener('DOMContentLoaded', drawContour);
        setTimeout(drawContour, 100);
    </script>
</body>
</html>"##,
        kop.title,
        coord_top_left,
        coord_top_mid_geo,
        coord_top_mid,
        coord_top_right,
        coord_left_top,
        coord_left_mid_geo,
        coord_left_bottom,
        coord_top_left,
        coord_top_mid_geo,
        coord_top_mid,
        coord_top_right,
        logo_html,
        kop.company,
        kop.title,
        kop.drawn_by,
        kop.date_created,
        kop.topo_date,
        kop.design_name,
        // Legend swatch colors & dynamic text from config
        config.color_cut_deep,
        config.label_cut_deep,
        config.color_cut_high,
        config.label_cut_high,
        config.color_cut_mid,
        config.label_cut_mid,
        config.color_cut_low,
        config.label_cut_low,
        config.color_cut_near,
        config.label_cut_near,
        config.color_cut_to_grade,
        config.label_cut_to_grade,
        config.color_ongrade,
        config.label_ongrade,
        config.color_fill_to_grade,
        config.label_fill_to_grade,
        config.color_fill_near,
        config.label_fill_near,
        config.color_fill_low,
        config.label_fill_low,
        config.color_fill_mid,
        config.label_fill_mid,
        config.color_fill_high,
        config.label_fill_high,
        config.color_fill_deep,
        config.label_fill_deep,
        format_number_with_commas(summary.cut_m3),
        format_number_with_commas(summary.fill_m3),
        raster_json,
        design_lines_json,
        palette_js
    )
}
