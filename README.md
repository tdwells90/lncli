# lncli

A Linear CLI built for AI agents, written in Rust. Defaults to [TOON](https://github.com/toon-format/spec) output to minimize token usage, with full CRUD support across Linear's core resources, with responses stripped to the essentials for your AI agent to do its job.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/tdwells90/lncli/main/install.sh | sh
```

## Authentication

Provide your Linear API token via (in priority order):

1. `--api-token <TOKEN>` flag
2. `LINEAR_API_TOKEN` environment variable
3. `~/.linear_api_token` file

## Agent setup

Add the following to your `CLAUDE.md` or `AGENTS.md` to give your AI agent access to Linear:

````markdown
## Linear

Use the **`lncli`** CLI for all Linear operations (reading issues, updating status, creating issues, adding comments, etc.). Do NOT use Linear MCP tools — use `lncli` instead.

Run `lncli` or `lncli usage` to see all available commands and usage details.

### Field filtering

Limit output to specific fields using `--fields`:

```bash
lncli issues list --fields id,identifier,title,state
[7]{id,identifier,title,state}:
  "abc123...",ENG-123,"Fix auth bug","In Progress"
```

Nested fields use dot notation:

```bash
lncli issues list --fields id,title,assignee.email,team.name
```

Mandatory fields (id, identifier) are always included.

### CLI examples

```bash
# Filter fields and nested data
lncli issues list --fields id,identifier,title,assignee.email

# Read a specific issue
lncli issues read ENG-123

# Create and update
lncli issues create --title "Bug fix" --team <key>
lncli issues update ENG-123 --status "Done"
```
````

## Output format

Output defaults to TOON — a compact, token-efficient format for LLM consumption.

```
$ lncli teams list
[4]{id,key,name,description}:
  "cdb1eb08-...",ENG,Engineering,null
  "b04263e0-...",MAR,Marketing,null
```

Pass `--format json` for standard JSON:

```
$ lncli --format json teams list
[
  {
    "id": "cdb1eb08-...",
    "key": "ENG",
    "name": "Engineering",
    "description": null
  }
]
```

## Commands

| Command | Operations |
|---|---|
| `issues` | list, read, search, create, update, delete |
| `comments` | create, update, delete |
| `documents` | create, update, read, list, delete |
| `embeds` | upload, download |
| `labels` | list, create, update, delete |
| `teams` | list |
| `users` | list |
| `projects` | list, read, create, update, delete |
| `cycles` | list, read, create, update |
| `project-milestones` | list, read, create, update, delete |

Run `lncli <command> --help` for full usage details on any command.
