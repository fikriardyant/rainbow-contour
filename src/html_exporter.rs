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
    _grid: &[GridPointDelta],
    summary: &VolumeSummary,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>PETA RAINBOW CONTOUR • {}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/html2canvas/1.4.1/html2canvas.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/jspdf/2.5.1/jspdf.umd.min.js"></script>
    <style>
        @page {{ size: A4 landscape; margin: 0; }}
        .map-frame {{ border: 3px solid #0f172a; border-radius: 0px; }}
        .sidebar-border {{ border-left: 3px solid #0f172a; }}
        .section-divider {{ border-bottom: 2px solid #0f172a; }}

        /* Fixed Print & Paper Sheet Aspect Ratio Lock */
        #pdf-kop-container {{
            width: 1123px !important;
            height: 794px !important;
            min-width: 1123px !important;
            min-height: 794px !important;
            max-width: 1123px !important;
            max-height: 794px !important;
            box-sizing: border-box !important;
        }}

        /* Ensure Fixed Canvas Resolution (No responsive stretch distortion) */
        #contourCanvas {{
            width: 100% !important;
            height: 100% !important;
        }}
    </style>
    <script>
        async function downloadPDF() {{
            const actionBtnArea = document.getElementById('action-btn-area');
            const originalDisplay = actionBtnArea.style.display;
            actionBtnArea.style.display = 'none';

            try {{
                const element = document.getElementById('pdf-kop-container');
                const canvas = await html2canvas(element, {{
                    scale: 2,
                    useCORS: true,
                    logging: false,
                    backgroundColor: '#ffffff'
                }});

                const imgData = canvas.toDataURL('image/png');
                const {{ jsPDF }} = window.jspdf;
                const pdf = new jsPDF({{
                    orientation: 'landscape',
                    unit: 'mm',
                    format: 'a4'
                }});

                const pdfWidth = pdf.internal.pageSize.getWidth();
                const pdfHeight = pdf.internal.pageSize.getHeight();

                pdf.addImage(imgData, 'PNG', 0, 0, pdfWidth, pdfHeight);
                pdf.save('Peta_Rainbow_Contour_' + (new Date().toISOString().slice(0,10)) + '.pdf');
            }} catch (err) {{
                alert('Gagal membuat PDF: ' + err.message);
            }} finally {{
                actionBtnArea.style.display = originalDisplay;
            }}
        }}
    </script>
</head>
<body class="bg-slate-100 p-4 font-sans text-slate-900 flex justify-center items-center min-h-screen">
    <!-- Main A4 Landscape Map Layout Sheet -->
    <div id="pdf-kop-container" class="w-[1123px] h-[794px] bg-white map-frame flex relative p-3 gap-3 shadow-2xl overflow-hidden">
        
        <!-- Left: Map Area with Grid Coordinates Frame -->
        <div class="flex-1 flex flex-col relative border-2 border-slate-900">
            <!-- Top Coordinate Labels -->
            <div class="h-5 bg-slate-200 border-b border-slate-900 flex justify-between px-10 items-center text-[9px] font-mono font-bold text-slate-700">
                <span>273000 mE</span>
                <span>115°27'00"E</span>
                <span>274000 mE</span>
                <span>115°27'30"E</span>
                <span>275000 mE</span>
            </div>

            <div class="flex-1 flex relative">
                <!-- Left Coordinate Labels (Clean Padding, Outside Line Collision) -->
                <div class="w-10 bg-slate-200 border-r-2 border-slate-900 flex flex-col justify-between py-12 px-1 items-center text-[9px] font-mono font-bold text-slate-700 select-none">
                    <span class="[writing-mode:vertical-lr] rotate-180 tracking-tight text-slate-900">9695000 mN</span>
                    <span class="[writing-mode:vertical-lr] rotate-180 text-amber-700 bg-amber-100 px-0.5 border border-amber-300 rounded-[2px] font-extrabold my-2">03°45'00"S</span>
                    <span class="[writing-mode:vertical-lr] rotate-180 tracking-tight text-slate-900">9694000 mN</span>
                </div>

                <!-- Main Canvas (Rainbow Contour Heatmap & Vector Overlay) -->
                <div class="flex-1 bg-slate-900 relative overflow-hidden flex items-center justify-center">
                    <canvas id="contourCanvas" class="w-full h-full object-contain"></canvas>
                    
                    <!-- North Arrow Overlay (Floating Top Right Map Canvas) -->
                    <div class="absolute top-4 right-4 bg-white/90 backdrop-blur border-2 border-slate-900 px-2 py-1 text-center rounded shadow">
                        <div class="font-black text-xs text-slate-900">N</div>
                        <div class="text-[16px] font-black leading-none text-red-600">▲</div>
                    </div>
                </div>

                <!-- Right Map Inner Grid Ticks -->
                <div class="w-2 bg-slate-200 border-l border-slate-900"></div>
            </div>

            <!-- Bottom Coordinate Labels -->
            <div class="h-5 bg-slate-200 border-t border-slate-900 flex justify-between px-10 items-center text-[9px] font-mono font-bold text-slate-700">
                <span>273000 mE</span>
                <span>115°27'00"E</span>
                <span>274000 mE</span>
                <span>115°27'30"E</span>
                <span>275000 mE</span>
            </div>
        </div>

        <!-- Right: Official Mine Plan Sidebar Kop -->
        <div class="w-[280px] border-2 border-slate-900 flex flex-col justify-between p-3 bg-white text-slate-900">
            <!-- Header Block -->
            <div class="text-center section-divider pb-2">
                <h1 class="font-black text-base uppercase tracking-tight text-slate-900">PETA RAINBOW CONTOUR</h1>
                <h2 class="font-bold text-xs uppercase text-amber-600 mt-0.5">{}</h2>
                <div class="mt-2 text-xs font-black uppercase tracking-wider text-slate-800 bg-amber-200 py-1 border border-slate-900">
                    {}
                </div>
            </div>

            <!-- Metadata Table -->
            <div class="section-divider py-2 text-[10px] space-y-1">
                <div class="flex justify-between border-b border-slate-200 pb-0.5">
                    <span class="font-semibold text-slate-500">Drawn By:</span>
                    <span class="font-bold text-slate-900">{}</span>
                </div>
                <div class="flex justify-between border-b border-slate-200 pb-0.5">
                    <span class="font-semibold text-slate-500">Date Created:</span>
                    <span class="font-bold text-slate-900">{}</span>
                </div>
                <div class="flex justify-between border-b border-slate-200 pb-0.5">
                    <span class="font-semibold text-slate-500">Topo Date:</span>
                    <span class="font-bold text-slate-900">{}</span>
                </div>
                <div class="flex justify-between">
                    <span class="font-semibold text-slate-500">Design Name:</span>
                    <span class="font-bold text-slate-900">{}</span>
                </div>
            </div>

            <!-- Rainbow Legend Block -->
            <div class="section-divider py-2 space-y-1">
                <h3 class="font-black text-[10px] uppercase text-slate-700 tracking-wider">KETERANGAN / LEGEND (DELTA Z)</h3>
                <div class="grid grid-cols-2 gap-1 text-[8px] font-bold">
                    <!-- Cut Side (Red/Warm Gradient) -->
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-red-800 border border-slate-900 inline-block"></span>
                        <span>&gt; +16m (Heavy Cut)</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-red-600 border border-slate-900 inline-block"></span>
                        <span>+12m to +16m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-red-500 border border-slate-900 inline-block"></span>
                        <span>+8m to +12m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-orange-500 border border-slate-900 inline-block"></span>
                        <span>+4m to +8m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-amber-400 border border-slate-900 inline-block"></span>
                        <span>+2m to +4m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-yellow-300 border border-slate-900 inline-block"></span>
                        <span>0m to +2m (Minor Cut)</span>
                    </div>

                    <!-- On Grade -->
                    <div class="flex items-center gap-1.5 col-span-2 my-0.5 bg-emerald-100 p-0.5 border border-emerald-500">
                        <span class="w-2.5 h-2.5 bg-emerald-500 border border-slate-900 inline-block"></span>
                        <span class="text-emerald-900">0m (ON GRADE)</span>
                    </div>

                    <!-- Fill Side (Blue/Cool Gradient) -->
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-cyan-300 border border-slate-900 inline-block"></span>
                        <span>0m to -2m (Minor Fill)</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-cyan-500 border border-slate-900 inline-block"></span>
                        <span>-2m to -4m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-blue-400 border border-slate-900 inline-block"></span>
                        <span>-4m to -8m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-blue-600 border border-slate-900 inline-block"></span>
                        <span>-8m to -12m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-indigo-700 border border-slate-900 inline-block"></span>
                        <span>-12m to -16m</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <span class="w-2.5 h-2.5 bg-purple-900 border border-slate-900 inline-block"></span>
                        <span>&lt; -16m (Heavy Fill)</span>
                    </div>
                </div>
            </div>

            <!-- Volume Summary Box (No Net) -->
            <div class="bg-slate-100 border-2 border-slate-900 p-2 space-y-1">
                <h3 class="font-black text-[10px] uppercase text-slate-900 border-b border-slate-900 pb-0.5">VOLUME SUMMARY</h3>
                <div class="flex justify-between items-center text-xs">
                    <span class="font-bold text-red-600">CUT:</span>
                    <span class="font-black text-slate-900">{:.2} m³</span>
                </div>
                <div class="flex justify-between items-center text-xs">
                    <span class="font-bold text-blue-600">FILL:</span>
                    <span class="font-black text-slate-900">{:.2} m³</span>
                </div>
            </div>

            <!-- Action Button / Direct Download PDF (Hidden during PDF Capture) -->
            <div id="action-btn-area" class="pt-1">
                <button id="export-btn" onclick="downloadPDF()" class="w-full bg-yellow-300 hover:bg-yellow-400 border-2 border-slate-900 py-1.5 font-black text-xs uppercase shadow-[2px_2px_0px_rgba(15,23,42,1)] active:translate-x-0.5 active:translate-y-0.5 flex items-center justify-center gap-2">
                    📥 DOWNLOAD PDF KOP
                </button>
            </div>
        </div>
    </div>
</body>
</html>"#,
        kop.title,
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
