use structopt::StructOpt;

/// FeatherFlow - A lightweight workflow engine
#[derive(StructOpt, Debug)]
#[structopt(name = "featherflow")]
pub enum FeatherFlowCli {
    /// Start a workflow
    #[structopt(name = "start")]
    Start {
        /// Path to workflow definition file
        #[structopt(short, long)]
        file: String,

        /// Optional workflow name
        #[structopt(short, long)]
        name: Option<String>,
    },

    /// List all workflows
    #[structopt(name = "list")]
    List {
        /// Filter by status (running, completed, failed)
        #[structopt(short, long)]
        status: Option<String>,
    },

    /// Get status of a specific workflow
    #[structopt(name = "status")]
    Status {
        /// Workflow ID
        #[structopt(short, long)]
        id: String,
    },

    /// Stop a running workflow
    #[structopt(name = "stop")]
    Stop {
        /// Workflow ID
        #[structopt(short, long)]
        id: String,

        /// Force stop without waiting for current tasks to complete
        #[structopt(short, long)]
        force: bool,
    },

    /// Configure FeatherFlow settings
    #[structopt(name = "config")]
    Config {
        /// Set a configuration key-value pair
        #[structopt(short, long)]
        set: Option<Vec<String>>,

        /// Get value of a configuration key
        #[structopt(short, long)]
        get: Option<String>,
    },
}

pub fn parse_cli() -> FeatherFlowCli {
    FeatherFlowCli::from_args()
}

// This can be used in your main.rs or bin file
pub fn run_cli() {
    let cli = parse_cli();

    match cli {
        FeatherFlowCli::Start { file, name } => {
            println!("Starting workflow from file: {}", file);
            if let Some(workflow_name) = name {
                println!("Workflow name: {}", workflow_name);
            }
            // Call actual implementation
        }
        FeatherFlowCli::List { status } => {
            println!("Listing workflows");
            if let Some(status_filter) = status {
                println!("Filtering by status: {}", status_filter);
            }
            // Call actual implementation
        }
        FeatherFlowCli::Status { id } => {
            println!("Getting status for workflow: {}", id);
            // Call actual implementation
        }
        FeatherFlowCli::Stop { id, force } => {
            println!("Stopping workflow: {}", id);
            if force {
                println!("Using force stop");
            }
            // Call actual implementation
        }
        FeatherFlowCli::Config { set, get } => {
            if let Some(kvs) = set {
                println!("Setting configuration: {:?}", kvs);
            }
            if let Some(key) = get {
                println!("Getting configuration for: {}", key);
            }
            // Call actual implementation
        }
    }
}
