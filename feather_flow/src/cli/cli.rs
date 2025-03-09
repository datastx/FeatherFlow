use structopt::StructOpt;

/// FeatherFlow - A lightweight workflow engine
#[derive(StructOpt, Debug)]
#[structopt(name = "featherflow")]
pub enum FeatherFlowCli {
    /// Generate or display the workflow DAG
    #[structopt(name = "dag")]
    Dag {
        #[structopt(short, long)]
        target: String,
    },

    /// Show details about a workflow or task
    #[structopt(name = "show")]
    Show {
        #[structopt(short, long)]
        target: String,
    },

    /// Compile a workflow definition
    #[structopt(name = "compile")]
    Compile {
        #[structopt(short, long)]
        target: String,
    },
}

pub fn parse_cli() -> FeatherFlowCli {
    FeatherFlowCli::from_args()
}

pub fn run_cli() {
    let cli = parse_cli();

    match cli {
        FeatherFlowCli::Dag { target } => {
            println!("Generating DAG for target: {}", target);
        }
        FeatherFlowCli::Show { target } => {
            println!("Showing details for target: {}", target);
        }
        FeatherFlowCli::Compile { target } => {
            println!("Compiling workflow for target: {}", target);
        }
    }
}
