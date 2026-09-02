use delaunator::{triangulate, Point as DelPoint};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Triangle3D {
    pub v0: Point3D,
    pub v1: Point3D,
    pub v2: Point3D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledPolyline {
    pub points: Vec<Point3D>,
    pub color_hex: String,
}

impl Triangle3D {
    pub fn max_edge_len_sq(&self) -> f64 {
        let d01 = (self.v0.x - self.v1.x).powi(2) + (self.v0.y - self.v1.y).powi(2);
        let d12 = (self.v1.x - self.v2.x).powi(2) + (self.v1.y - self.v2.y).powi(2);
        let d20 = (self.v2.x - self.v0.x).powi(2) + (self.v2.y - self.v0.y).powi(2);
        d01.max(d12).max(d20)
    }
}

pub fn aci_to_hex(aci: i32) -> &'static str {
    match aci {
        1 => "#ef4444",   // Red (Crest / Main)
        2 => "#facc15",   // Yellow
        3 => "#22c55e",   // Green (Toe)
        4 => "#06b6d4",   // Cyan
        5 => "#3b82f6",   // Blue (Drainage/Ramp)
        6 => "#d946ef",   // Magenta
        7 => "#f8fafc",   // White / Light
        8 => "#64748b",   // Dark Gray
        9 => "#94a3b8",   // Light Gray
        10..=19 => "#ef4444",
        20..=29 => "#f97316",
        30..=39 => "#f59e0b",
        40..=49 => "#eab308",
        50..=69 => "#84cc16",
        70..=119 => "#10b981",
        120..=159 => "#06b6d4",
        160..=199 => "#3b82f6",
        200..=239 => "#a855f7",
        240..=249 => "#ec4899",
        250..=252 => "#475569",
        253..=255 => "#94a3b8",
        _ => "#f8fafc",
    }
}

/// Weeding & Supplementing:
/// 1. Weeds out points closer than min_dist (0.5m).
/// 2. Supplements (subdivides) segments longer than max_dist (20.0m) to prevent triangulator gaps.
pub fn process_polyline_points(points: &[Point3D], min_dist: f64, max_dist: f64) -> Vec<Point3D> {
    if points.is_empty() {
        return Vec::new();
    }
    let min_dist_sq = min_dist * min_dist;
    let mut processed: Vec<Point3D> = Vec::with_capacity(points.len());
    
    for i in 0..points.len() {
        let p = points[i];
        if let Some(&prev) = processed.last() {
            let dx = p.x - prev.x;
            let dy = p.y - prev.y;
            let dz = p.z - prev.z;
            let d_sq = dx * dx + dy * dy;

            // Weeding filter
            if d_sq < min_dist_sq && i + 1 < points.len() {
                continue;
            }

            // Supplementing interpolation
            let d = d_sq.sqrt();
            if d > max_dist {
                let num_subdivisions = (d / max_dist).ceil() as usize;
                for step in 1..num_subdivisions {
                    let t = step as f64 / num_subdivisions as f64;
                    processed.push(Point3D {
                        x: prev.x + dx * t,
                        y: prev.y + dy * t,
                        z: prev.z + dz * t,
                    });
                }
            }
        }
        processed.push(p);
    }
    
    processed
}

/// Triangulate 3D points using 2D Delaunay triangulation (XY projection)
/// with max edge limit = 300m to accommodate wide pit floors, crest-toe spans, and benches
pub fn triangulate_points_delaunay(points: &[Point3D]) -> Vec<Triangle3D> {
    if points.len() < 3 {
        return Vec::new();
    }

    let del_points: Vec<DelPoint> = points
        .iter()
        .map(|p| DelPoint { x: p.x, y: p.y })
        .collect();

    let result = triangulate(&del_points);
    let mut triangles = Vec::with_capacity(result.triangles.len() / 3);

    // Max allowed edge length for mining surface interpolation (300m)
    let max_edge_sq = 300.0 * 300.0;

    for i in (0..result.triangles.len()).step_by(3) {
        let i0 = result.triangles[i];
        let i1 = result.triangles[i + 1];
        let i2 = result.triangles[i + 2];

        if i0 < points.len() && i1 < points.len() && i2 < points.len() {
            let tri = Triangle3D {
                v0: points[i0],
                v1: points[i1],
                v2: points[i2],
            };
            if tri.max_edge_len_sq() <= max_edge_sq {
                triangles.push(tri);
            }
        }
    }

    triangles
}

