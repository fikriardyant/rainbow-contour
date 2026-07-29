use rainbow_contour::dxf::{parse_dxf_mesh, Point3D, Triangle3D};

#[test]
fn test_parse_simple_dxf_3dface() {
    let dxf_content = r#"
0
SECTION
2
ENTITIES
0
3DFACE
8
TOPO_LAYER
10
0.0
20
0.0
30
10.0
11
10.0
21
0.0
31
12.0
12
10.0
22
10.0
32
15.0
13
0.0
23
10.0
33
11.0
0
ENDSEC
0
EOF
"#;

    let triangles = parse_dxf_mesh(dxf_content).unwrap();
    assert_eq!(triangles.len(), 2);
    assert_eq!(triangles[0].v0, Point3D { x: 0.0, y: 0.0, z: 10.0 });
}
