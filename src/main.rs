mod cli;
mod commands;
mod graphql;
mod models;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use utils::{auth, output};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    output::set_format(cli.format);

    if let Err(e) = run(cli).await {
        output::print_error(&e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), utils::error::CliError> {
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            print_command_groups();
            return Ok(());
        }
    };

    match command {
        Commands::Usage => {
            commands::usage::execute();
            Ok(())
        }
        _ => {
            let token = auth::get_api_token(cli.api_token.as_deref())?;
            let client = graphql::client::GraphqlClient::new(&token);

            match command {
                Commands::Usage => unreachable!(),
                Commands::Teams(args) => commands::teams::execute(&client, args).await,
                Commands::Users(args) => commands::users::execute(&client, args).await,
                Commands::Labels(args) => commands::labels::execute(&client, args).await,
                Commands::Projects(args) => commands::projects::execute(&client, args).await,
                Commands::Issues(args) => commands::issues::execute(&client, args).await,
                Commands::Comments(args) => commands::comments::execute(&client, args).await,
                Commands::Cycles(args) => commands::cycles::execute(&client, args).await,
                Commands::ProjectMilestones(args) => {
                    commands::project_milestones::execute(&client, args).await
                }
                Commands::Documents(args) => commands::documents::execute(&client, args).await,
                Commands::Embeds(args) => commands::embeds::execute(&client, args).await,
            }
        }
    }
}

fn print_command_groups() {
    let groups = serde_json::json!({
        "commands": [
            {"name": "usage", "description": "Show usage info for all subcommands"},
            {"name": "issues", "description": "Issue operations (list, read, search, create, update, delete)"},
            {"name": "comments", "description": "Comment operations (create, update, delete)"},
            {"name": "documents", "description": "Document operations (create, update, read, list, delete)"},
            {"name": "embeds", "description": "Embed file operations (upload, download)"},
            {"name": "labels", "description": "Label operations (list, create, update, delete)"},
            {"name": "teams", "description": "Team operations (list)"},
            {"name": "users", "description": "User operations (list)"},
            {"name": "projects", "description": "Project operations (list, read, create, update, delete)"},
            {"name": "cycles", "description": "Cycle operations (list, read, create, update)"},
            {"name": "project-milestones", "description": "Project milestone operations (list, read, create, update, delete)"}
        ]
    });
    output::print_json(&groups);
}
