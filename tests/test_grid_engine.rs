use rainbow_contour::dxf::{Point3D, Triangle3D};
use rainbow_contour::grid_engine::compute_grid_delta;

#[test]
fn test_compute_grid_delta_simple() {
    let topo_mesh = vec![Triangle3D {
        v0: Point3D { x: 0.0, y: 0.0, z: 10.0 },
        v1: Point3D { x: 10.0, y: 0.0, z: 10.0 },
        v2: Point3D { x: 0.0, y: 10.0, z: 10.0 },
    }];

    let design_mesh = vec![Triangle3D {
        v0: Point3D { x: 0.0, y: 0.0, z: 15.0 },
        v1: Point3D { x: 10.0, y: 0.0, z: 15.0 },
        v2: Point3D { x: 0.0, y: 10.0, z: 15.0 },
    }];

    let boundary_polygon = vec![
        Point3D { x: 0.0, y: 0.0, z: 0.0 },
        Point3D { x: 5.0, y: 0.0, z: 0.0 },
        Point3D { x: 0.0, y: 5.0, z: 0.0 },
    ];

    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary_polygon, 1.0);
    assert!(grid.len() > 0);
    assert_eq!(grid[0].delta_z, 5.0); // 15.0 - 10.0 = 5.0 (Fill +5m)
}
