mod cli;
use clap::Parser;
use rainbow_contour::config::EngineConfig;
use rainbow_contour::dxf::{parse_dxf_boundary, parse_dxf_mesh};
use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::{generate_html_viewer, KopInfo};
use rainbow_contour::marching_squares::generate_isolines;
use rainbow_contour::volume::calculate_volume;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn prompt_input(prompt_text: &str, required: bool, default_val: &str) -> String {
    loop {
        if !default_val.is_empty() {
            print!("{} [{}]: ", prompt_text, default_val);
        } else {
            print!("{}: ", prompt_text);
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || input.is_empty() {
            return default_val.to_string();
        }

        let trimmed = input.trim().trim_matches('\'').trim_matches('"').to_string();

        if trimmed.is_empty() {
            if !default_val.is_empty() {
                return default_val.to_string();
            }
            if required {
                println!("  [Error] This field is required.");
                continue;
            } else {
                return String::new();
            }
        }
        return trimmed;
    }
}

fn prompt_file_path(prompt_text: &str, required: bool) -> String {
    loop {
        let trimmed = prompt_input(prompt_text, required, "");
        if trimmed.is_empty() && !required {
            return String::new();
        }
        if Path::new(&trimmed).exists() {
            return trimmed;
        } else {
            println!("  [Error] File not found: '{}'. Please check the path and try again.", trimmed);
        }
    }
}

pub fn extract_design_name_default(design_path: &str) -> String {
    let p = Path::new(design_path);
    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
        if !stem.is_empty() {
            return stem.to_string();
        }
    }
    "Plan EOM Design".to_string()
}

fn main() {
    let args = cli::CliArgs::parse();
    println!("======================================================================");
    println!("  RAINBOW CONTOUR - Cut & Fill Difference Map Generator v1.0");
    println!("======================================================================");

    // 1. Load or auto-generate config.dat
    let mut config = EngineConfig::load_or_create(&args.config);
    if let Some(c) = args.company { if !c.is_empty() { config.company_name = c; } }
    if let Some(t) = args.rainbow_title { if !t.is_empty() { config.default_title = t; } }
    if let Some(d) = args.drawn_by { if !d.is_empty() { config.drawn_by = d; } }
    if let Some(s) = args.step { config.grid_step = s; }
    if let Some(o) = args.outdir { if !o.is_empty() { config.default_outdir = o; } }

    let topo_path = match args.topo {
        Some(path) if !path.is_empty() => path,
        _ => prompt_file_path("[1/3] Enter Topo DXF file path", true),
    };

    let design_path = match args.design {
        Some(path) if !path.is_empty() => path,
        _ => prompt_file_path("[2/3] Enter Design DXF file path", true),
    };

    let boundary_path = match args.boundary {
        Some(path) if !path.is_empty() => path,
        _ => prompt_file_path("[3/3] Enter Boundary DXF path (opt)", false),
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

    println!("Parsing Topo & Design 3D meshes (max edge: {:.0}m)...", config.max_tin_edge);
    let topo_mesh = parse_dxf_mesh(&topo_content).expect("Failed to parse Topo mesh");
    let design_mesh = parse_dxf_mesh(&design_content).expect("Failed to parse Design mesh");
    let design_lines = rainbow_contour::dxf::parse_dxf_styled_polylines(&design_content);
    let mut boundary = parse_dxf_boundary(&boundary_content).expect("Failed to parse Boundary");
    if boundary.is_empty() || (boundary.len() <= 4 && boundary[0].x == 0.0 && boundary[1].x == 100.0) {
        if let Some(detected) = rainbow_contour::dxf::auto_detect_closed_boundary(&design_lines) {
            println!("Auto-detected pit outer boundary crest limit ({} vertices)", detected.len());
            boundary = detected;
        }
    }

    println!("Computing spatial grid delta (step = {}m)...", config.grid_step);
    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, config.grid_step);
    let volume = calculate_volume(&grid, config.grid_step);

    println!("Generating Marching Squares contour isolines...");
    let isolines = generate_isolines(&grid, config.grid_step, config.contour_levels.clone());
    let dxf_vector = export_isolines_to_dxf(&isolines);

    println!("\n----------------------------------------------------------------------");
    println!(
        "Volume Summary: Cut = {:.2} m³ | Fill = {:.2} m³ | Net = {:.2} m³",
        volume.cut_m3, volume.fill_m3, volume.net_m3
    );

    let topo_date = match args.topo_date {
        Some(td) if !td.is_empty() => td,
        _ => prompt_input("Enter Topo Survey Date", true, "28 July 2026"),
    };

    // Auto-detect default Design Name from Design DXF filename
    let default_design_name = extract_design_name_default(&design_path);
    let design_name = match args.design_name {
        Some(dn) if !dn.is_empty() => dn,
        _ => prompt_input("Enter Design Name", true, &default_design_name),
    };

    let date_created_str = "29 July 2026".to_string();

    let kop_info = KopInfo {
        title: &config.default_title,
        company: &config.company_name,
        drawn_by: &config.drawn_by,
        date_created: &date_created_str,
        topo_date: &topo_date,
        design_name: &design_name,
    };

    let out_dir = Path::new(&config.default_outdir);
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    let html_content = generate_html_viewer(&kop_info, &grid, &volume, &design_lines);
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
