# Tech Spec: Gross Volume Calculation & 15-Band Tri-Green Transition Palette

**Date:** 2026-09-12  
**Status:** Approved by Fikri  
**Scope:** Rainbow Contour Core Engine (`src/volume.rs`, `src/config.rs`, `src/html_exporter.rs`, `src/main.rs`, `config.dat`)

---

## 1. Context & Root Cause

### 1.1 Volume Discrepancy
When running on the same Topo and Design datasets (e.g. Bodydam Intan):
- **AutoCAD Civil 3D:** CUT ~8,500 m³, FILL 10,281 m³
- **MineScape:** CUT 8,569 m³, FILL 10,431 m³
- **Rainbow Contour (Current):** CUT 7,959.50 m³, FILL 9,104.91 m³ (-7.1% Cut, -12.7% Fill)

**Root Cause:**
In `src/volume.rs`, `calculate_volume_with_tolerance` excluded grid points within the ongrade range `[-0.50m, +0.50m]` from Cut and Fill accumulation. Over a 13,947 m² ongrade area, this discarded:
- 620.38 m³ of Cut ($0.0 < \Delta Z \le 0.5\text{m}$)
- 1,302.16 m³ of Fill ($-0.5\text{m} \le \Delta Z < 0.0$)

When calculating gross volume ($\Delta Z > 0 \to \text{Cut}$, $\Delta Z < 0 \to \text{Fill}$):
- **Rainbow Contour (Gross):** CUT 8,579.88 m³ (+0.12% vs MineScape), FILL 10,407.07 m³ (-0.23% vs MineScape).

### 1.2 Visualization & Palette Needs
1. Eliminate the concept of an "On Grade" volume exclusion.
2. The legend label should display exact numerical ranges (e.g. `-0.2 - 0.2m`) rather than the text "ON GRADE".
3. Provide a natural 3-stage green transition zone around zero:
   - `-1.0 s/d -0.2m`: Dark Green (Fill to Grade)
   - `-0.2 s/d +0.2m`: Pure Green (Grade / Level Center)
   - `+0.2 s/d +1.0m`: Light Lime Green (Cut to Grade)
4. Outside $[-1.0\text{m}, +1.0\text{m}]$, resume standard vivid rainbow progression:
   - Cut: Yellow ($+1\text{m} \text{ to } +2\text{m}$) $\to$ Orange $\to$ Red $\to$ Deep Red.
   - Fill: Cyan ($-2\text{m} \text{ to } -1\text{m}$) $\to$ Light Blue $\to$ Blue $\to$ Indigo $\to$ Deep Purple.

---

## 2. Technical Architecture & Component Changes

### 2.1 `src/volume.rs`
- Volume calculation logic will accumulate gross volume:
  - Every point with $\Delta Z > 0.0$ contributes to `cut_m3 += delta_z * cell_area`.
  - Every point with $\Delta Z < 0.0$ contributes to `fill_m3 += delta_z.abs() * cell_area`.
- Update `calculate_volume` as the primary standard calculation function.
- Retain backward compatibility if helper functions are imported in existing tests, but `main.rs` pipeline uses the gross calculation.

### 2.2 `src/config.rs` (15 Bands Configuration)
Expand palette and labels from 13 entries to 15 entries:

| ID | Category | Elevation Range $\Delta Z$ | Default Hex | Name / Purpose |
|:---|:---|:---|:---|:---|
| 1 | Cut | $> 16.0\text{m}$ | `#7F0000` | Deep Cut (Dark Maroon) |
| 2 | Cut | $12.0 - 16.0\text{m}$ | `#B71C1C` | High Cut (Dark Red) |
| 3 | Cut | $8.0 - 12.0\text{m}$ | `#D50000` | Mid Cut (Red) |
| 4 | Cut | $4.0 - 8.0\text{m}$ | `#FF3D00` | Low Cut (Orange-Red) |
| 5 | Cut | $2.0 - 4.0\text{m}$ | `#FF9100` | Near Cut (Orange) |
| 6 | Cut | $1.0 - 2.0\text{m}$ | `#FFE600` | Minor Cut (Yellow) |
| 7 | Cut | $0.2 - 1.0\text{m}$ | `#76FF03` | Cut to Grade (Light Lime Green) |
| 8 | Level | $-0.2 - 0.2\text{m}$ | `#00E676` | On Grade / Level (Emerald Green) |
| 9 | Fill | $-1.0 - -0.2\text{m}$ | `#1B5E20` | Fill to Grade (Dark Forest Green) |
| 10 | Fill | $-2.0 - -1.0\text{m}$ | `#00E5FF` | Minor Fill (Cyan) |
| 11 | Fill | $-4.0 - -2.0\text{m}$ | `#00B0FF` | Near Fill (Light Blue) |
| 12 | Fill | $-8.0 - -4.0\text{m}$ | `#2979FF` | Low Fill (Blue) |
| 13 | Fill | $-12.0 - -8.0\text{m}$ | `#0039CB` | Mid Fill (Medium Blue) |
| 14 | Fill | $-16.0 - -12.0\text{m}$ | `#4A148C` | High Fill (Indigo) |
| 15 | Fill | $< -16.0\text{m}$ | `#311B92` | Deep Fill (Deep Purple) |

Default labels:
- `label_cut_deep`: `> 16m`
- `label_cut_high`: `12-16m`
- `label_cut_mid`: `8-12m`
- `label_cut_low`: `4-8m`
- `label_cut_near`: `2-4m`
- `label_cut_minor`: `1-2m`
- `label_cut_to_grade`: `0.2-1m`
- `label_ongrade`: `-0.2 - 0.2m`
- `label_fill_to_grade`: `-1 - -0.2m`
- `label_fill_minor`: `-2 - -1m`
- `label_fill_near`: `2-4m`
- `label_fill_low`: `4-8m`
- `label_fill_mid`: `8-12m`
- `label_fill_high`: `12-16m`
- `label_fill_deep`: `> 16m`

### 2.3 `src/html_exporter.rs`
1. `delta_z_to_color_id(dz: f64) -> u32`:
   - Returns 1..=15 based on the elevation boundaries above.
2. `paletteRGBA` in JavaScript:
   - 16 entries (index 0 = transparent, index 1..15 = color IDs 1..15).
3. LEGEND Table in Kop:
   - Render 15 swatches cleanly.
   - Row 1: `> 16m` | `12-16m`
   - Row 2: `8-12m` | `4-8m`
   - Row 3: `2-4m` | `1-2m`
   - Row 4: `0.2 - 1m` (Light Lime Green)
   - Row 5: `-0.2 - 0.2m` (Emerald Green - centered full width)
   - Row 6: `-1 - -0.2m` (Dark Forest Green)
   - Row 7: `-2 - -1m` | `2-4m` (Fill)
   - Row 8: `4-8m` | `8-12m` (Fill)
   - Row 9: `12-16m` | `> 16m` (Fill)
   Compact row height (padding 1.5px) to fit smoothly within the existing flex KOP container without pushing Volume Summary down.

### 2.4 `config.dat`
Update configuration file keys and descriptions to reflect:
- New color variables and label variables for `1-2m` (Cut Minor) and `-2 - -1m` (Fill Minor).
- Updated default values for `label_ongrade=-0.2 - 0.2m`.

---

## 3. Testing & Verification

1. **Volume Test:**
   - Verify unit test `test_volume_calculation`: Cut and Fill do not have deadband loss.
   - On Bodydam Intan sample dataset: Cut matches ~8,580 m³ ($\pm 0.2\%$), Fill matches ~10,407 m³ ($\pm 0.3\%$).
2. **Palette & Color ID Test:**
   - Test `delta_z_to_color_id` across boundary conditions:
     - $dz = 20.0 \to 1$
     - $dz = 1.5 \to 6$ (Yellow)
     - $dz = 0.5 \to 7$ (Light Green)
     - $dz = 0.0 \to 8$ (Pure Green)
     - $dz = -0.5 \to 9$ (Dark Green)
     - $dz = -1.5 \to 10$ (Cyan)
     - $dz = -20.0 \to 15$ (Deep Purple)
3. **HTML Exporter E2E Test:**
   - HTML output contains valid 16-element palette array in JS.
   - All 15 swatches and labels are rendered in the HTML legend.
   - Ensure no layout shift or visual truncation on A4 landscape.