pub fn parse_dxf_styled_polylines(content: &str) -> Vec<StyledPolyline> {
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut layer_colors: HashMap<String, i32> = HashMap::new();
    
    // 1. First pass: parse layer table colors
    let mut in_tables = false;
    let mut cur_layer_name = String::new();
    let mut cur_layer_color = 7;
    let mut idx = 0;

    while idx < lines.len() {
        if lines[idx] == "0" && idx + 1 < lines.len() {
            let tag = lines[idx + 1];
            if tag == "TABLE" && idx + 3 < lines.len() && lines[idx + 2] == "2" && lines[idx + 3] == "LAYER" {
                in_tables = true;
                idx += 4;
                continue;
            } else if in_tables && tag == "ENDTAB" {
                if !cur_layer_name.is_empty() {
                    layer_colors.insert(cur_layer_name.clone(), cur_layer_color);
                }
                in_tables = false;
            } else if in_tables && tag == "LAYER" {
                if !cur_layer_name.is_empty() {
                    layer_colors.insert(cur_layer_name.clone(), cur_layer_color);
                }
                cur_layer_name = String::new();
                cur_layer_color = 7;
            }
        }
        if in_tables {
            if lines[idx] == "2" && idx + 1 < lines.len() {
                cur_layer_name = lines[idx + 1].to_string();
            } else if lines[idx] == "62" && idx + 1 < lines.len() {
                cur_layer_color = lines[idx + 1].parse().unwrap_or(7);
            }
        }
        idx += 1;
    }

    // 2. Second pass: parse entities with color inheritance
    let mut polylines = Vec::new();
    let mut current_pl = Vec::new();
    let mut in_polyline = false;
    let mut in_entities = false;
    let mut cur_pl_color: Option<i32> = None;
    let mut cur_pl_layer = String::new();

    idx = 0;
    while idx < lines.len() {
        if lines[idx] == "0" && idx + 1 < lines.len() {
            let tag = lines[idx + 1];
            if tag == "SECTION" && idx + 3 < lines.len() && lines[idx + 2] == "2" && lines[idx + 3] == "ENTITIES" {
                in_entities = true;
                idx += 4;
                continue;
            } else if tag == "ENDSEC" {
                in_entities = false;
            }

            if in_entities {
                if tag == "POLYLINE" {
                    if in_polyline && !current_pl.is_empty() {
                        let color = cur_pl_color.or_else(|| layer_colors.get(&cur_pl_layer).copied()).unwrap_or(7);
                        polylines.push(StyledPolyline {
                            points: current_pl,
                            color_hex: aci_to_hex(color).to_string(),
                        });
                        current_pl = Vec::new();
                    }
                    in_polyline = true;
                    cur_pl_color = None;
                    cur_pl_layer = String::new();
                } else if tag == "SEQEND" {
                    if in_polyline && !current_pl.is_empty() {
                        let color = cur_pl_color.or_else(|| layer_colors.get(&cur_pl_layer).copied()).unwrap_or(7);
                        polylines.push(StyledPolyline {
                            points: current_pl,
                            color_hex: aci_to_hex(color).to_string(),
                        });
                        current_pl = Vec::new();
                    }
                    in_polyline = false;
                } else if tag == "VERTEX" {
                    let mut pt = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "10" => pt.x = val.parse().unwrap_or(0.0),
                                "20" => pt.y = val.parse().unwrap_or(0.0),
                                "30" => pt.z = val.parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                        idx += 2;
                    }
                    if pt.x.abs() < 1e8 && pt.y.abs() < 1e8 {
                        current_pl.push(pt);
                    }
                    continue;
                } else if tag == "LWPOLYLINE" {
                    let mut lw_pts: Vec<Point3D> = Vec::new();
                    let mut lw_color = None;
                    let mut lw_layer = String::new();
                    let mut lw_elev = 0.0;
                    let mut cur_x: Option<f64> = None;

                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "8" => lw_layer = val.to_string(),
                                "62" => lw_color = val.parse().ok(),
                                "38" => lw_elev = val.parse().unwrap_or(0.0),
                                "10" => cur_x = val.parse().ok(),
                                "20" => {
                                    if let (Some(x), Ok(y)) = (cur_x, val.parse::<f64>()) {
                                        if x.abs() < 1e8 && y.abs() < 1e8 {
                                            lw_pts.push(Point3D { x, y, z: lw_elev });
                                        }
                                        cur_x = None;
                                    }
                                }
                                _ => {}
                            }
                        }
                        idx += 2;
                    }

                    if !lw_pts.is_empty() {
                        let color = lw_color.or_else(|| layer_colors.get(&lw_layer).copied()).unwrap_or(7);
                        polylines.push(StyledPolyline {
                            points: lw_pts,
                            color_hex: aci_to_hex(color).to_string(),
                        });
                    }
                    continue;
                } else if tag == "LINE" {
                    let mut p0 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    let mut p1 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    let mut line_color = None;
                    let mut line_layer = String::new();

                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "10" => p0.x = val.parse().unwrap_or(0.0),
                                "20" => p0.y = val.parse().unwrap_or(0.0),
                                "30" => p0.z = val.parse().unwrap_or(0.0),
                                "11" => p1.x = val.parse().unwrap_or(0.0),
                                "21" => p1.y = val.parse().unwrap_or(0.0),
                                "31" => p1.z = val.parse().unwrap_or(0.0),
                                "8" => line_layer = val.to_string(),
                                "62" => line_color = val.parse().ok(),
                                _ => {}
                            }
                        }
                        idx += 2;
                    }
                    if p0.x.abs() < 1e8 && p1.x.abs() < 1e8 {
                        let color = line_color.or_else(|| layer_colors.get(&line_layer).copied()).unwrap_or(7);
                        polylines.push(StyledPolyline {
                            points: vec![p0, p1],
                            color_hex: aci_to_hex(color).to_string(),
                        });
                    }
                    continue;
                }
            }
        }
        if in_polyline {
            if lines[idx] == "8" && idx + 1 < lines.len() {
                cur_pl_layer = lines[idx + 1].to_string();
            } else if lines[idx] == "62" && idx + 1 < lines.len() {
                cur_pl_color = lines[idx + 1].parse().ok();
            }
        }
        idx += 1;
    }

    if in_polyline && !current_pl.is_empty() {
        let color = cur_pl_color.or_else(|| layer_colors.get(&cur_pl_layer).copied()).unwrap_or(7);
        polylines.push(StyledPolyline {
            points: current_pl,
            color_hex: aci_to_hex(color).to_string(),
        });
    }

    polylines
}

