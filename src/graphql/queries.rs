// ── Fragments ────────────────────────────────────────────────────────────────

pub const ISSUE_CORE_FIELDS: &str = "
    id
    identifier
    title
    description
    branchName
    priority
";

pub const ISSUE_RELATIONS: &str = "
    state { id name type }
    assignee { id name }
    team { id key }
    project { id name }
    cycle { id name number }
    projectMilestone { id name }
    labels { nodes { id name } }
    parent { id identifier title }
    children { nodes { id identifier title } }
";

pub const ISSUE_COMMENTS_FRAGMENT: &str = "
    comments {
        nodes {
            id
            body
            user { name }
        }
    }
";

// ── Teams ────────────────────────────────────────────────────────────────────

pub const TEAMS_LIST: &str = "
    query TeamsList {
        teams(orderBy: updatedAt) {
            nodes {
                id
                key
                name
                description
            }
        }
    }
";

// ── Users ────────────────────────────────────────────────────────────────────

pub const USERS_LIST: &str = "
    query UsersList {
        users {
            nodes {
                id
                name
                displayName
                email
                active
            }
        }
    }
";

pub const VIEWER: &str = "
    query Viewer {
        viewer {
            id
            name
            displayName
            email
            active
        }
    }
";

// ── Labels ───────────────────────────────────────────────────────────────────

pub const LABELS_LIST: &str = "
    query LabelsList {
        issueLabels(orderBy: updatedAt) {
            nodes {
                id
                name
                color
                isGroup
                parent { id name }
                team { id key name }
            }
        }
    }
";

pub const LABELS_LIST_BY_TEAM: &str = "
    query LabelsListByTeam($teamId: ID!) {
        issueLabels(
            orderBy: updatedAt,
            filter: { team: { id: { eq: $teamId } } }
        ) {
            nodes {
                id
                name
                color
                isGroup
                parent { id name }
                team { id key name }
            }
        }
    }
";

// ── Projects ─────────────────────────────────────────────────────────────────

pub const PROJECTS_LIST: &str = "
    query ProjectsList($first: Int) {
        projects(
            first: $first,
            orderBy: updatedAt
        ) {
            nodes {
                id
                name
                state
                progress
                startDate
                targetDate
                teams { nodes { key } }
                lead { name }
            }
        }
    }
";

pub const PROJECT_READ: &str = "
    query ProjectRead($id: String!) {
        project(id: $id) {
            id
            name
            description
            content
            state
            priority
            progress
            startDate
            targetDate
            teams { nodes { key name } }
            lead { name }
        }
    }
";

pub const PROJECT_CREATE: &str = "
    mutation ProjectCreate($input: ProjectCreateInput!) {
        projectCreate(input: $input) {
            success
            project {
                id
                name
                state
            }
        }
    }
";

pub const PROJECT_UPDATE: &str = "
    mutation ProjectUpdate($id: String!, $input: ProjectUpdateInput!) {
        projectUpdate(id: $id, input: $input) {
            success
            project {
                id
                name
                state
            }
        }
    }
";

pub const PROJECT_DELETE: &str = "
    mutation ProjectDelete($id: String!) {
        projectDelete(id: $id) {
            success
        }
    }
";

// ── Issues ───────────────────────────────────────────────────────────────────

pub fn issues_list_query() -> String {
    format!(
        "query IssuesList($first: Int!) {{
            issues(first: $first, orderBy: updatedAt) {{
                nodes {{
                    {ISSUE_CORE_FIELDS}
                    {ISSUE_RELATIONS}
                }}
            }}
        }}"
    )
}

pub fn issue_read_by_id_query() -> String {
    format!(
        "query IssueById($id: String!) {{
            issue(id: $id) {{
                {ISSUE_CORE_FIELDS}
                {ISSUE_RELATIONS}
                {ISSUE_COMMENTS_FRAGMENT}
            }}
        }}"
    )
}

pub fn issue_read_by_identifier_query() -> String {
    format!(
        "query IssueByIdentifier($teamKey: String!, $number: Float!) {{
            issues(
                filter: {{
                    team: {{ key: {{ eq: $teamKey }} }},
                    number: {{ eq: $number }}
                }},
                first: 1
            ) {{
                nodes {{
                    {ISSUE_CORE_FIELDS}
                    {ISSUE_RELATIONS}
                    {ISSUE_COMMENTS_FRAGMENT}
                }}
            }}
        }}"
    )
}

pub fn issues_search_query() -> String {
    format!(
        "query IssuesSearch($term: String!, $first: Int!, $filter: IssueFilter) {{
            searchIssues(term: $term, first: $first, filter: $filter, orderBy: updatedAt, includeArchived: true) {{
                nodes {{
                    {ISSUE_CORE_FIELDS}
                    {ISSUE_RELATIONS}
                }}
            }}
        }}"
    )
}

