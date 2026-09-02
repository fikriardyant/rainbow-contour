mod cli;
use clap::Parser;
use rainbow_contour::dxf::{parse_dxf_boundary, parse_dxf_mesh};
use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::{generate_html_viewer, KopInfo};
use rainbow_contour::marching_squares::generate_isolines;
use rainbow_contour::volume::calculate_volume;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub company_name: String,
    pub rainbow_title: String,
    pub drawn_by: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            company_name: "PT PAMA PERSADA NUSANTARA".to_string(),
            rainbow_title: "PIT A CUT & FILL MAP".to_string(),
            drawn_by: "Fikri Ardyantoro".to_string(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".rainbow_contour_config.json")
}

pub fn load_or_init_config(
    arg_company: Option<String>,
    arg_title: Option<String>,
    arg_drawn_by: Option<String>,
) -> AppConfig {
    let config_path = get_config_path();
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str::<AppConfig>(&content).unwrap_or_default()
    } else {
        println!("\n======================================================================");
        println!("  FIRST TIME SETUP - Initial Setup (Saved to ~/.rainbow_contour_config.json)");
        println!("======================================================================");

        let company = prompt_input("Enter Default Company Name [PT PAMA PERSADA NUSANTARA]: ", false, "PT PAMA PERSADA NUSANTARA");
        let title = prompt_input("Enter Default Rainbow Name [PIT A CUT & FILL MAP]: ", false, "PIT A CUT & FILL MAP");
        let drawn = prompt_input("Enter Default Drawn By Name [Fikri Ardyantoro]: ", false, "Fikri Ardyantoro");

        let new_cfg = AppConfig {
            company_name: company,
            rainbow_title: title,
            drawn_by: drawn,
        };

        if let Ok(json) = serde_json::to_string_pretty(&new_cfg) {
            let _ = fs::write(&config_path, json);
        }
        println!("----------------------------------------------------------------------\n");
        new_cfg
    };

    if let Some(c) = arg_company { if !c.is_empty() { config.company_name = c; } }
    if let Some(t) = arg_title { if !t.is_empty() { config.rainbow_title = t; } }
    if let Some(d) = arg_drawn_by { if !d.is_empty() { config.drawn_by = d; } }

    config
}

fn prompt_input(prompt_text: &str, required: bool, default_val: &str) -> String {
    loop {
        print!("{}", prompt_text);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || input.is_empty() {
            // EOF or pipe closed
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
    let design_lines = rainbow_contour::dxf::parse_dxf_styled_polylines(&design_content);
    let mut boundary = parse_dxf_boundary(&boundary_content).expect("Failed to parse Boundary");
    if boundary.is_empty() || (boundary.len() <= 4 && boundary[0].x == 0.0 && boundary[1].x == 100.0) {
        if let Some(detected) = rainbow_contour::dxf::auto_detect_closed_boundary(&design_lines) {
            println!("Auto-detected pit outer boundary crest limit ({} vertices)", detected.len());
            boundary = detected;
        }
    }

    println!("Computing spatial grid delta (step = {}m)...", args.step);
    let grid = compute_grid_delta(&topo_mesh, &design_mesh, &boundary, args.step);
    let volume = calculate_volume(&grid, args.step);

    println!("Generating Marching Squares contour isolines...");
    let levels = vec![
        -20.0, -18.0, -16.0, -14.0, -12.0, -10.0, -8.0, -6.0, -4.0, -2.0,
        0.0,
        2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0,
    ];
    let isolines = generate_isolines(&grid, args.step, levels);
    let dxf_vector = export_isolines_to_dxf(&isolines);

    println!("\n----------------------------------------------------------------------");
    println!(
        "Volume Summary: Cut = {:.2} m³ | Fill = {:.2} m³ | Net = {:.2} m³",
        volume.cut_m3, volume.fill_m3, volume.net_m3
    );

    let config = load_or_init_config(args.company, args.rainbow_title, args.drawn_by);

    let topo_date = match args.topo_date {
        Some(td) if !td.is_empty() => td,
        _ => prompt_input("Enter Topo Survey Date (e.g. 28 July 2026): ", true, "28 July 2026"),
    };

    let design_name = match args.design_name {
        Some(dn) if !dn.is_empty() => dn,
        _ => prompt_input("Enter Design Name (e.g. Plan EOM July 2026): ", true, "Plan EOM July 2026"),
    };

    let date_created_str = "29 July 2026".to_string();

    let kop_info = KopInfo {
        title: &config.rainbow_title,
        company: &config.company_name,
        drawn_by: &config.drawn_by,
        date_created: &date_created_str,
        topo_date: &topo_date,
        design_name: &design_name,
    };

    let out_dir = Path::new(&args.outdir);
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
