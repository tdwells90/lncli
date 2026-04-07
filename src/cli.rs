use clap::{Args, Parser, Subcommand, ValueEnum};

/// An opinionated CLI client for Linear.app.
#[derive(Parser)]
#[command(
    name = "lncli",
    version,
    about = "An opinionated CLI client for Linear.app.\n\nUse --fields to restrict which fields are returned in output (e.g., --fields id,title,description).\nNested fields use dot notation (e.g., --fields user.name,assignee.email).\nMandatory fields like 'id' are always included."
)]
pub struct Cli {
    /// Linear API token (overrides env var and token file)
    #[arg(long = "api-token", global = true)]
    pub api_token: Option<String>,

    /// Output format (toon or json)
    #[arg(long, global = true, value_enum, default_value = "toon")]
    pub format: OutputFormat,

    /// Fields to include in output (comma-separated, e.g. --fields id,title,description)
    /// Use dot notation for nested fields (e.g. --fields user.name,assignee.email)
    /// Mandatory fields (id, identifier) are always included
    #[arg(long, global = true)]
    pub fields: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Toon,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show usage info for all subcommands
    Usage,

    /// Issue operations
    Issues(IssuesArgs),

    /// Comment operations
    Comments(CommentsArgs),

    /// Document operations
    Documents(DocumentsArgs),

    /// Embed file operations (upload/download)
    Embeds(EmbedsArgs),

    /// Label operations
    Labels(LabelsArgs),

    /// Team operations
    Teams(TeamsArgs),

    /// User operations
    Users(UsersArgs),

    /// Project operations
    Projects(ProjectsArgs),

    /// Cycle (sprint) operations
    Cycles(CyclesArgs),

    /// Project milestone operations
    #[command(name = "project-milestones")]
    ProjectMilestones(ProjectMilestonesArgs),
}

// ── Issues ──────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct IssuesArgs {
    #[command(subcommand)]
    pub command: IssuesCommand,
}

#[derive(Subcommand)]
pub enum IssuesCommand {
    /// List issues with all relationships
    List {
        /// Maximum number of issues to return
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Read a single issue by ID or identifier (e.g. ABC-123)
    Read {
        /// Issue UUID or identifier like ABC-123
        issue_id: String,
    },

    /// Search issues with optional filters
    Search {
        /// Search query text
        query: String,

        /// Filter by team key, name, or ID
        #[arg(long)]
        team: Option<String>,

        /// Filter by assignee ID
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by project name or ID
        #[arg(long)]
        project: Option<String>,

        /// Filter by status (comma-separated)
        #[arg(long)]
        status: Option<String>,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },

    /// Create a new issue
    Create {
        /// Issue title
        title: String,

        /// Issue description in markdown (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,

        /// Assign to user ID
        #[arg(short, long)]
        assignee: Option<String>,

        /// Priority level (1=urgent, 2=high, 3=medium, 4=low)
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=4))]
        priority: Option<u8>,

        /// Add to project (name or ID)
        #[arg(long)]
        project: Option<String>,

        /// Team key, name, or ID (required)
        #[arg(long)]
        team: String,

        /// Labels (comma-separated names or IDs)
        #[arg(long)]
        labels: Option<String>,

        /// Project milestone name or ID (requires --project)
        #[arg(long)]
        project_milestone: Option<String>,

        /// Cycle name or ID (requires --team)
        #[arg(long)]
        cycle: Option<String>,

        /// Status name or ID
        #[arg(long)]
        status: Option<String>,

        /// Parent issue ID or identifier
        #[arg(long)]
        parent_ticket: Option<String>,
    },

    /// Delete (trash) an issue
    Delete {
        /// Issue UUID or identifier like ABC-123
        issue_id: String,
    },

    /// Update an existing issue
    Update {
        /// Issue UUID or identifier like ABC-123
        issue_id: String,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New description in markdown (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,

        /// New status name or ID
        #[arg(short, long)]
        status: Option<String>,

        /// New priority (1=urgent, 2=high, 3=medium, 4=low)
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=4))]
        priority: Option<u8>,

        /// New assignee ID
        #[arg(short, long)]
        assignee: Option<String>,

        /// New project (name or ID)
        #[arg(long)]
        project: Option<String>,

        // ── Labels group ──
        /// Labels (comma-separated names or IDs)
        #[arg(long, conflicts_with = "clear_labels")]
        labels: Option<String>,

        /// How to apply labels: adding (merge) or overwriting (replace)
        #[arg(long, value_enum, default_value = "adding", requires = "labels")]
        label_by: LabelMode,

        /// Remove all labels from issue
        #[arg(long, conflicts_with_all = ["labels", "label_by"])]
        clear_labels: bool,

        // ── Parent ticket group ──
        /// Set parent issue ID or identifier
        #[arg(long, conflicts_with = "clear_parent_ticket")]
        parent_ticket: Option<String>,

        /// Clear existing parent relationship
        #[arg(long)]
        clear_parent_ticket: bool,

        // ── Project milestone group ──
        /// Set project milestone (name or ID)
        #[arg(long, conflicts_with = "clear_project_milestone")]
        project_milestone: Option<String>,

        /// Clear existing milestone assignment
        #[arg(long)]
        clear_project_milestone: bool,

        // ── Cycle group ──
        /// Set cycle (name or ID)
        #[arg(long, conflicts_with = "clear_cycle")]
        cycle: Option<String>,

        /// Clear existing cycle assignment
        #[arg(long)]
        clear_cycle: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum LabelMode {
    Adding,
    Overwriting,
}