pub const RESOLVE_ISSUE_BY_IDENTIFIER: &str = "
    query ResolveIssueByIdentifier($teamKey: String!, $number: Float!) {
        issues(
            filter: {
                team: { key: { eq: $teamKey } },
                number: { eq: $number }
            },
            first: 1
        ) {
            nodes {
                id
                identifier
                title
            }
        }
    }
";

pub const ISSUE_CREATE: &str = "
    mutation IssueCreate($input: IssueCreateInput!) {
        issueCreate(input: $input) {
            success
            issue {
                id
                identifier
                title
            }
        }
    }
";

pub const ISSUE_UPDATE: &str = "
    mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
        issueUpdate(id: $id, input: $input) {
            success
            issue {
                id
                identifier
                title
            }
        }
    }
";

pub const ISSUE_DELETE: &str = "
    mutation IssueDelete($id: String!) {
        issueDelete(id: $id) {
            success
        }
    }
";

// ── Resolve helpers ──────────────────────────────────────────────────────────

pub const RESOLVE_TEAM_BY_KEY: &str = "
    query ResolveTeamByKey($key: String!) {
        teams(filter: { key: { eq: $key } }, first: 1) {
            nodes { id key name }
        }
    }
";

pub const RESOLVE_TEAM_BY_NAME: &str = "
    query ResolveTeamByName($name: String!) {
        teams(filter: { name: { eq: $name } }, first: 1) {
            nodes { id key name }
        }
    }
";

pub const RESOLVE_PROJECT_BY_NAME: &str = "
    query ResolveProjectByName($name: String!) {
        projects(filter: { name: { eq: $name } }, first: 1) {
            nodes { id name }
        }
    }
";

pub const RESOLVE_WORKFLOW_STATES: &str = "
    query ResolveWorkflowStates($teamId: ID!) {
        workflowStates(filter: { team: { id: { eq: $teamId } } }) {
            nodes { id name type }
        }
    }
";

// ── Comments ─────────────────────────────────────────────────────────────────

pub const COMMENT_CREATE: &str = "
    mutation CommentCreate($input: CommentCreateInput!) {
        commentCreate(input: $input) {
            success
            comment {
                id
                body
                user { name }
            }
        }
    }
";

pub const COMMENT_UPDATE: &str = "
    mutation CommentUpdate($id: String!, $input: CommentUpdateInput!) {
        commentUpdate(id: $id, input: $input) {
            success
            comment {
                id
                body
                user { name }
            }
        }
    }
";

pub const COMMENT_DELETE: &str = "
    mutation CommentDelete($id: String!) {
        commentDelete(id: $id) {
            success
        }
    }
";

// ── Labels CRUD ─────────────────────────────────────────────────────────────

pub const LABEL_CREATE: &str = "
    mutation IssueLabelCreate($input: IssueLabelCreateInput!) {
        issueLabelCreate(input: $input) {
            success
            issueLabel {
                id
                name
                color
                isGroup
                parent { id name }
                team { id key name }
            }
        }
    }
";

pub const LABEL_UPDATE: &str = "
    mutation IssueLabelUpdate($id: String!, $input: IssueLabelUpdateInput!) {
        issueLabelUpdate(id: $id, input: $input) {
            success
            issueLabel {
                id
                name
                color
                isGroup
                parent { id name }
                team { id key name }
            }
        }
    }
";

pub const LABEL_DELETE: &str = "
    mutation IssueLabelDelete($id: String!) {
        issueLabelDelete(id: $id) {
            success
        }
    }
";

// ── Cycles ───────────────────────────────────────────────────────────────────

pub const CYCLES_LIST: &str = "
    query CyclesList($first: Int, $filter: CycleFilter) {
        cycles(first: $first, filter: $filter, orderBy: updatedAt) {
            nodes {
                id
                name
                number
                startsAt
                endsAt
                progress
                team { key name }
            }
        }
    }
";

pub fn cycle_read_query() -> String {
    format!(
        "query CycleRead($id: String!, $issuesFirst: Int!) {{
            cycle(id: $id) {{
                id
                name
                number
                startsAt
                endsAt
                progress
                team {{ key name }}
                issues(first: $issuesFirst) {{
                    nodes {{
                        {ISSUE_CORE_FIELDS}
                        {ISSUE_RELATIONS}
                    }}
                }}
            }}
        }}"
    )
}

pub const CYCLE_CREATE: &str = "
    mutation CycleCreate($input: CycleCreateInput!) {
        cycleCreate(input: $input) {
            success
            cycle {
                id
                name
                number
                startsAt
                endsAt
                progress
                team { key name }
            }
        }
    }
";

