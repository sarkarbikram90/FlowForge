use clap::{Parser, Subcommand};
use colored::*;
use reqwest::Client;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "flowforge")]
#[command(about = "FlowForge - Cloud-Native Workload Orchestration Platform CLI", version = "0.2.0")]
struct Cli {
    #[arg(short, long, env = "FLOWFORGE_API_URL", default_value = "http://localhost:8080")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication and profile commands
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Workflow management (validate, apply, list, get)
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },
    /// Workflow run operations (trigger, list, get, cancel)
    Run {
        #[command(subcommand)]
        command: RunCommands,
    },
    /// Worker management (list, drain)
    Worker {
        #[command(subcommand)]
        command: WorkerCommands,
    },
    /// Queue and Dead Letter Queue management
    Queue {
        #[command(subcommand)]
        command: QueueCommands,
    },
    /// Cluster status and health summary
    Status,
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Authenticate with FlowForge
    Login {
        #[arg(short, long, default_value = "admin@flowforge.internal")]
        email: String,
    },
    /// Show current authenticated user context
    Whoami,
    /// Generate an API key
    ApiKey,
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// Validate workflow YAML file locally and against the server
    Validate {
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Apply/upload workflow definition YAML
    Apply {
        #[arg(short, long)]
        file: PathBuf,
    },
    /// List all workflows
    List,
    /// Get workflow details by ID
    Get { id: String },
}

#[derive(Subcommand)]
enum RunCommands {
    /// Trigger a workflow run
    Trigger {
        name: String,
        #[arg(short, long)]
        variables: Option<String>,
    },
    /// List recent workflow runs
    List,
    /// Get run details and task statuses
    Get { id: String },
    /// Cancel an active workflow run
    Cancel { id: String },
}

#[derive(Subcommand)]
enum WorkerCommands {
    /// List all registered workers
    List,
    /// Drain a worker before termination
    Drain { id: String },
}

#[derive(Subcommand)]
enum QueueCommands {
    /// List DLQ tasks
    Dlq,
    /// Resolve a DLQ item
    Resolve { id: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = Client::new();

    match cli.command {
        Commands::Status => {
            let res = client
                .get(format!("{}/api/v1/stats", cli.api_url))
                .send()
                .await?
                .json::<Value>()
                .await?;

            if let Some(data) = res.get("data") {
                println!("{}", "=== FlowForge Cluster Status ===".bold().cyan());
                println!("Active Workflows : {}", data["active_workflows"]);
                println!("Total Runs       : {}", data["total_runs"]);
                println!("Running Runs     : {}", data["running_runs"].to_string().yellow());
                println!("Succeeded Runs   : {}", data["succeeded_runs"].to_string().green());
                println!("Failed Runs      : {}", data["failed_runs"].to_string().red());
                println!("Active Workers   : {}", data["active_workers"].to_string().blue());
                println!("DLQ Count        : {}", data["dlq_count"].to_string().magenta());
                println!("Scheduler Leader : {}", data["scheduler_leader_id"]);
                println!("Scheduler Healthy: {}", data["scheduler_healthy"].to_string().green());
            } else {
                println!("{}", "Failed to fetch status".red());
            }
        }
        Commands::Auth { command } => match command {
            AuthCommands::Login { email } => {
                let res = client
                    .post(format!("{}/api/v1/auth/login", cli.api_url))
                    .json(&serde_json::json!({ "email": email }))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{} Authenticated as {}", "✔".green(), email.bold());
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
            AuthCommands::Whoami => {
                let res = client
                    .get(format!("{}/api/v1/auth/whoami", cli.api_url))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
            AuthCommands::ApiKey => {
                let res = client
                    .post(format!("{}/api/v1/auth/keys", cli.api_url))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{} Generated API Key:", "✔".green());
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
        },
        Commands::Workflow { command } => match command {
            WorkflowCommands::Validate { file } => {
                let content = std::fs::read_to_string(&file)?;
                let res = client
                    .post(format!("{}/api/v1/workflows/validate", cli.api_url))
                    .json(&serde_json::json!({ "yaml": content }))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                if res["success"].as_bool().unwrap_or(false) {
                    println!("{} Workflow definition is valid!", "✔".green());
                    println!("{}", serde_json::to_string_pretty(&res["data"])?);
                } else {
                    println!("{} Validation error: {}", "✖".red(), res["error"]["message"]);
                }
            }
            WorkflowCommands::Apply { file } => {
                let content = std::fs::read_to_string(&file)?;
                let res = client
                    .post(format!("{}/api/v1/workflows", cli.api_url))
                    .json(&serde_json::json!({ "yaml": content }))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                if res["success"].as_bool().unwrap_or(false) {
                    println!("{} Workflow applied successfully!", "✔".green());
                    println!("ID: {}", res["data"]["workflow"]["id"]);
                    println!("Name: {}", res["data"]["workflow"]["name"]);
                } else {
                    println!("{} Failed to apply: {}", "✖".red(), res["error"]["message"]);
                }
            }
            WorkflowCommands::List => {
                let res = client
                    .get(format!("{}/api/v1/workflows", cli.api_url))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
            WorkflowCommands::Get { id } => {
                let res = client
                    .get(format!("{}/api/v1/workflows/{}", cli.api_url, id))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
        },
        Commands::Run { command } => match command {
            RunCommands::Trigger { name, variables } => {
                let vars: Value = variables
                    .and_then(|v| serde_json::from_str(&v).ok())
                    .unwrap_or(serde_json::json!({}));
                let res = client
                    .post(format!("{}/api/v1/workflow-runs", cli.api_url))
                    .json(&serde_json::json!({
                        "workflow_name": name,
                        "variables": vars
                    }))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                if res["success"].as_bool().unwrap_or(false) {
                    println!("{} Workflow run triggered!", "✔".green());
                    println!("Run ID: {}", res["data"]["id"]);
                    println!("Status: {}", res["data"]["status"].to_string().yellow());
                } else {
                    println!("{} Failed: {}", "✖".red(), res["error"]["message"]);
                }
            }
            RunCommands::List => {
                let res = client
                    .get(format!("{}/api/v1/workflow-runs", cli.api_url))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
            RunCommands::Get { id } => {
                let res = client
                    .get(format!("{}/api/v1/workflow-runs/{}", cli.api_url, id))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
            RunCommands::Cancel { id } => {
                let _res = client
                    .post(format!("{}/api/v1/workflow-runs/{}/cancel", cli.api_url, id))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{} Workflow run canceled", "✔".green());
            }
        },
        Commands::Worker { command } => match command {
            WorkerCommands::List => {
                let res = client
                    .get(format!("{}/api/v1/workers", cli.api_url))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
            WorkerCommands::Drain { id } => {
                let _res = client
                    .post(format!("{}/api/v1/workers/{}/drain", cli.api_url, id))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{} Worker {} set to DRAINING", "✔".green(), id);
            }
        },
        Commands::Queue { command } => match command {
            QueueCommands::Dlq => {
                let res = client
                    .get(format!("{}/api/v1/dlq", cli.api_url))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res["data"])?);
            }
            QueueCommands::Resolve { id } => {
                let _res = client
                    .post(format!("{}/api/v1/dlq/{}/resolve", cli.api_url, id))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{} DLQ item resolved", "✔".green());
            }
        },
    }

    Ok(())
}
