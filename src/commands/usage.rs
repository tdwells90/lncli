/// Print usage info for all subcommands, optimized for LLM consumption.
pub fn execute() {
    let usage = serde_json::json!({
        "name": "linear-cli",
        "description": "CLI client for Linear.app. All output is JSON.",
        "authentication": {
            "methods": [
                {"source": "--api-token <token>", "priority": "highest"},
                {"source": "LINEAR_API_TOKEN env var", "priority": "medium"},
                {"source": "~/.linear_api_token file", "priority": "lowest"}
            ]
        },
        "id_resolution": "All ID arguments accept: UUIDs (pass-through), team-prefixed identifiers (ABC-123), or human-readable names for teams, projects, labels, cycles, and milestones.",
        "commands": {
            "issues": [
                {"command": "issues list", "flags": "[-l, --limit <n=25>]", "description": "List issues with all relationships."},
                {"command": "issues read <issueId>", "description": "Read a single issue. issueId: UUID or identifier (ABC-123)."},
                {"command": "issues search <query>", "flags": "[--team <team>] [--assignee <id>] [--project <project>] [--status <statuses>] [-l, --limit <n=10>]", "description": "Search issues. --status accepts comma-separated values."},
                {"command": "issues create <title>", "flags": "--team <team> [-d, --description <desc>] [-a, --assignee <id>] [-p, --priority <1-4>] [--project <project>] [--labels <labels>] [--project-milestone <milestone>] [--cycle <cycle>] [--status <status>] [--parent-ticket <id>]", "description": "Create an issue. --team is required. --labels accepts comma-separated names or IDs."},
                {"command": "issues update <issueId>", "flags": "[-t, --title <title>] [-d, --description <desc>] [-s, --status <status>] [-p, --priority <1-4>] [-a, --assignee <id>] [--project <project>] [--labels <labels>] [--label-by <adding|overwriting>] [--clear-labels] [--parent-ticket <id>] [--clear-parent-ticket] [--project-milestone <milestone>] [--clear-project-milestone] [--cycle <cycle>] [--clear-cycle]", "description": "Update an issue. Mutually exclusive: --labels/--clear-labels, --parent-ticket/--clear-parent-ticket, --project-milestone/--clear-project-milestone, --cycle/--clear-cycle."},
                {"command": "issues delete <issueId>", "description": "Delete (trash) an issue."}
            ],
            "comments": [
                {"command": "comments create <issueId>", "flags": "--body <body>", "description": "Create a comment on an issue. Body is markdown."},
                {"command": "comments update <commentId>", "flags": "--body <body>", "description": "Update an existing comment."},
                {"command": "comments delete <commentId>", "description": "Delete a comment."}
            ],
            "documents": [
                {"command": "documents create", "flags": "--title <title> [--content <md>] [--project <project>] [--team <team>] [--icon <icon>] [--color <color>] [--attach-to <issueId>]", "description": "Create a document. --attach-to also creates an attachment on the issue."},
                {"command": "documents update <documentId>", "flags": "[--title <title>] [--content <md>] [--project <project>] [--icon <icon>] [--color <color>]", "description": "Update a document. documentId: UUID, slug ID, or Linear URL."},
                {"command": "documents read <documentId>", "description": "Read a document."},
                {"command": "documents list", "flags": "[--project <project>] [--issue <issueId>] [-l, --limit <n=50>]", "description": "List documents. --project and --issue are mutually exclusive."},
                {"command": "documents delete <documentId>", "description": "Soft-delete a document (moves to trash)."}
            ],
            "embeds": [
                {"command": "embeds upload <file>", "description": "Upload a file (max 20MB). Returns asset URL."},
                {"command": "embeds download <url>", "flags": "[-o, --output <path>] [--overwrite]", "description": "Download a file from Linear storage."}
            ],
            "labels": [
                {"command": "labels list", "flags": "[--team <team>]", "description": "List labels, excluding group containers."},
                {"command": "labels create <name>", "flags": "[--color <hex>] [--team <team>] [--parent <label>]", "description": "Create a label. --team scopes to a team. --parent sets the group."},
                {"command": "labels update <labelId>", "flags": "[-n, --name <name>] [--color <hex>]", "description": "Update a label."},
                {"command": "labels delete <labelId>", "description": "Delete a label."}
            ],
            "teams": [
                {"command": "teams list", "description": "List all teams."}
            ],
            "users": [
                {"command": "users list", "flags": "[--me]", "description": "List all users. Use --me to get the currently authenticated user."}
            ],
            "projects": [
                {"command": "projects list", "flags": "[-l, --limit <n>]", "description": "List non-archived projects."},
                {"command": "projects read <projectIdOrName>", "description": "Read a single project by ID or name."},
                {"command": "projects create <name>", "flags": "--teams <teams> [-d, --description <desc>] [--content <md>] [--lead <userId>] [-p, --priority <0-4>] [--start-date <YYYY-MM-DD>] [--target-date <YYYY-MM-DD>] [--icon <icon>] [--color <color>]", "description": "Create a project. --teams accepts comma-separated team keys, names, or IDs."},
                {"command": "projects update <projectIdOrName>", "flags": "[-n, --name <name>] [-d, --description <desc>] [--content <md>] [--lead <userId>] [-p, --priority <0-4>] [--start-date <YYYY-MM-DD>] [--target-date <YYYY-MM-DD>] [--icon <icon>] [--color <color>] [--teams <teams>]", "description": "Update a project."},
                {"command": "projects delete <projectIdOrName>", "description": "Delete (trash) a project."}
            ],
            "cycles": [
                {"command": "cycles list", "flags": "[--team <team>] [--active] [--around-active <n>]", "description": "List cycles. --around-active returns active +/- n cycles (requires --team)."},
                {"command": "cycles create", "flags": "--team <team> --starts-at <ISO8601> --ends-at <ISO8601> [-n, --name <name>] [-d, --description <desc>]", "description": "Create a cycle. --team, --starts-at, and --ends-at are required."},
                {"command": "cycles update <cycleIdOrName>", "flags": "[--team <team>] [-n, --name <name>] [--starts-at <ISO8601>] [--ends-at <ISO8601>] [-d, --description <desc>]", "description": "Update a cycle."},
                {"command": "cycles read <cycleIdOrName>", "flags": "[--team <team>] [--issues-first <n=50>]", "description": "Read a cycle with its issues."}
            ],
            "project-milestones": [
                {"command": "project-milestones list", "flags": "--project <project> [-l, --limit <n=50>]", "description": "List milestones in a project."},
                {"command": "project-milestones read <milestoneIdOrName>", "flags": "[--project <project>] [--issues-first <n=50>]", "description": "Read a milestone with its issues."},
                {"command": "project-milestones create <name>", "flags": "--project <project> [-d, --description <desc>] [--target-date <YYYY-MM-DD>]", "description": "Create a milestone."},
                {"command": "project-milestones update <milestoneIdOrName>", "flags": "[--project <project>] [-n, --name <name>] [-d, --description <desc>] [--target-date <YYYY-MM-DD>] [--sort-order <n>]", "description": "Update a milestone."},
                {"command": "project-milestones delete <milestoneIdOrName>", "flags": "[--project <project>]", "description": "Delete a milestone."}
            ]
        }
    });

    crate::utils::output::print_json(&usage);
}
