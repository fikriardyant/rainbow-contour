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

    #[arg(short, long, help = "Grid resolution step in meters (overrides config.dat)")]
    pub step: Option<f64>,

    #[arg(short, long, help = "Output directory (overrides config.dat)")]
    pub outdir: Option<String>,

    #[arg(long, help = "Company Name for Kop (overrides config.dat)")]
    pub company: Option<String>,

    #[arg(long, help = "Rainbow Title / Project Name for Kop (overrides config.dat)")]
    pub rainbow_title: Option<String>,

    #[arg(long, help = "Drawn By name for Kop (overrides config.dat)")]
    pub drawn_by: Option<String>,

    #[arg(long, help = "Topo Survey Date for Kop")]
    pub topo_date: Option<String>,

    #[arg(long, help = "Design Name for Kop")]
    pub design_name: Option<String>,

    #[arg(long, help = "Path to Company Logo PNG/JPG (default: company_logo.png)")]
    pub logo: Option<String>,

    #[arg(long, help = "Disable automatically opening HTML viewer in default browser")]
    pub no_open: bool,

    #[arg(long, default_value = "config.dat", help = "Path to config.dat file")]
    pub config: String,
}