// ── Comments ────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct CommentsArgs {
    #[command(subcommand)]
    pub command: CommentsCommand,
}

#[derive(Subcommand)]
pub enum CommentsCommand {
    /// Create a new comment on an issue
    Create {
        /// Issue UUID or identifier like ABC-123
        issue_id: String,

        /// Comment body in markdown (use - for stdin)
        #[arg(long)]
        body: String,
    },

    /// Update an existing comment
    Update {
        /// Comment UUID
        comment_id: String,

        /// New comment body in markdown (use - for stdin)
        #[arg(long)]
        body: String,
    },

    /// Delete a comment
    Delete {
        /// Comment UUID
        comment_id: String,
    },
}

// ── Documents ───────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct DocumentsArgs {
    #[command(subcommand)]
    pub command: DocumentsCommand,
}

#[derive(Subcommand)]
pub enum DocumentsCommand {
    /// Create a new document
    Create {
        /// Document title (required)
        #[arg(long)]
        title: String,

        /// Document content in markdown (use - for stdin)
        #[arg(long)]
        content: Option<String>,

        /// Project name or ID
        #[arg(long)]
        project: Option<String>,

        /// Team key or name
        #[arg(long)]
        team: Option<String>,

        /// Document icon
        #[arg(long)]
        icon: Option<String>,

        /// Icon color
        #[arg(long)]
        color: Option<String>,

        /// Also attach document to this issue (e.g. ABC-123)
        #[arg(long)]
        attach_to: Option<String>,
    },

    /// Update an existing document
    Update {
        /// Document UUID, slug ID, or Linear URL
        document_id: String,

        /// New document title
        #[arg(long)]
        title: Option<String>,

        /// New document content in markdown (use - for stdin)
        #[arg(long)]
        content: Option<String>,

        /// Move to different project (name or ID)
        #[arg(long)]
        project: Option<String>,

        /// Document icon
        #[arg(long)]
        icon: Option<String>,

        /// Icon color
        #[arg(long)]
        color: Option<String>,
    },

    /// Read a document
    Read {
        /// Document UUID, slug ID, or Linear URL
        document_id: String,
    },

