mod api;
mod cache;
mod display;
mod models;

use anyhow::Result;
use clap::{Parser, ValueEnum};

use api::RhythiaClient;
use cache::{Cache, SortBy};
use display::{paginate, progress_bar};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SortArg {
    Plays,
    Date,
}

impl From<SortArg> for SortBy {
    fn from(s: SortArg) -> Self {
        match s {
            SortArg::Plays => SortBy::Plays,
            SortArg::Date => SortBy::Date,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "rhythia-rp-finder",
    about = "Liste les maps Rhythia dont le Max RP est dans une fourchette donnée",
    version
)]
struct Args {
    /// Borne basse du Max RP (inclusive)
    #[arg(long)]
    low: u64,

    /// Borne haute du Max RP (inclusive)
    #[arg(long)]
    high: u64,

    /// Critère de tri
    #[arg(long, value_enum, default_value = "plays")]
    sort: SortArg,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.low >= args.high {
        eprintln!("Erreur : --low doit être strictement inférieur à --high.");
        std::process::exit(1);
    }

    let client = RhythiaClient::new()?;

    println!("Connexion à rhythia.com…");
    let maps = client.fetch_all(|fetched, total| {
        progress_bar(fetched, total);
    })?;

    // Clear progress line
    println!();

    let cache = Cache::new(maps);
    println!(
        "{} maps ranked chargées au total.",
        cache.total()
    );

    let sort: SortBy = args.sort.into();
    let filtered = cache.filter_by_rp(args.low, args.high, sort);
    let count = filtered.len();

    paginate(&filtered, count, args.low, args.high);

    Ok(())
}