pub const CYCLE_UPDATE: &str = "
    mutation CycleUpdate($id: String!, $input: CycleUpdateInput!) {
        cycleUpdate(id: $id, input: $input) {
            success
            cycle {
                id
                name
                number
                startsAt
                endsAt
                progress
                team { key name }
            }
        }
    }
";

pub const RESOLVE_CYCLE_BY_NAME: &str = "
    query ResolveCycleByName($teamId: ID!, $name: String!) {
        cycles(
            filter: {
                team: { id: { eq: $teamId } },
                name: { eq: $name }
            },
            first: 1
        ) {
            nodes { id name number }
        }
    }
";

// ── Project Milestones ───────────────────────────────────────────────────────

pub const PROJECT_MILESTONES_LIST: &str = "
    query ProjectMilestonesList($projectId: ID!, $first: Int!) {
        projectMilestones(
            filter: { project: { id: { eq: $projectId } } },
            first: $first
        ) {
            nodes {
                id
                name
                description
                targetDate
            }
        }
    }
";

pub fn project_milestone_read_query() -> String {
    format!(
        "query ProjectMilestoneRead($id: String!, $issuesFirst: Int!) {{
            projectMilestone(id: $id) {{
                id
                name
                description
                targetDate
                issues(first: $issuesFirst) {{
                    nodes {{
                        {ISSUE_CORE_FIELDS}
                        {ISSUE_RELATIONS}
                    }}
                }}
            }}
        }}"
    )
}

pub const PROJECT_MILESTONE_CREATE: &str = "
    mutation ProjectMilestoneCreate($input: ProjectMilestoneCreateInput!) {
        projectMilestoneCreate(input: $input) {
            success
            projectMilestone {
                id
                name
                description
                targetDate
            }
        }
    }
";

pub const PROJECT_MILESTONE_UPDATE: &str = "
    mutation ProjectMilestoneUpdate($id: String!, $input: ProjectMilestoneUpdateInput!) {
        projectMilestoneUpdate(id: $id, input: $input) {
            success
            projectMilestone {
                id
                name
                description
                targetDate
            }
        }
    }
";

pub const PROJECT_MILESTONE_DELETE: &str = "
    mutation ProjectMilestoneDelete($id: String!) {
        projectMilestoneDelete(id: $id) {
            success
        }
    }
";

pub const RESOLVE_MILESTONE_BY_NAME: &str = "
    query ResolveMilestoneByName($projectId: ID!, $name: String!) {
        projectMilestones(
            filter: {
                project: { id: { eq: $projectId } },
                name: { eq: $name }
            },
            first: 1
        ) {
            nodes { id name }
        }
    }
";

// ── Documents ────────────────────────────────────────────────────────────────

pub const DOCUMENTS_LIST: &str = "
    query DocumentsList($first: Int!, $filter: DocumentFilter) {
        documents(first: $first, filter: $filter, orderBy: updatedAt) {
            nodes {
                id
                title
                content
                project { name }
            }
        }
    }
";

pub const DOCUMENT_READ: &str = "
    query DocumentRead($id: String!) {
        document(id: $id) {
            id
            slugId
            title
            icon
            color
            content
            createdAt
            updatedAt
            project { id name }
            creator { id name displayName }
        }
    }
";

pub const DOCUMENT_CREATE: &str = "
    mutation DocumentCreate($input: DocumentCreateInput!) {
        documentCreate(input: $input) {
            success
            document {
                id
                title
                content
                project { name }
            }
        }
    }
";

pub const DOCUMENT_UPDATE: &str = "
    mutation DocumentUpdate($id: String!, $input: DocumentUpdateInput!) {
        documentUpdate(id: $id, input: $input) {
            success
            document {
                id
                title
                content
                project { name }
            }
        }
    }
";

pub const DOCUMENT_DELETE: &str = "
    mutation DocumentDelete($id: String!) {
        documentDelete(id: $id) {
            success
        }
    }
";

// ── Attachments (for document linking) ───────────────────────────────────────

pub const ATTACHMENT_CREATE: &str = "
    mutation AttachmentCreate($input: AttachmentCreateInput!) {
        attachmentCreate(input: $input) {
            success
            attachment {
                id
                title
                url
            }
        }
    }
";

pub const ATTACHMENTS_FOR_ISSUE: &str = "
    query AttachmentsForIssue($issueId: ID!) {
        attachments(filter: { issue: { id: { eq: $issueId } } }) {
            nodes {
                id
                title
                url
                metadata
            }
        }
    }
";

// ── File Upload ──────────────────────────────────────────────────────────────

pub const FILE_UPLOAD_URL: &str = "
    mutation FileUpload($contentType: String!, $filename: String!, $size: Int!) {
        fileUpload(contentType: $contentType, filename: $filename, size: $size) {
            uploadFile {
                uploadUrl
                assetUrl
                headers {
                    key
                    value
                }
            }
        }
    }
";
