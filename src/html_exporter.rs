use crate::grid_engine::GridPointDelta;
use crate::volume::VolumeSummary;

pub struct KopInfo<'a> {
    pub title: &'a str,
    pub company: &'a str,
    pub drawn_by: &'a str,
    pub date_created: &'a str,
    pub topo_date: &'a str,
    pub design_name: &'a str,
}

pub fn generate_html_viewer(
    kop: &KopInfo,
    grid: &[GridPointDelta],
    summary: &VolumeSummary,
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
                <div class="flex-1 bg-slate-900 relative overflow-hidden flex items-center justify-center">
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
                <h1 class="font-black text-[13px] uppercase tracking-tight text-slate-900 leading-tight">PETA RAINBOW CONTOUR</h1>
                <h2 class="font-bold text-[11px] uppercase text-amber-700 mt-0.5 leading-tight">{}</h2>
                <div class="mt-2 border border-slate-900 bg-amber-200 text-center font-black uppercase tracking-wider text-slate-900 text-[11px]" style="height: 26px; line-height: 26px;">
                    {}
                </div>
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
                    <td class="font-black text-slate-900 text-xs text-right" style="padding: 2px 8px; vertical-align: middle;">{:.2} m³</td>
                </tr>
                <tr style="height: 20px;">
                    <td class="font-bold text-blue-600 font-cad-title text-xs" style="padding: 2px 8px; vertical-align: middle;">FILL:</td>
                    <td class="font-black text-slate-900 text-xs text-right" style="padding: 2px 8px; vertical-align: middle;">{:.2} m³</td>
                </tr>
            </table>
        </div>
    </div>
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
        kop.title,
        kop.company,
        kop.drawn_by,
        kop.date_created,
        kop.topo_date,
        kop.design_name,
        summary.cut_m3,
        summary.fill_m3
    )
}