pub fn parse_dxf_mesh(content: &str) -> Result<Vec<Triangle3D>, String> {
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut triangles = Vec::new();
    let mut all_points = Vec::new();
    let mut in_entities = false;

    let mut current_pl_pts = Vec::new();
    let mut in_pl = false;

    let mut idx = 0;

    while idx < lines.len() {
        if lines[idx] == "0" && idx + 1 < lines.len() {
            let entity_type = lines[idx + 1];
            if entity_type == "SECTION" && idx + 3 < lines.len() && lines[idx + 2] == "2" && lines[idx + 3] == "ENTITIES" {
                in_entities = true;
                idx += 4;
                continue;
            } else if entity_type == "ENDSEC" {
                in_entities = false;
            }

            if in_entities {
                if entity_type == "3DFACE" {
                    let mut p0 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    let mut p1 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    let mut p2 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    let mut p3 = Point3D { x: 0.0, y: 0.0, z: 0.0 };

                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "10" => p0.x = val.parse().unwrap_or(0.0),
                                "20" => p0.y = val.parse().unwrap_or(0.0),
                                "30" => p0.z = val.parse().unwrap_or(0.0),
                                "11" => p1.x = val.parse().unwrap_or(0.0),
                                "21" => p1.y = val.parse().unwrap_or(0.0),
                                "31" => p1.z = val.parse().unwrap_or(0.0),
                                "12" => p2.x = val.parse().unwrap_or(0.0),
                                "22" => p2.y = val.parse().unwrap_or(0.0),
                                "32" => p2.z = val.parse().unwrap_or(0.0),
                                "13" => p3.x = val.parse().unwrap_or(0.0),
                                "23" => p3.y = val.parse().unwrap_or(0.0),
                                "33" => p3.z = val.parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                        idx += 2;
                    }

                    if p0.x.abs() < 1e8 && p1.x.abs() < 1e8 && p2.x.abs() < 1e8 {
                        triangles.push(Triangle3D { v0: p0, v1: p1, v2: p2 });
                        if p2 != p3 {
                            triangles.push(Triangle3D { v0: p0, v1: p2, v2: p3 });
                        }
                    }
                    continue;
                } else if entity_type == "POLYLINE" {
                    if in_pl && !current_pl_pts.is_empty() {
                        let processed = process_polyline_points(&current_pl_pts, 0.5, 10.0);
                        all_points.extend(processed);
                        current_pl_pts.clear();
                    }
                    in_pl = true;
                } else if entity_type == "SEQEND" {
                    if in_pl && !current_pl_pts.is_empty() {
                        let processed = process_polyline_points(&current_pl_pts, 0.5, 10.0);
                        all_points.extend(processed);
                        current_pl_pts.clear();
                    }
                    in_pl = false;
                } else if entity_type == "VERTEX" {
                    let mut pt = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "10" => pt.x = val.parse().unwrap_or(0.0),
                                "20" => pt.y = val.parse().unwrap_or(0.0),
                                "30" => pt.z = val.parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                        idx += 2;
                    }
                    if pt.x.abs() < 1e8 && pt.y.abs() < 1e8 {
                        current_pl_pts.push(pt);
                    }
                    continue;
                } else if entity_type == "LWPOLYLINE" {
                    let mut lw_pts: Vec<Point3D> = Vec::new();
                    let mut lw_elev = 0.0;
                    let mut cur_x: Option<f64> = None;

                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "38" => lw_elev = val.parse().unwrap_or(0.0),
                                "10" => cur_x = val.parse().ok(),
                                "20" => {
                                    if let (Some(x), Ok(y)) = (cur_x, val.parse::<f64>()) {
                                        if x.abs() < 1e8 && y.abs() < 1e8 {
                                            lw_pts.push(Point3D { x, y, z: lw_elev });
                                        }
                                        cur_x = None;
                                    }
                                }
                                _ => {}
                            }
                        }
                        idx += 2;
                    }
                    if !lw_pts.is_empty() {
                        let processed = process_polyline_points(&lw_pts, 0.5, 10.0);
                        all_points.extend(processed);
                    }
                    continue;
                } else if entity_type == "POINT" {
                    let mut pt = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "10" => pt.x = val.parse().unwrap_or(0.0),
                                "20" => pt.y = val.parse().unwrap_or(0.0),
                                "30" => pt.z = val.parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                        idx += 2;
                    }
                    if pt.x.abs() < 1e8 && pt.y.abs() < 1e8 {
                        all_points.push(pt);
                    }
                    continue;
                }
            }
        }
        idx += 1;
    }

    if in_pl && !current_pl_pts.is_empty() {
        let processed = process_polyline_points(&current_pl_pts, 0.5, 10.0);
        all_points.extend(processed);
    }

    if !triangles.is_empty() {
        Ok(triangles)
    } else if !all_points.is_empty() {
        Ok(triangulate_points_delaunay(&all_points))
    } else {
        Ok(Vec::new())
    }
}

