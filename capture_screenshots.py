#!/usr/bin/env python3
"""
=============================================================================
RAINBOW CONTOUR - AUTOMATED SCREENSHOT GENERATOR
=============================================================================
Automates taking crisp, pixel-perfect screenshots of:
  1. A4 Landscape Kop Map (preview.png)
  2. Mining Kop Sidebar & Volume Summary Box (summary.png)

Directly from rainbow-viewer.html using Google Chrome headless rendering
and automated layout boundary detection.

Usage:
  python3 capture_screenshots.py [options]

Examples:
  python3 capture_screenshots.py
  python3 capture_screenshots.py --html trial_intan/output/rainbow-viewer.html --sync-dashboard
  python3 capture_screenshots.py --scale 2 --sync-dashboard
=============================================================================
"""

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
import numpy as np
from PIL import Image

DEFAULT_CANDIDATE_PATHS = [
    "trial_intan/output/rainbow-viewer.html",
    "output/rainbow-viewer.html",
    "output_latest_trial/rainbow-viewer.html",
    "sample_demo_output/rainbow-viewer.html",
    "/tmp/rainbow_showcase_gen/rainbow-viewer.html",
]

DASHBOARD_ASSETS_DIR = "/home/cells/Documents/Antigravity Project/Dashboard/public/rainbow-contour"


def find_chrome_binary():
    """Locates Google Chrome or Chromium executable."""
    for cmd in ["google-chrome", "google-chrome-stable", "chromium-browser", "chromium"]:
        path = shutil.which(cmd)
        if path:
            return path
    return None


def resolve_html_path(custom_path=None):
    """Resolves the target rainbow-viewer.html to capture."""
    if custom_path:
        p = Path(custom_path).resolve()
        if p.exists():
            return str(p)
        raise FileNotFoundError(f"Specified HTML file not found: {custom_path}")

    base_dir = Path(__file__).resolve().parent
    for rel in DEFAULT_CANDIDATE_PATHS:
        p = (base_dir / rel).resolve()
        if p.exists():
            return str(p)
        if Path(rel).exists():
            return str(Path(rel).resolve())

    raise FileNotFoundError(
        "Could not find any rainbow-viewer.html. Please specify path via --html <path>"
    )


