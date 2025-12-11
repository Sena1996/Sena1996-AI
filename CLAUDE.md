# SENA Project Standards

## Core Rules
- Use `Result<T, E>` for errors, never `unwrap()` in production
- No comments - self-documenting names only
- Functions under 20 lines
- NEVER git commit/push without user approval

## Rust Patterns
| Avoid | Use |
|-------|-----|
| `unwrap()` | `?` / `ok_or()` |
| `String` param | `&str` |
| `for` loops | iterators |

## SENA CLI
Run via Bash: `sena health`, `sena think`, `sena agent <type>`
