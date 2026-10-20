use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

#[derive(Parser)]
#[command(
    name = "dustmap",
    about = "DustMap: blazing-fast disk space analyzer",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a directory and report disk usage
    Scan {
        path: PathBuf,
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(long, default_value = "logical")]
        metric: String,
    },
    /// Print a summary of disk usage for a path
    Summary { path: PathBuf },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            path,
            json,
            metric: _,
        } => {
            eprintln!("Scanning {}...", path.display());
            let cancel = Arc::new(AtomicBool::new(false));
            let (tx, rx) = mpsc::channel();

            let options = dustmap_core::scanner::ScanOptions::default();
            let cancel_clone = cancel.clone();
            let path_clone = path.clone();

            let handle = std::thread::spawn(move || {
                dustmap_core::scanner::scan(&path_clone, options, cancel_clone, tx)
            });

            // Drain progress updates
            while let Ok(progress) = rx.recv() {
                eprint!(
                    "\r  {} entries | {} | {:.1}s",
                    progress.entries_seen,
                    dustmap_core::metrics::format_size(progress.bytes_accounted),
                    progress.elapsed_secs
                );
            }

            let result = handle
                .join()
                .map_err(|_| anyhow::anyhow!("Scanner thread panicked"))??;
            let root_agg = result.index.aggregate(dustmap_core::index::ROOT_ID);

            if json {
                let summary = serde_json::json!({
                    "root": path.display().to_string(),
                    "total_logical": root_agg.map(|a| a.logical_total).unwrap_or(0),
                    "total_items": root_agg.map(|a| a.item_count).unwrap_or(0),
                    "errors": result.errors.len(),
                    "duration_secs": result.duration.as_secs_f64(),
                    "cancelled": result.cancelled,
                });
                println!("\n{}", serde_json::to_string_pretty(&summary)?);
            } else {
                let agg = root_agg.cloned().unwrap_or_default();
                println!(
                    "\n\nRoot: {}\nTotal size: {}\nItems: {}\nErrors: {}\nDuration: {:.1}s",
                    path.display(),
                    dustmap_core::metrics::format_size(agg.logical_total),
                    agg.item_count,
                    result.errors.len(),
                    result.duration.as_secs_f64(),
                );
            }
        }
        Commands::Summary { path } => {
            println!("Summary for: {}", path.display());
            let cancel = Arc::new(AtomicBool::new(false));
            let (tx, _rx) = mpsc::channel();
            let options = dustmap_core::scanner::ScanOptions::default();
            let result = dustmap_core::scanner::scan(&path, options, cancel, tx)?;
            let agg = result
                .index
                .aggregate(dustmap_core::index::ROOT_ID)
                .cloned()
                .unwrap_or_default();
            println!(
                "Total Size: {}",
                dustmap_core::metrics::format_size(agg.logical_total)
            );
            println!("Total Items: {}", agg.item_count);
        }
    }

    Ok(())
}