pub fn auto_detect_closed_boundary(design_lines: &[StyledPolyline]) -> Option<Vec<Point3D>> {
    // Look for closed polylines (first point == last point within tolerance)
    let mut candidates: Vec<&StyledPolyline> = design_lines
        .iter()
        .filter(|pl| {
            if pl.points.len() < 4 {
                return false;
            }
            let first = pl.points[0];
            let last = pl.points[pl.points.len() - 1];
            let d_sq = (first.x - last.x).powi(2) + (first.y - last.y).powi(2);
            d_sq < 1.0 // closed loop within 1m tolerance
        })
        .collect();

    // Pick the longest closed loop (outermost crest/pit boundary)
    candidates.sort_by_key(|pl| std::cmp::Reverse(pl.points.len()));

    candidates.first().map(|pl| pl.points.clone())
}

pub fn parse_dxf_boundary(content: &str) -> Result<Vec<Point3D>, String> {
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut points = Vec::new();
    let mut in_entities = false;
    let mut idx = 0;

    while idx < lines.len() {
        if lines[idx] == "0" && idx + 1 < lines.len() {
            let entity_type = lines[idx + 1];
            if entity_type == "SECTION" && idx + 3 < lines.len() && lines[idx + 2] == "2" && lines[idx + 3] == "ENTITIES" {
                in_entities = true;
                idx += 4;
                continue;
            } else if entity_type == "ENDSEC" {
                in_entities = false;
            }

            if in_entities {
                if entity_type == "VERTEX" || entity_type == "LWPOLYLINE" {
                    let mut p = Point3D { x: 0.0, y: 0.0, z: 0.0 };
                    idx += 2;
                    while idx < lines.len() && lines[idx] != "0" {
                        if idx + 1 < lines.len() {
                            let code = lines[idx];
                            let val = lines[idx + 1];
                            match code {
                                "10" => p.x = val.parse().unwrap_or(0.0),
                                "20" => p.y = val.parse().unwrap_or(0.0),
                                "30" => p.z = val.parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                        idx += 2;
                    }
                    if p.x.abs() < 1e8 && p.y.abs() < 1e8 {
                        points.push(p);
                    }
                    continue;
                }
            }
        }
        idx += 1;
    }

    if points.is_empty() {
        Ok(vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 100.0, y: 0.0, z: 0.0 },
            Point3D { x: 100.0, y: 100.0, z: 0.0 },
            Point3D { x: 0.0, y: 100.0, z: 0.0 },
        ])
    } else {
        Ok(points)
    }
}
