# Session Start

Start a new SENA session with a custom name.

## Instructions

Ask the user: "What would you like to name this session?"

Once they provide a name, run:
```bash
sena session start --name "<user_provided_name>"
```

Then display the session info:
```bash
sena session info
```

## Example Response

After starting, show:
```
🦁 SENA Session Started!

Session Name: <name>
Session ID: <id>
Started: <timestamp>

You can now collaborate with other sessions using:
- sena who        (see online sessions)
- sena tell       (send messages)
- sena inbox      (check messages)
```
