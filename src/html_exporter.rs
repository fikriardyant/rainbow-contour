use crate::dxf::StyledPolyline;
use crate::grid_engine::GridPointDelta;
use crate::volume::VolumeSummary;

pub struct KopInfo<'a> {
    pub title: &'a str,
    pub company: &'a str,
    pub drawn_by: &'a str,
    pub date_created: &'a str,
    pub topo_date: &'a str,
    pub design_name: &'a str,
    pub logo_data_uri: Option<&'a str>,
}

pub fn format_number_with_commas(val: f64) -> String {
    let formatted = format!("{:.2}", val);
    let mut split = formatted.split('.');
    let int_part = split.next().unwrap_or("0");
    let dec_part = split.next().unwrap_or("00");

    let is_negative = int_part.starts_with('-');
    let raw_int = if is_negative { &int_part[1..] } else { int_part };

    let mut result = String::new();
    let chars: Vec<char> = raw_int.chars().rev().collect();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    let formatted_int: String = result.chars().rev().collect();
    if is_negative {
        format!("-{}.{}", formatted_int, dec_part)
    } else {
        format!("{}.{}", formatted_int, dec_part)
    }
}

#[derive(serde::Serialize)]
struct CompactStyledLine<'a> {
    c: &'a str,
    pts: Vec<[f64; 2]>,
}

#[derive(serde::Serialize)]
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

fn delta_z_to_color_id(dz: f64) -> u32 {
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
    } else if dz > 0.0 {
        6
    } else if dz == 0.0 {
        7
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

    // Efficient RLE Raster Encoding:
    // Compresses millions of [x, y, dz] tuples from 60MB-260MB down to <500KB JSON payload.
    // Instantaneous client-side canvas decompression with zero memory lag.
    let step = if grid.len() > 1 {
        // Estimate step from grid sample
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
            grid_bytes[gy * cols + gx] = delta_z_to_color_id(p.delta_z) as u8;
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

    // Serialize design styled polyline vectors with true DXF ACI / Layer colors
    let compact_design_lines: Vec<CompactStyledLine> = design_lines
        .iter()
        .map(|pl| CompactStyledLine {
            c: &pl.color_hex,
            pts: pl.points.iter().map(|pt| [(pt.x * 100.0).round() / 100.0, (pt.y * 100.0).round() / 100.0]).collect(),
        })
        .collect();
    let design_lines_json = serde_json::to_string(&compact_design_lines).unwrap_or_else(|_| "[]".to_string());

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

                const canvas = await html2canvasFn(element, {{
                    scale: 2,
                    useCORS: true,
                    logging: false,
                    backgroundColor: '#ffffff',
                    width: 1123,
                    height: 794,
                    windowWidth: 1123,
                    windowHeight: 794,
                    scrollX: 0,
                    scrollY: 0
                }});

                const imgData = canvas.toDataURL('image/png', 1.0);
                const {{ jsPDF }} = window.jspdf;
                const pdf = new jsPDF({{
                    orientation: 'landscape',
                    unit: 'mm',
                    format: 'a4',
                    compress: true
                }});

                const pdfWidth = pdf.internal.pageSize.getWidth();
                const pdfHeight = pdf.internal.pageSize.getHeight();

                pdf.addImage(imgData, 'PNG', 0, 0, pdfWidth, pdfHeight);
                pdf.save('Peta_Rainbow_Contour_' + (new Date().toISOString().slice(0,10)) + '.pdf');
            }} catch (err) {{
                alert('Gagal membuat PDF: ' + err.message);
            }}
        }}
    </script>