    /// List documents
    List {
        /// Filter by project name or ID
        #[arg(long, conflicts_with = "issue")]
        project: Option<String>,

        /// Filter by issue (shows documents attached to the issue)
        #[arg(long, conflicts_with = "project")]
        issue: Option<String>,

        /// Maximum number of documents
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Delete (soft-delete) a document
    Delete {
        /// Document UUID, slug ID, or Linear URL
        document_id: String,
    },
}

// ── Embeds ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct EmbedsArgs {
    #[command(subcommand)]
    pub command: EmbedsCommand,
}

#[derive(Subcommand)]
pub enum EmbedsCommand {
    /// Upload a file to Linear cloud storage (max 20MB)
    Upload {
        /// Path to file to upload
        file: String,
    },

    /// Download a file from Linear cloud storage
    Download {
        /// Linear upload URL
        url: String,

        /// Destination file path
        #[arg(short, long)]
        output: Option<String>,

        /// Allow replacing existing files
        #[arg(long)]
        overwrite: bool,
    },
}

// ── Labels ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct LabelsArgs {
    #[command(subcommand)]
    pub command: LabelsCommand,
}

#[derive(Subcommand)]
pub enum LabelsCommand {
    /// List all available labels
    List {
        /// Filter by team key, name, or ID
        #[arg(long)]
        team: Option<String>,
    },

    /// Create a new label
    Create {
        /// Label name
        name: String,

        /// Label color (hex, e.g. #ff0000)
        #[arg(long)]
        color: Option<String>,

        /// Scope to a team (key, name, or ID)
        #[arg(long)]
        team: Option<String>,

        /// Parent label group name or ID
        #[arg(long)]
        parent: Option<String>,
    },

    /// Update an existing label
    Update {
        /// Label UUID
        label_id: String,

        /// New label name
        #[arg(short, long)]
        name: Option<String>,

        /// New label color (hex)
        #[arg(long)]
        color: Option<String>,
    },

    /// Delete a label
    Delete {
        /// Label UUID
        label_id: String,
    },
}

// ── Teams ────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct TeamsArgs {
    #[command(subcommand)]
    pub command: TeamsCommand,
}

#[derive(Subcommand)]
pub enum TeamsCommand {
    /// List all teams
    List,
}

// ── Users ────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct UsersArgs {
    #[command(subcommand)]
    pub command: UsersCommand,
}

#[derive(Subcommand)]
pub enum UsersCommand {
    /// List all users
    List {
        /// Get the currently authenticated user
        #[arg(long)]
        me: bool,
    },
}

// ── Projects ─────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ProjectsArgs {
    #[command(subcommand)]
    pub command: ProjectsCommand,
}

#[derive(Subcommand)]
pub enum ProjectsCommand {
    /// List non-archived projects
    List {
        /// Maximum number of projects
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Read a single project by ID or name
    Read {
        /// Project UUID or name
        project_id_or_name: String,
    },

    /// Create a new project
    Create {
        /// Project name
        name: String,

        /// Team keys, names, or IDs (comma-separated, required)
        #[arg(long)]
        teams: String,

        /// Project description (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,

        /// Project content in markdown (use - for stdin)
        #[arg(long)]
        content: Option<String>,

        /// Project lead user ID
        #[arg(long)]
        lead: Option<String>,

        /// Priority (0=none, 1=urgent, 2=high, 3=normal, 4=low)
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..=4))]
        priority: Option<u8>,

        /// Planned start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,

        /// Planned target date (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,

        /// Project icon
        #[arg(long)]
        icon: Option<String>,

        /// Project color
        #[arg(long)]
        color: Option<String>,
    },

    /// Update an existing project
    Update {
        /// Project UUID or name
        project_id_or_name: String,

        /// New project name
        #[arg(short, long)]
        name: Option<String>,

        /// New description (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,

        /// New content in markdown (use - for stdin)
        #[arg(long)]
        content: Option<String>,

        /// New project lead user ID
        #[arg(long)]
        lead: Option<String>,

        /// New priority (0=none, 1=urgent, 2=high, 3=normal, 4=low)
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..=4))]
        priority: Option<u8>,

        /// New start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,

        /// New target date (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,

        /// New icon
        #[arg(long)]
        icon: Option<String>,

        /// New color
        #[arg(long)]
        color: Option<String>,

        /// Team keys, names, or IDs (comma-separated, replaces existing)
        #[arg(long)]
        teams: Option<String>,
    },

    /// Delete (trash) a project
    Delete {
        /// Project UUID or name
        project_id_or_name: String,
    },
}

