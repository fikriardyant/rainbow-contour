mod cli;
use clap::Parser;
use rainbow_contour::dxf::{parse_dxf_boundary, parse_dxf_mesh};
use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::generate_html_viewer;
use rainbow_contour::marching_squares::generate_isolines;
use rainbow_contour::volume::calculate_volume;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn prompt_file_path(prompt_text: &str, required: bool) -> String {
    loop {
        print!("{}", prompt_text);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return String::new();
        }

        let trimmed = input.trim().trim_matches('\'').trim_matches('"').to_string();

        if trimmed.is_empty() {
            if required {
                println!("  [Error] This file path is required. Please enter a valid path.");
                continue;
            } else {
                return String::new();
            }
        }

        if Path::new(&trimmed).exists() {
            return trimmed;
        } else {
            println!("  [Error] File not found: '{}'. Please check the path and try again.", trimmed);
        }
    }
}

fn main() {
    let args = cli::CliArgs::parse();
    println!("======================================================================");
    println!("  RAINBOW CONTOUR - Cut & Fill Difference Map Generator v1.0");
    println!("======================================================================");

    let topo_path = match args.topo {
        Some(path) if !path.is_empty() => path,
        _ => prompt_file_path("[1/3] Enter Topo DXF file path     : ", true),
    };

    let design_path = match args.design {
        Some(path) if !path.is_empty() => path,
        _ => prompt_file_path("[2/3] Enter Design DXF file path   : ", true),
    };

    let boundary_path = match args.boundary {
        Some(path) if !path.is_empty() => path,
        _ => prompt_file_path("[3/3] Enter Boundary DXF path (opt): ", false),
    };

    println!("\n----------------------------------------------------------------------");
    println!("Reading Topo & Design DXF files...");
    let topo_content = fs::read_to_string(&topo_path).expect("Failed to read Topo DXF");
    let design_content = fs::read_to_string(&design_path).expect("Failed to read Design DXF");
    let boundary_content = if !boundary_path.is_empty() {
        fs::read_to_string(&boundary_path).unwrap_or_default()
    } else {
        String::new()
    };

    println!("Parsing Topo & Design 3D meshes...");
    let topo_mesh = parse_dxf_mesh(&topo_content).expect("Failed to parse Topo mesh");
    let design_mesh = parse_dxf_mesh(&design_content).expect("Failed to parse Design mesh");
    let boundary = parse_dxf_boundary(&boundary_content).expect("Failed to parse Boundary");

    println!("Computing spatial grid delta (step = {}m)...", args.step);
    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, args.step);
    let volume = calculate_volume(&grid, args.step);

    println!("Generating Marching Squares contour isolines...");
    let isolines = generate_isolines(&grid, args.step, vec![-5.0, -2.5, -1.0, 0.0, 1.0, 2.5, 5.0]);
    let dxf_vector = export_isolines_to_dxf(&isolines);

    println!("\n----------------------------------------------------------------------");
    println!(
        "Volume Summary: Cut = {:.2} m³ | Fill = {:.2} m³ | Net = {:.2} m³",
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

    fs::write(out_dir.join("rainbow-output.dxf"), dxf_vector)
        .expect("Failed to write rainbow-output.dxf");

    println!("----------------------------------------------------------------------");
    println!("SUCCESS! Output generated in '{}':", out_dir.display());
    println!("  - Viewer      : {}/rainbow-viewer.html", out_dir.display());
    println!("  - Vector DXF  : {}/rainbow-output.dxf", out_dir.display());
    println!("  - Volume JSON : {}/volume-summary.json", out_dir.display());
    println!("======================================================================\n");
}
