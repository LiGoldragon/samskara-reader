use std::sync::Arc;

use clap::Parser;
use std::path::PathBuf;

mod mcp;

/// Samskara Reader — read-only MCP server for querying samskara's world state.
/// Opens the same world.db as samskara but in immutable mode.
#[derive(Parser)]
#[command(name = "samskara-reader", about = "Read-only MCP server for samskara world state")]
struct Args {
    /// Path to the sqlite-backed CozoDB database (samskara's world.db).
    #[arg(long, value_name = "DB_PATH")]
    db_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let args = Args::parse();

    tracing::info!("opening sqlite db (read-only) at {}", args.db_path.display());
    let db = criome_cozo::CriomeDb::open_sqlite(&args.db_path)?;

    // Verify the database is initialized
    if !samskara_core::boot::is_initialized(&db) {
        return Err("database is not initialized — run samskara first to create world.db".into());
    }

    let db = Arc::new(db);
    let server = mcp::SamskaraReader::new(db);

    tracing::info!("samskara-reader MCP server starting on stdio");
    let service = rmcp::ServiceExt::serve(server, rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
