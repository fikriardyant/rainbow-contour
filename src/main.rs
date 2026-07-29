mod cli;
use clap::Parser;

fn main() {
    let args = cli::CliArgs::parse();
    println!("Rainbow Contour Cut & Fill Engine v0.1.0");
    println!("Topo: {:?}", args.topo);
    println!("Design: {:?}", args.design);
    println!("Boundary: {:?}", args.boundary);
    println!("Grid Step: {}m", args.step);
}
