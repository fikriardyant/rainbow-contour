use rainbow_contour::dxf::parse_dxf_mesh;

#[test]
fn test_parse_dxf_with_polylines_and_triangulate() {
    // Synthetic DXF with 3D POLYLINE and VERTEX
    let dxf_content = r#"0
SECTION
2
ENTITIES
0
POLYLINE
8
CONTOUR
66
1
70
8
0
VERTEX
8
CONTOUR
10
0.0
20
0.0
30
10.0
70
32
0
VERTEX
8
CONTOUR
10
10.0
20
0.0
30
12.0
70
32
0
VERTEX
8
CONTOUR
10
10.0
20
10.0
30
15.0
70
32
0
VERTEX
8
CONTOUR
10
0.0
20
10.0
30
11.0
70
32
0
SEQEND
0
ENDSEC
0
EOF"#;

    let triangles = parse_dxf_mesh(dxf_content).expect("Should parse mesh from polylines");
    assert!(!triangles.is_empty(), "Triangles should be generated via Delaunay triangulation");
    assert_eq!(triangles.len(), 2, "4 quad corners should triangulate into 2 triangles");
}
