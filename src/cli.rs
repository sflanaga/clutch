use anyhow::{anyhow, Context, Result};
use structopt::StructOpt;

pub const TU32: u32 = 1;
pub const TF64: u32 = 2;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
rename_all = "kebab-case",
global_settings(& [
structopt::clap::AppSettings::ColoredHelp,
structopt::clap::AppSettings::UnifiedHelpMessage
]),
)]
pub struct Cli {
    #[structopt(default_value("2"))]
    /// Number of times to run test
    pub iterations: u32,

    #[structopt(default_value("2"))]
    /// Passes to make to same set of clutches adding more OMs
    pub passes: u32,

    #[structopt(default_value("2"))]
    /// Key1 in address
    pub k1: u32,

    #[structopt(default_value("2"))]
    /// Key2 in address
    pub k2: u32,

    #[structopt(default_value("2"))]
    /// Key3 in address
    pub k3: u32,

    #[structopt(default_value("4"))]
    /// OMs per record per keys and per passes
    pub oms: u32,

    #[structopt(default_value("4"), parse(try_from_str = parse_types_list), default_value("u32,f64"))]
    /// OMs per record per keys and per passes per type u32/f64
    ///
    /// --om-types u32,f64
    pub om_types: u32,

    #[structopt(long="dump-full")]
    /// write the full clutch set out - otherwise just first and last
    pub dump_full: bool,

    #[structopt(short="p")]
    /// pause for user input (ENTER) before going to next iteration
    pub pause: bool,

    #[structopt(short = "v", parse(from_occurrences))]
    /// Verbosity - use more than one v for greater detail
    pub verbose: usize,

    #[structopt(short = "n", default_value("0"))]
    /// every N OM (k3 mod N) will be null, 0 = never
    pub random_nulls: u32,

}

fn parse_types_list(str: &str) -> Result<u32> {
    let mut types = 0u32;
    for t in str.split(',') {
        match t {
            "u32" => types |= TU32,
            "f64" => types |= TF64,
            _ => Err(anyhow!("type {} not understood in {}", &t, &str))?
        }
    }
    Ok(types)
}
