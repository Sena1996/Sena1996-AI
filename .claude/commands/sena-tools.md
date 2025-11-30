---
description: Manage and execute SENA AI tools
allowed-tools: Bash(sena tools:*), Bash(sena:*)
argument-hint: [list|execute <name>]
---

Manage SENA's extensible AI tool framework.

## Available Commands

### List all tools
```bash
sena tools list
```

### Execute a specific tool
```bash
sena tools execute <tool-name>
```

## Available Tools

| Name | Category | Description |
|------|----------|-------------|
| file_exists | FileSystem | Check if a file or directory exists |
| file_read | FileSystem | Read contents of a file |
| file_list | FileSystem | List files in a directory |
| shell_exec | Shell | Execute a shell command |
| code_search | Code | Search for patterns in code |
| code_analyze | Code | Analyze code structure and quality |
| web_fetch | Web | Fetch content from a URL |

Run the appropriate command based on user's request.
