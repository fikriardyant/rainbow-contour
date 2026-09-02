use rainbow_contour::dxf::{Point3D, Triangle3D};
use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::{generate_html_viewer, KopInfo};
use rainbow_contour::marching_squares::generate_isolines;
use rainbow_contour::volume::calculate_volume;
use std::fs;
use std::path::Path;

#[test]
fn test_generate_sample_output_artifacts() {
    let mut topo_mesh = Vec::new();
    let mut design_mesh = Vec::new();

    let step = 5.0f64;
    let mut x = 0.0f64;
    while x < 100.0 {
        let mut y = 0.0f64;
        while y < 100.0 {
            let z_t0 = 10.0 + (x * 0.1) + (y * 0.05);
            let z_t1 = 10.0 + ((x + step) * 0.1) + (y * 0.05);
            let z_t2 = 10.0 + (x * 0.1) + ((y + step) * 0.05);

            topo_mesh.push(Triangle3D {
                v0: Point3D { x, y, z: z_t0 },
                v1: Point3D {
                    x: x + step,
                    y,
                    z: z_t1,
                },
                v2: Point3D {
                    x,
                    y: y + step,
                    z: z_t2,
                },
            });

            let dx: f64 = x - 50.0;
            let dy: f64 = y - 50.0;
            let dist_center = (dx * dx + dy * dy).sqrt();

            let z_d0 = if dist_center < 30.0 { 5.0 } else { 18.0 };
            let z_d1 = if dist_center < 30.0 { 5.0 } else { 18.0 };
            let z_d2 = if dist_center < 30.0 { 5.0 } else { 18.0 };

            design_mesh.push(Triangle3D {
                v0: Point3D { x, y, z: z_d0 },
                v1: Point3D {
                    x: x + step,
                    y,
                    z: z_d1,
                },
                v2: Point3D {
                    x,
                    y: y + step,
                    z: z_d2,
                },
            });

            y += step;
        }
        x += step;
    }

    let boundary = vec![
        Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Point3D {
            x: 100.0,
            y: 0.0,
            z: 0.0,
        },
        Point3D {
            x: 100.0,
            y: 100.0,
            z: 0.0,
        },
        Point3D {
            x: 0.0,
            y: 100.0,
            z: 0.0,
        },
    ];

    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, 1.0);
    let volume = calculate_volume(&grid, 1.0);
    let isolines = generate_isolines(&grid, 1.0, vec![-8.0, -5.0, -2.5, 0.0, 2.5, 5.0, 8.0]);
    let dxf_vector = export_isolines_to_dxf(&isolines);

    let out_dir = Path::new("./sample_demo_output");
    fs::create_dir_all(out_dir).unwrap();

    let kop = KopInfo {
        title: "Sangatta Pit Alpha - July 2026",
        company: "PT PAMA PERSADA NUSANTARA",
        drawn_by: "Fikri Ardyantoro",
        date_created: "29 July 2026",
        topo_date: "28 July 2026",
        design_name: "Plan EOM July 2026",
    };
    let html_content = generate_html_viewer(&kop, &grid, &volume, &[]);
    fs::write(out_dir.join("rainbow-viewer.html"), html_content).unwrap();
    fs::write(
        out_dir.join("volume-summary.json"),
        serde_json::to_string_pretty(&volume).unwrap(),
    )
    .unwrap();
    fs::write(out_dir.join("rainbow-output.dxf"), dxf_vector).unwrap();
}