// ── Cycles ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct CyclesArgs {
    #[command(subcommand)]
    pub command: CyclesCommand,
}

#[derive(Subcommand)]
pub enum CyclesCommand {
    /// List cycles
    List {
        /// Team key, name, or ID
        #[arg(long)]
        team: Option<String>,

        /// Only show active cycles
        #[arg(long)]
        active: bool,

        /// Return active +/- n cycles (requires --team)
        #[arg(long, requires = "team")]
        around_active: Option<u32>,
    },

    /// Create a new cycle
    Create {
        /// Team key, name, or ID (required)
        #[arg(long)]
        team: String,

        /// Cycle name
        #[arg(short, long)]
        name: Option<String>,

        /// Start date (ISO 8601, e.g. 2025-01-01)
        #[arg(long)]
        starts_at: String,

        /// End date (ISO 8601, e.g. 2025-01-14)
        #[arg(long)]
        ends_at: String,

        /// Cycle description (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Update an existing cycle
    Update {
        /// Cycle UUID or name
        cycle_id_or_name: String,

        /// Team key, name, or ID (scopes name lookup)
        #[arg(long)]
        team: Option<String>,

        /// New cycle name
        #[arg(short, long)]
        name: Option<String>,

        /// New start date (ISO 8601)
        #[arg(long)]
        starts_at: Option<String>,

        /// New end date (ISO 8601)
        #[arg(long)]
        ends_at: Option<String>,

        /// New description (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Read a cycle with its issues
    Read {
        /// Cycle UUID or name
        cycle_id_or_name: String,

        /// Team key, name, or ID (scopes name lookup)
        #[arg(long)]
        team: Option<String>,

        /// How many issues to fetch
        #[arg(long, default_value = "50")]
        issues_first: u32,
    },
}

// ── Project Milestones ───────────────────────────────────────────────────────

#[derive(Args)]
pub struct ProjectMilestonesArgs {
    #[command(subcommand)]
    pub command: ProjectMilestonesCommand,
}

#[derive(Subcommand)]
pub enum ProjectMilestonesCommand {
    /// List milestones in a project
    List {
        /// Project name or ID (required)
        #[arg(long)]
        project: String,

        /// Maximum number of milestones
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Read a milestone with its issues
    Read {
        /// Milestone UUID or name
        milestone_id_or_name: String,

        /// Project name or ID (scopes name lookup)
        #[arg(long)]
        project: Option<String>,

        /// How many issues to fetch
        #[arg(long, default_value = "50")]
        issues_first: u32,
    },

    /// Create a milestone
    Create {
        /// Milestone name
        name: String,

        /// Project name or ID (required)
        #[arg(long)]
        project: String,

        /// Milestone description (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,

        /// Target date in ISO format (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,
    },

    /// Delete a milestone
    Delete {
        /// Milestone UUID or name
        milestone_id_or_name: String,

        /// Project name or ID (scopes name lookup)
        #[arg(long)]
        project: Option<String>,
    },

    /// Update a milestone
    Update {
        /// Milestone UUID or name
        milestone_id_or_name: String,

        /// Project name or ID (scopes name lookup)
        #[arg(long)]
        project: Option<String>,

        /// New milestone name
        #[arg(short, long)]
        name: Option<String>,

        /// New milestone description (use - for stdin)
        #[arg(short, long)]
        description: Option<String>,

        /// New target date (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,

        /// New sort order
        #[arg(long)]
        sort_order: Option<f64>,
    },
}
