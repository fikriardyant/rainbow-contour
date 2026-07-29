mod cli;
use clap::Parser;
use rainbow_contour::dxf::{parse_dxf_boundary, parse_dxf_mesh};
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::generate_html_viewer;
use rainbow_contour::volume::calculate_volume;
use std::fs;
use std::path::Path;

fn main() {
    let args = cli::CliArgs::parse();
    println!("RAINBOW CONTOUR Cut & Fill Engine v0.1.0");

    let topo_path = args.topo.unwrap_or_default();
    let design_path = args.design.unwrap_or_default();
    let boundary_path = args.boundary.unwrap_or_default();

    if topo_path.is_empty() || design_path.is_empty() {
        println!("Please provide --topo and --design DXF files.");
        return;
    }

    let topo_content = fs::read_to_string(&topo_path).expect("Failed to read Topo DXF");
    let design_content = fs::read_to_string(&design_path).expect("Failed to read Design DXF");
    let boundary_content = if !boundary_path.is_empty() {
        fs::read_to_string(&boundary_path).unwrap_or_default()
    } else {
        String::new()
    };

    println!("Parsing Topo & Design DXF meshes...");
    let topo_mesh = parse_dxf_mesh(&topo_content).expect("Failed to parse Topo mesh");
    let design_mesh = parse_dxf_mesh(&design_content).expect("Failed to parse Design mesh");
    let boundary = parse_dxf_boundary(&boundary_content).expect("Failed to parse Boundary");

    println!(
        "Computing spatial grid delta (step = {}m)...",
        args.step
    );
    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, args.step);
    let volume = calculate_volume(&grid, args.step);

    println!(
        "Volume Calculated: Cut = {} m³, Fill = {} m³, Net = {} m³",
        volume.cut_m3, volume.fill_m3, volume.net_m3
    );

    let out_dir = Path::new(&args.outdir);
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    let html_content = generate_html_viewer("Pit A Cut & Fill Map", &grid, &volume);
    fs::write(out_dir.join("rainbow-viewer.html"), html_content)
        .expect("Failed to write rainbow-viewer.html");

    let json_content = serde_json::to_string_pretty(&volume).expect("Failed to serialize volume");
    fs::write(out_dir.join("volume-summary.json"), json_content)
        .expect("Failed to write volume-summary.json");

    println!(
        "SUCCESS! Artifacts saved to {}",
        out_dir.display()
    );
}
