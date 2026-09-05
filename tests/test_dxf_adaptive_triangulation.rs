use rainbow_contour::dxf::{parse_dxf_mesh_with_params, triangulate_points_delaunay_with_max_edge, Point3D};

#[test]
fn test_dxf_adaptive_densification() {
    // DXF with a small polyline: 20m segment from (0,0,10) to (20,0,12)
    let dxf_content = r#"0
SECTION
2
ENTITIES
0
POLYLINE
8
CREST
0
VERTEX
10
0.0
20
0.0
30
10.0
0
VERTEX
10
20.0
20
0.0
30
12.0
0
VERTEX
10
20.0
20
10.0
30
12.0
0
VERTEX
10
0.0
20
10.0
30
10.0
0
SEQEND
0
ENDSEC
0
EOF"#;

    // With supplement_max_dist = 2.0m, a 20m edge must be subdivided into at least 10 segments
    let triangles = parse_dxf_mesh_with_params(dxf_content, 0.1, 2.0, 50.0).expect("Should parse mesh");
    assert!(triangles.len() > 10, "Adaptive densification should produce fine triangles, got {}", triangles.len());
}

#[test]
fn test_small_surface_triangulation_bounds() {
    let pts = vec![
        Point3D { x: 0.0, y: 0.0, z: 10.0 },
        Point3D { x: 10.0, y: 0.0, z: 10.0 },
        Point3D { x: 10.0, y: 10.0, z: 10.0 },
        Point3D { x: 0.0, y: 10.0, z: 10.0 },
        // Outlier point far away (150m)
        Point3D { x: 150.0, y: 150.0, z: 10.0 },
    ];

    // Restrict max edge to 15.0m for small surface
    let triangles = triangulate_points_delaunay_with_max_edge(&pts, 15.0);
    assert_eq!(triangles.len(), 2, "Only the 10x10 quad should be triangulated; outlier should be filtered");
    for tri in &triangles {
        assert!(tri.max_edge_len_sq() <= 15.0 * 15.0);
    }
}