</head>
<body class="bg-slate-100 p-6 font-cad-title text-slate-900 flex flex-col justify-start items-center min-h-screen gap-4">
    <!-- Top Action Bar (Outside Paper Sheet / Fixed Layout) -->
    <header class="w-[1123px] flex justify-between items-center bg-white border-2 border-slate-900 p-3 shadow-[4px_4px_0px_rgba(15,23,42,1)]">
        <div class="flex items-center gap-3">
            <span class="text-xl">🌈</span>
            <div>
                <h1 class="font-black text-sm uppercase tracking-tight text-slate-900">RAINBOW CONTOUR ENGINE</h1>
                <p class="text-[10px] text-slate-500 font-cad-mono uppercase">A4 Landscape Map Preview &amp; Export</p>
            </div>
        </div>
        <button id="export-btn" onclick="downloadPDF()" class="bg-yellow-300 hover:bg-yellow-400 border-2 border-slate-900 px-4 py-2 font-black text-xs uppercase shadow-[2px_2px_0px_rgba(15,23,42,1)] active:translate-x-0.5 active:translate-y-0.5 flex items-center justify-center gap-2 cursor-pointer transition-colors">
            📥 DOWNLOAD PDF KOP
        </button>
    </header>

    <!-- Main A4 Landscape Map Layout Sheet -->
    <div id="pdf-kop-container" class="map-frame shadow-2xl">
        
        <!-- Left: Map Area with Grid Coordinates Frame -->
        <div class="flex-1 flex flex-col border-2 border-slate-900 overflow-hidden relative">
            <!-- Top Coordinate Labels -->
            <div class="h-6 bg-slate-200 border-b border-slate-900 flex justify-between px-8 items-center text-[10px] font-cad-mono font-bold text-slate-800 shrink-0">
                <span>{}</span>
                <span>{}</span>
                <span>{}</span>
                <span>{}</span>
            </div>

            <div class="flex-1 flex relative overflow-hidden">
                <!-- Left Coordinate Labels: Rendered as SVG for 100% Matrix Precision in html2canvas -->
                <div class="w-10 bg-slate-200 border-r-2 border-slate-900 flex flex-col justify-around items-center py-2 shrink-0 select-none">
                    <svg width="36" height="100" class="overflow-visible">
                        <text x="-50" y="22" transform="rotate(-90)" fill="#0f172a" font-family="Consolas, monospace" font-size="10" font-weight="bold" text-anchor="middle">{}</text>
                    </svg>
                    <div class="bg-amber-200 border border-amber-400 px-1 py-0.5 rounded-[2px] flex items-center justify-center">
                        <svg width="30" height="90" class="overflow-visible">
                            <text x="-45" y="19" transform="rotate(-90)" fill="#78350f" font-family="Consolas, monospace" font-size="10" font-weight="900" text-anchor="middle">{}</text>
                        </svg>
                    </div>
                    <svg width="36" height="100" class="overflow-visible">
                        <text x="-50" y="22" transform="rotate(-90)" fill="#0f172a" font-family="Consolas, monospace" font-size="10" font-weight="bold" text-anchor="middle">{}</text>
                    </svg>
                </div>

                <!-- Main Canvas (Rainbow Contour Heatmap & Vector Overlay) -->
                <div class="flex-1 relative bg-slate-950 overflow-hidden">
                    <canvas id="contourCanvas" class="w-full h-full object-contain"></canvas>
                    
                    <!-- North Arrow Overlay (Floating Top Right Map Canvas) -->
                    <div class="absolute top-4 right-4 bg-white border-2 border-slate-900 px-2 py-1 text-center shadow flex flex-col items-center justify-center">
                        <div class="font-black text-[11px] leading-none text-slate-900">N</div>
                        <div class="text-[14px] font-black leading-none text-red-600 mt-0.5">▲</div>
                    </div>
                </div>

                <!-- Right Map Inner Grid Ticks -->
                <div class="w-2 bg-slate-200 border-l border-slate-900 shrink-0"></div>
            </div>

            <!-- Bottom Coordinate Labels -->
            <div class="h-6 bg-slate-200 border-t border-slate-900 flex justify-between px-8 items-center text-[10px] font-cad-mono font-bold text-slate-800 shrink-0">
                <span>{}</span>
                <span>{}</span>
                <span>{}</span>
                <span>{}</span>
            </div>
        </div>

        <!-- Right: Official Mine Plan Sidebar Kop -->
        <div class="w-[280px] border-2 border-slate-900 flex flex-col justify-between p-3 bg-white text-slate-900 overflow-hidden shrink-0">
            <!-- Header Block -->
            <div class="text-center border-b-2 border-slate-900 pb-2">
                {}
                <div class="mb-1.5 border border-slate-900 bg-amber-200 text-center font-black uppercase tracking-wider text-slate-900 text-[11px]" style="height: 26px; line-height: 26px;">
                    {}
                </div>
                <h1 class="font-black text-[13px] uppercase tracking-tight text-slate-900 leading-tight">PETA RAINBOW CONTOUR</h1>
                <h2 class="font-bold text-[11px] uppercase text-amber-700 mt-0.5 leading-tight">{}</h2>
            </div>

            <!-- Metadata Table -->
            <div class="border-b-2 border-slate-900 py-2 text-[10px]">
                <table class="w-full" style="border-collapse: collapse;">
                    <tr style="border-bottom: 1px solid #e2e8f0; height: 18px;">
                        <td class="font-semibold text-slate-500" style="vertical-align: middle;">Drawn By:</td>
                        <td class="font-bold text-slate-900 text-right" style="vertical-align: middle;">{}</td>
                    </tr>
                    <tr style="border-bottom: 1px solid #e2e8f0; height: 18px;">
                        <td class="font-semibold text-slate-500" style="vertical-align: middle;">Date Created:</td>
                        <td class="font-bold text-slate-900 text-right" style="vertical-align: middle;">{}</td>
                    </tr>
                    <tr style="border-bottom: 1px solid #e2e8f0; height: 18px;">
                        <td class="font-semibold text-slate-500" style="vertical-align: middle;">Topo Date:</td>
                        <td class="font-bold text-slate-900 text-right" style="vertical-align: middle;">{}</td>
                    </tr>
                    <tr style="height: 18px;">
                        <td class="font-semibold text-slate-500" style="vertical-align: middle;">Design Name:</td>
                        <td class="font-bold text-slate-900 text-right" style="vertical-align: middle;">{}</td>
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
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #991b1b; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">&gt; +16m (Cut)</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #dc2626; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">+12m to +16m</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #ef4444; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">+8m to +12m</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #f97316; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">+4m to +8m</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #fbbf24; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">+2m to +4m</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #fde047; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">0m to +2m</span>
                        </td>
                    </tr>

                    <!-- On Grade Row -->
                    <tr>
                        <td colspan="2" style="background-color: #d1fae5; border: 1px solid #10b981; height: 20px; vertical-align: middle; padding: 0 6px;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #10b981; border: 1px solid #0f172a; vertical-align: middle; margin-right: 6px;"></span><span style="vertical-align: middle; color: #064e3b; font-weight: 900; font-size: 10px; line-height: 12px;">0m (ON GRADE)</span>
                        </td>
                    </tr>

                    <!-- Fill Rows -->
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #67e8f9; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">0m to -2m</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #06b6d4; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">-2m to -4m</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #60a5fa; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">-4m to -8m</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #2563eb; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">-8m to -12m</span>
                        </td>
                    </tr>
                    <tr style="height: 15px;">
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #4338ca; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">-12m to -16m</span>
                        </td>
                        <td style="width: 50%; vertical-align: middle;">
                            <span style="display: inline-block; width: 12px; height: 12px; background-color: #581c87; border: 1px solid #0f172a; vertical-align: middle; margin-right: 4px;"></span><span style="vertical-align: middle; line-height: 12px;">&lt; -16m (Fill)</span>
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
        
        const paletteRGBA = [
            [0, 0, 0, 0],         // 0: empty/transparent
            [153, 27, 27, 255],   // 1: >16m #991b1b
            [220, 38, 38, 255],   // 2: 12..16m #dc2626
            [239, 68, 68, 255],   // 3: 8..12m #ef4444
            [249, 115, 22, 255],  // 4: 4..8m #f97316
            [251, 191, 36, 255],  // 5: 2..4m #fbbf24
            [253, 224, 71, 255],  // 6: 0..2m #fde047
            [16, 185, 129, 255],  // 7: 0m On Grade #10b981
            [103, 232, 249, 255], // 8: 0..-2m #67e8f9
            [6, 182, 212, 255],   // 9: -2..-4m #06b6d4
            [96, 165, 250, 255],  // 10: -4..-8m #60a5fa
            [37, 99, 235, 255],   // 11: -8..-12m #2563eb
            [67, 56, 202, 255],   // 12: -12..-16m #4338ca
            [88, 28, 135, 255]    // 13: <-16m #581c87
        ];

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
                    // Flip Y in bitmap so top row is maxY
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

            // Hi-DPI Scaling (Device Pixel Ratio 2x)
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

            // 1. Draw Seamless Raster Heatmap Image (0 memory lag, zero gap / anti-bolong)
            const rasterImg = buildRasterImage();
            if (rasterImg) {{
                ctx.imageSmoothingEnabled = false; // Keep sharp CAD pixel boundary
                const destW = dx * scale;
                const destH = dy * scale;
                const destX = offsetX;
                const destY = height - (offsetY + destH);

                ctx.drawImage(rasterImg, destX, destY, destW, destH);
            }}

            // 2. Draw Overlay Vector CAD Design Lines with Original Colors (Crest, Toe, Ramp, Road)
            if (designPolylines && designPolylines.length > 0) {{
                ctx.lineWidth = 1.0;

                for (let i = 0; i < designPolylines.length; i++) {{
                    const item = designPolylines[i];
                    const pts = item.pts;
                    if (!pts || pts.length < 2) continue;

                    ctx.strokeStyle = item.c || '#f8fafc';
                    ctx.beginPath();
                    for (let j = 0; j < pts.length; j++) {{
                        const [x, y] = pts[j];
                        const px = offsetX + (x - minX) * scale;
                        const py = height - (offsetY + (y - minY) * scale);

                        if (j === 0) {{
                            ctx.moveTo(px, py);
                        }} else {{
                            ctx.lineTo(px, py);
                        }}
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
        format_number_with_commas(summary.cut_m3),
        format_number_with_commas(summary.fill_m3),
        raster_json,
        design_lines_json
    )
}
