use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "flowforge", about = "FlowForge Workflow Scheduler CLI", version)]
struct Cli {
    /// API server URL
    #[arg(long, default_value = "http://localhost:8080", env = "FLOWFORGE_API_URL")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a DAG from a YAML file
    Submit {
        /// Path to the YAML DAG file
        #[arg(short, long)]
        file: String,
    },
    /// Trigger a DAG run
    Trigger {
        /// DAG ID to trigger
        dag_id: String,
    },
    /// List all DAGs
    Dags,
    /// List recent runs
    Runs,
    /// Get run details and task statuses
    Run {
        /// Run ID (UUID)
        run_id: String,
    },
    /// Get system status
    Status,
    /// List active workers
    Workers,
    /// Get DAG details
    Dag {
        /// DAG ID
        dag_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let base = cli.api_url.trim_end_matches('/');

    match cli.command {
        Commands::Submit { file } => {
            let yaml = std::fs::read_to_string(&file)?;
            let body = serde_json::json!({ "yaml": yaml });
            let resp = client
                .post(format!("{base}/api/v1/dags"))
                .json(&body)
                .send()
                .await?;
            print_response(resp).await?;
        }
        Commands::Trigger { dag_id } => {
            let body = serde_json::json!({ "dag_id": dag_id, "triggered_by": "cli" });
            let resp = client
                .post(format!("{base}/api/v1/runs"))
                .json(&body)
                .send()
                .await?;
            print_response(resp).await?;
        }
        Commands::Dags => {
            let resp = client.get(format!("{base}/api/v1/dags")).send().await?;
            print_response(resp).await?;
        }
        Commands::Runs => {
            let resp = client.get(format!("{base}/api/v1/runs")).send().await?;
            print_response(resp).await?;
        }
        Commands::Run { run_id } => {
            // Get run info
            let resp = client.get(format!("{base}/api/v1/runs/{run_id}")).send().await?;
            let body: Value = resp.json().await?;
            println!("=== Run ===");
            println!("{}", serde_json::to_string_pretty(&body)?);

            // Get tasks
            let resp = client.get(format!("{base}/api/v1/runs/{run_id}/tasks")).send().await?;
            let body: Value = resp.json().await?;
            println!("\n=== Tasks ===");
            if let Some(tasks) = body.get("data").and_then(|d| d.as_array()) {
                println!("{:<20} {:<12} {:<8} {:<20}", "TASK ID", "STATUS", "ATTEMPT", "WORKER");
                println!("{}", "-".repeat(60));
                for t in tasks {
                    println!(
                        "{:<20} {:<12} {:<8} {:<20}",
                        t.get("task_id").and_then(|v| v.as_str()).unwrap_or("-"),
                        t.get("status").and_then(|v| v.as_str()).unwrap_or("-"),
                        t.get("attempt").and_then(|v| v.as_i64()).unwrap_or(0),
                        t.get("worker_id").and_then(|v| v.as_str()).unwrap_or("-"),
                    );
                }
            }
        }
        Commands::Status => {
            let resp = client.get(format!("{base}/api/v1/status")).send().await?;
            print_response(resp).await?;
        }
        Commands::Workers => {
            let resp = client.get(format!("{base}/api/v1/workers")).send().await?;
            print_response(resp).await?;
        }
        Commands::Dag { dag_id } => {
            let resp = client.get(format!("{base}/api/v1/dags/{dag_id}")).send().await?;
            print_response(resp).await?;
        }
    }

    Ok(())
}

async fn print_response(resp: reqwest::Response) -> Result<()> {
    let status = resp.status();
    let body: Value = resp.json().await?;
    if status.is_success() {
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        eprintln!("Error ({}): {}", status, serde_json::to_string_pretty(&body)?);
        std::process::exit(1);
    }
    Ok(())
}
