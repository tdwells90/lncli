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
