# Set Session Name

Rename the current SENA session.

## Instructions

1. First check current session:
```bash
sena session info
```

2. Ask the user: "What would you like to name this session?"

3. Once they provide a name, end old session and start new one with the name:
```bash
sena session start --name "<user_provided_name>"
```

4. Confirm the change by showing session info again.

## Note
Session names help identify different Claude Code windows when collaborating.
