use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rainbow-contour", author = "Fikri Ardyantoro / Kuda", version = "0.1.0", about = "Rainbow Contour Cut & Fill Engine")]
pub struct CliArgs {
    #[arg(short, long, help = "Path to Topo DXF file")]
    pub topo: Option<String>,

    #[arg(short, long, help = "Path to Design DXF file")]
    pub design: Option<String>,

    #[arg(short, long, help = "Path to Boundary DXF file")]
    pub boundary: Option<String>,

    #[arg(short, long, default_value_t = 1.0, help = "Grid resolution step in meters")]
    pub step: f64,

    #[arg(short, long, default_value = "./output", help = "Output directory")]
    pub outdir: String,
}
