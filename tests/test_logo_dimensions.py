# tests/test_logo_dimensions.py
import os
from PIL import Image

def test_company_logo_aspect_ratio_and_resolution():
    logo_path = "company_logo.png"
    assert os.path.exists(logo_path), f"{logo_path} must exist"
    with Image.open(logo_path) as im:
        w, h = im.size
        assert w == h, f"Logo must be 1:1 square ratio, got {w}x{h}"
        assert w >= 512, f"Logo must be at least 512px resolution, got {w}"
        assert im.mode in ("RGBA", "RGB"), f"Logo mode must be RGBA or RGB, got {im.mode}"

if __name__ == "__main__":
    test_company_logo_aspect_ratio_and_resolution()
    print("Logo dimensions test passed!")
