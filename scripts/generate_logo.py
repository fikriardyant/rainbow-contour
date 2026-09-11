#!/usr/bin/env python3
"""
Generative 1:1 Corporate Mining Emblem Generator
Creates a 1024x1024 pixel-perfect vector-rasterized corporate mining logo
for PT MINING NUSANTARA PRIMA.
"""
import os
import subprocess
import tempfile
from PIL import Image

def generate_logo(output_path="company_logo.png"):
    svg_content = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">
  <defs>
    <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#1E293B"/>
      <stop offset="100%" stop-color="#0F172A"/>
    </linearGradient>
    <linearGradient id="goldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#F59E0B"/>
      <stop offset="100%" stop-color="#D97706"/>
    </linearGradient>
    <linearGradient id="emeraldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#10B981"/>
      <stop offset="100%" stop-color="#047857"/>
    </linearGradient>
    <linearGradient id="cyanGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#38BDF8"/>
      <stop offset="100%" stop-color="#0284C7"/>
    </linearGradient>
    <filter id="shadow" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="16" stdDeviation="24" flood-color="#000000" flood-opacity="0.4"/>
    </filter>
  </defs>

  <!-- Circular Outer Badge -->
  <circle cx="512" cy="512" r="480" fill="url(#bgGrad)" stroke="#334155" stroke-width="16" filter="url(#shadow)"/>
  
  <!-- Outer Geometric Contour Rings -->
  <circle cx="512" cy="512" r="430" fill="none" stroke="#475569" stroke-width="4" stroke-dasharray="16 12"/>
  <circle cx="512" cy="512" r="390" fill="none" stroke="#64748B" stroke-width="6"/>

  <!-- Stylized Isometric Pit Hexagon / Strata Steps -->
  <!-- Upper Topo Surface Prism (Amber/Gold) -->
  <polygon points="512,180 772,330 512,480 252,330" fill="url(#goldGrad)" stroke="#0F172A" stroke-width="8"/>
  
  <!-- West Pit Bench Wall (Emerald Green) -->
  <polygon points="252,345 504,490 504,750 252,605" fill="url(#emeraldGrad)" stroke="#0F172A" stroke-width="8"/>
  
  <!-- East Pit Bench Wall (Cyan Blue) -->
  <polygon points="520,490 772,345 772,605 520,750" fill="url(#cyanGrad)" stroke="#0F172A" stroke-width="8"/>

  <!-- Internal Contour Step Terraces -->
  <polyline points="290,400 504,520 504,570 330,470" fill="#065F46" opacity="0.6"/>
  <polyline points="734,400 520,520 520,570 694,470" fill="#0369A1" opacity="0.6"/>

  <!-- Compass Star Zenith Point -->
  <circle cx="512" cy="330" r="22" fill="#FFFFFF" stroke="#0F172A" stroke-width="6"/>
  <line x1="512" y1="280" x2="512" y2="380" stroke="#0F172A" stroke-width="6" stroke-linecap="round"/>
  <line x1="462" y1="330" x2="562" y2="330" stroke="#0F172A" stroke-width="6" stroke-linecap="round"/>

  <!-- Base Inscription Arc -->
  <path d="M 330 830 Q 512 870 694 830" fill="none" stroke="#F59E0B" stroke-width="12" stroke-linecap="round"/>
</svg>"""

    with tempfile.NamedTemporaryFile("w", suffix=".svg", delete=False) as f:
        f.write(svg_content)
        svg_file = f.name

    tmp_png = output_path + ".tmp.png"
    cmd = [
        "google-chrome",
        "--headless",
        "--disable-gpu",
        "--default-background-color=00000000",
        "--window-size=1024,1024",
        f"--screenshot={tmp_png}",
        f"file://{os.path.abspath(svg_file)}"
    ]
    subprocess.run(cmd, check=True, capture_output=True)
    os.unlink(svg_file)

    # Ensure exact 1024x1024 crop
    with Image.open(tmp_png) as im:
        cropped = im.crop((0, 0, 1024, 1024))
        cropped.save(output_path, "PNG")
    os.unlink(tmp_png)
    print(f"Generated 1:1 corporate logo at: {output_path}")

if __name__ == "__main__":
    generate_logo()
