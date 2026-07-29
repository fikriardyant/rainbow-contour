use std::fs;
use std::process::Command;

#[test]
fn test_end_to_end_pipeline_with_synthetic_dxf() {
    let temp_dir = std::env::temp_dir().join("rainbow_test_e2e");
    let _ = fs::create_dir_all(&temp_dir);

    let topo_dxf = temp_dir.join("topo.dxf");
    let design_dxf = temp_dir.join("design.dxf");
    let boundary_dxf = temp_dir.join("boundary.dxf");
    let out_dir = temp_dir.join("output");

    let sample_topo = r#"0
SECTION
2
ENTITIES
0
3DFACE
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
10.0
12
10.0
22
10.0
32
10.0
13
0.0
23
10.0
33
10.0
0
ENDSEC
0
EOF"#;

    let sample_design = r#"0
SECTION
2
ENTITIES
0
3DFACE
10
0.0
20
0.0
30
15.0
11
10.0
21
0.0
31
15.0
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
15.0
0
ENDSEC
0
EOF"#;

    let sample_boundary = r#"0
SECTION
2
ENTITIES
0
POLYLINE
0
VERTEX
10
0.0
20
0.0
30
0.0
0
VERTEX
10
5.0
20
0.0
30
0.0
0
VERTEX
10
5.0
20
5.0
30
0.0
0
VERTEX
10
0.0
20
5.0
30
0.0
0
SEQEND
0
ENDSEC
0
EOF"#;

    fs::write(&topo_dxf, sample_topo).unwrap();
    fs::write(&design_dxf, sample_design).unwrap();
    fs::write(&boundary_dxf, sample_boundary).unwrap();

    let status = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--topo",
            topo_dxf.to_str().unwrap(),
            "--design",
            design_dxf.to_str().unwrap(),
            "--boundary",
            boundary_dxf.to_str().unwrap(),
            "--step",
            "1.0",
            "--outdir",
            out_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute cargo run");

    assert!(status.success());
    assert!(out_dir.join("rainbow-viewer.html").exists());
    assert!(out_dir.join("volume-summary.json").exists());
}
