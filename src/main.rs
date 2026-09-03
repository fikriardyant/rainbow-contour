mod cli;
use clap::Parser;
use rainbow_contour::config::{
    get_current_date_string, get_file_modified_date_string, load_image_as_data_uri,
    open_file_in_default_browser, EngineConfig,
};
use rainbow_contour::dxf::{parse_dxf_boundary, parse_dxf_mesh};
use rainbow_contour::dxf_exporter::export_isolines_to_dxf;
use rainbow_contour::grid_engine::compute_grid_delta;
use rainbow_contour::html_exporter::{generate_html_viewer_with_config, KopInfo};
use rainbow_contour::marching_squares::generate_isolines;
use rainbow_contour::volume::calculate_volume_with_tolerance;
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

fn prompt_file_path(prompt_text: &str, required: bool, default_val: &str) -> String {
    loop {
        let trimmed = prompt_input(prompt_text, required, default_val);
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
    if let Some(ref tp) = args.topo { if !tp.is_empty() { config.topo_path = tp.clone(); } }
    if let Some(ref dp) = args.design { if !dp.is_empty() { config.design_path = dp.clone(); } }
    if let Some(ref c) = args.company { if !c.is_empty() { config.company_name = c.clone(); } }
    if let Some(ref t) = args.rainbow_title { if !t.is_empty() { config.default_title = t.clone(); } }
    if let Some(ref d) = args.drawn_by { if !d.is_empty() { config.drawn_by = d.clone(); } }
    if let Some(ref td) = args.topo_date { if !td.is_empty() { config.topo_date = td.clone(); } }
    if let Some(ref dn) = args.design_name { if !dn.is_empty() { config.design_name = dn.clone(); } }
    if let Some(ref lg) = args.logo { if !lg.is_empty() { config.company_logo_path = lg.clone(); } }
    if let Some(s) = args.step { config.grid_step = s; }
    if let Some(ref o) = args.outdir { if !o.is_empty() { config.default_outdir = o.clone(); } }

    let topo_path = if !config.topo_path.is_empty() && Path::new(&config.topo_path).exists() {
        config.topo_path.clone()
    } else {
        let def_topo = if Path::new(&config.topo_path).exists() { &config.topo_path } else { "" };
        prompt_file_path("[1/3] Enter Topo DXF file path", true, def_topo)
    };

    let design_path = if !config.design_path.is_empty() && Path::new(&config.design_path).exists() {
        config.design_path.clone()
    } else {
        let def_des = if Path::new(&config.design_path).exists() { &config.design_path } else { "" };
        prompt_file_path("[2/3] Enter Design DXF file path", true, def_des)
    };

    let boundary_path = match args.boundary {
        Some(path) if !path.is_empty() => path,
        _ => prompt_file_path("[3/3] Enter Boundary DXF path (opt)", false, ""),
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
    let volume = calculate_volume_with_tolerance(
        &grid,
        config.grid_step,
        config.ongrade_min,
        config.ongrade_max,
    );

    println!("Generating Marching Squares contour isolines...");
    let isolines = generate_isolines(&grid, config.grid_step, config.contour_levels.clone());
    let dxf_vector = export_isolines_to_dxf(&isolines);

    println!("\n----------------------------------------------------------------------");
    println!(
        "Volume Summary (Ongrade [{:+.2}m, {:+.2}m]): Cut = {:.2} m³ | Fill = {:.2} m³ | Ongrade Area = {:.2} m²",
        config.ongrade_min, config.ongrade_max, volume.cut_m3, volume.fill_m3, volume.ongrade_area_m2
    );

    // 1. Company Name: CLI > Prompt (default from config.dat)
    let company_name = if let Some(ref c) = args.company {
        c.clone()
    } else {
        prompt_input("Enter Company Name", true, &config.company_name)
    };

    // 2. Map Title / Project Title: CLI > Prompt (default from config.dat)
    let title = if let Some(ref t) = args.rainbow_title {
        t.clone()
    } else {
        prompt_input("Enter Map Title / Project Title", true, &config.default_title)
    };

    // 3. Drawn By: CLI > Prompt (default from config.dat)
    let drawn_by = if let Some(ref d) = args.drawn_by {
        d.clone()
    } else {
        prompt_input("Enter Drawn By", true, &config.drawn_by)
    };

    // 4. Topo Survey Date: CLI > Prompt (default from Topo DXF file modified date or config.dat)
    let default_topo_date = if !config.topo_date.is_empty() && config.topo_date != "28 July 2026" {
        config.topo_date.clone()
    } else {
        get_file_modified_date_string(&topo_path)
    };
    let topo_date = if let Some(ref td) = args.topo_date {
        td.clone()
    } else {
        prompt_input("Enter Topo Survey Date", true, &default_topo_date)
    };

    // 5. Design Name: CLI > Prompt (default from Design DXF filename or config.dat)
    let default_design_name = if !config.design_name.is_empty() {
        config.design_name.clone()
    } else {
        extract_design_name_default(&design_path)
    };
    let design_name = if let Some(ref dn) = args.design_name {
        dn.clone()
    } else {
        prompt_input("Enter Design Name", true, &default_design_name)
    };

    // 6. Date Created: Automatically today's date
    let date_created_str = get_current_date_string();

    // 7. Company Logo: CLI > Prompt (default from company_logo.png or config.dat)
    let default_logo_path = if !config.company_logo_path.is_empty() {
        config.company_logo_path.clone()
    } else {
        "company_logo.png".to_string()
    };
    let logo_path = if let Some(ref lg) = args.logo {
        lg.clone()
    } else {
        prompt_input("Enter Company Logo Path (press Enter to use default)", false, &default_logo_path)
    };

    // If user provided a different custom logo file, copy and save it as company_logo.png for subsequent runs
    let final_logo_path = if !logo_path.is_empty() && Path::new(&logo_path).exists() {
        let target_default = Path::new("company_logo.png");
        if Path::new(&logo_path) != target_default {
            if let Ok(_) = fs::copy(&logo_path, target_default) {
                println!("  [Saved] Copied '{}' to 'company_logo.png' for next runs.", logo_path);
            }
        }
        logo_path
    } else if Path::new(&default_logo_path).exists() {
        default_logo_path
    } else {
        String::new()
    };

    let logo_data_uri = load_image_as_data_uri(&final_logo_path);

    let kop_info = KopInfo {
        title: &title,
        company: &company_name,
        drawn_by: &drawn_by,
        date_created: &date_created_str,
        topo_date: &topo_date,
        design_name: &design_name,
        logo_data_uri: logo_data_uri.as_deref(),
    };

    let out_dir = Path::new(&config.default_outdir);
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    let html_path = out_dir.join("rainbow-viewer.html");
    let html_content = generate_html_viewer_with_config(&kop_info, &grid, &volume, &design_lines, &config);
    fs::write(&html_path, html_content)
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

    // Automatically open in default web browser unless disabled
    if config.auto_open_browser && !args.no_open {
        println!("Opening Rainbow Map Viewer in your default web browser...");
        open_file_in_default_browser(&html_path);
    }
}