def capture_full_render(chrome_bin, html_file, scale=2):
    """Renders HTML in headless Chrome with virtual time budget for canvas."""
    tmp_dir = tempfile.mkdtemp(prefix="rainbow_shot_")
    raw_shot_path = os.path.join(tmp_dir, "raw_render.png")

    # In 2x scale, window size of 2500x1800 gives plenty of canvas space
    window_w = 2500
    window_h = 1800

    cmd = [
        chrome_bin,
        "--headless",
        "--disable-gpu",
        "--hide-scrollbars",
        "--virtual-time-budget=3000",
        "--run-all-compositor-stages-before-draw",
        f"--force-device-scale-factor={scale}",
        f"--window-size={window_w},{window_h}",
        f"--screenshot={raw_shot_path}",
        f"file://{os.path.abspath(html_file)}"
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0 or not os.path.exists(raw_shot_path):
        shutil.rmtree(tmp_dir, ignore_errors=True)
        raise RuntimeError(f"Chrome headless screenshot failed: {result.stderr or result.stdout}")

    return raw_shot_path, tmp_dir


def extract_crops(raw_image_path, scale=2):
    """
    Locates #pdf-kop-container using slate-900 border detection,
    then extracts exact preview.png and summary.png.
    """
    im = Image.open(raw_image_path)
    arr = np.array(im)

    # Outer border color is #0f172a (RGB: [15, 23, 42])
    # Search for container top border along middle column of image
    mid_x = im.width // 2
    col = arr[:, mid_x]
    is_border = (col[:, 0] == 15) & (col[:, 1] == 23) & (col[:, 2] == 42)
    border_y_indices = np.where(is_border)[0]

    # Skip top action toolbar (which has borders under y <= 100 in 1x / y <= 200 in 2x)
    min_search_y = 50 * scale
    kop_y_candidates = [y for y in border_y_indices if y > min_search_y]

    if not kop_y_candidates:
        raise ValueError("Could not find #pdf-kop-container top border in rendered page.")

    top_y = kop_y_candidates[0]

    # Find left and right container borders by scanning horizontally slightly below top_y
    scan_row = top_y + (5 * scale)
    row = arr[scan_row, :]
    row_is_border = (row[:, 0] == 15) & (row[:, 1] == 23) & (row[:, 2] == 42)
    border_x_indices = np.where(row_is_border)[0]

    if not len(border_x_indices):
        raise ValueError("Could not find #pdf-kop-container horizontal borders in rendered page.")

    left_x = border_x_indices[0]

    expected_w = int(1123 * scale)
    expected_h = int(794 * scale)

    # 1. A4 Landscape Kop Map Preview
    preview_img = im.crop((left_x, top_y, left_x + expected_w, top_y + expected_h))

    # 2. Right Sidebar Kop & Volume Summary Box
    # Geometry inside #pdf-kop-container (1123x794):
    # Sidebar width: 300px, height: 765px
    # Offset from container top-left: X: 804px, Y: 14px
    sidebar_x1 = left_x + int(804 * scale)
    sidebar_x2 = sidebar_x1 + int(300 * scale)
    sidebar_y1 = top_y + int(14 * scale)
    sidebar_y2 = sidebar_y1 + int(765 * scale)

    summary_img = im.crop((sidebar_x1, sidebar_y1, sidebar_x2, sidebar_y2))

    return preview_img, summary_img


def main():
    parser = argparse.ArgumentParser(
        description="Automated screenshot generator for Rainbow Contour A4 Map & Volume Summary."
    )
    parser.add_argument(
        "--html",
        "-i",
        help="Path to rainbow-viewer.html (auto-detected if omitted)",
        default=None,
    )
    parser.add_argument(
        "--outdir",
        "-o",
        help="Output directory for screenshots (default: ./screenshots)",
        default="screenshots",
    )
    parser.add_argument(
        "--scale",
        "-s",
        type=int,
        choices=[1, 2],
        default=2,
        help="Scale factor: 1 for standard 96dpi, 2 for 2x retina (default: 2)",
    )
    parser.add_argument(
        "--sync-dashboard",
        action="store_true",
        help="Also copy output preview.png & summary.png to Dashboard/public/rainbow-contour/",
    )
    parser.add_argument(
        "--quiet",
        "-q",
        action="store_true",
        help="Suppress informational stdout",
    )

    args = parser.parse_args()

    chrome_bin = find_chrome_binary()
    if not chrome_bin:
        sys.stderr.write("ERROR: google-chrome or chromium not found in PATH.\n")
        sys.exit(1)

    try:
        html_path = resolve_html_path(args.html)
    except FileNotFoundError as e:
        sys.stderr.write(f"ERROR: {e}\n")
        sys.exit(1)

    outdir = Path(args.outdir).resolve()
    outdir.mkdir(parents=True, exist_ok=True)

    if not args.quiet:
        print(f"Target HTML : {html_path}")
        print(f"Scale Factor: {args.scale}x ({'Retina Hi-DPI' if args.scale == 2 else 'Standard 96dpi'})")
        print(f"Output Dir  : {outdir}")

    # Render headless
    raw_shot, tmp_dir = capture_full_render(chrome_bin, html_path, scale=args.scale)

    try:
        preview_img, summary_img = extract_crops(raw_shot, scale=args.scale)

        preview_out = outdir / "preview.png"
        summary_out = outdir / "summary.png"

        preview_img.save(preview_out, format="PNG")
        summary_img.save(summary_out, format="PNG")

        if not args.quiet:
            print(f"[OK] Wrote preview: {preview_out} ({preview_img.size[0]}x{preview_img.size[1]} px)")
            print(f"[OK] Wrote summary: {summary_out} ({summary_img.size[0]}x{summary_img.size[1]} px)")

        # Sync to dashboard if requested or available
        dashboard_dir = Path(DASHBOARD_ASSETS_DIR).resolve()
        if args.sync_dashboard:
            if dashboard_dir.exists():
                shutil.copy(preview_out, dashboard_dir / "preview.png")
                shutil.copy(summary_out, dashboard_dir / "summary.png")
                if not args.quiet:
                    print(f"[OK] Synced screenshots to Dashboard: {dashboard_dir}")
            else:
                sys.stderr.write(f"WARNING: Dashboard directory not found at {dashboard_dir}\n")

    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)

    if not args.quiet:
        print("Done! Screenshots successfully updated.")


if __name__ == "__main__":
    main()
