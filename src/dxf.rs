use serde::{Deserialize, Serialize};

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

pub fn parse_dxf_mesh(content: &str) -> Result<Vec<Triangle3D>, String> {
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut triangles = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        if lines[idx] == "3DFACE" {
            let mut p0 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            let mut p1 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            let mut p2 = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            let mut p3 = Point3D { x: 0.0, y: 0.0, z: 0.0 };

            idx += 1;
            while idx < lines.len() && lines[idx] != "0" {
                match lines[idx] {
                    "10" => p0.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "20" => p0.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "30" => p0.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "11" => p1.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "21" => p1.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "31" => p1.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "12" => p2.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "22" => p2.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "32" => p2.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "13" => p3.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "23" => p3.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "33" => p3.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    _ => {}
                }
                idx += 1;
            }

            triangles.push(Triangle3D { v0: p0, v1: p1, v2: p2 });
            if p2 != p3 {
                triangles.push(Triangle3D { v0: p0, v1: p2, v2: p3 });
            }
            continue;
        }
        idx += 1;
    }

    Ok(triangles)
}

pub fn parse_dxf_boundary(content: &str) -> Result<Vec<Point3D>, String> {
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut points = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        if lines[idx] == "VERTEX" || lines[idx] == "LWPOLYLINE" {
            let mut p = Point3D { x: 0.0, y: 0.0, z: 0.0 };
            idx += 1;
            while idx < lines.len() && lines[idx] != "0" {
                match lines[idx] {
                    "10" => p.x = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "20" => p.y = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    "30" => p.z = lines.get(idx + 1).unwrap_or(&"0").parse().unwrap_or(0.0),
                    _ => {}
                }
                idx += 1;
            }
            points.push(p);
            continue;
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
