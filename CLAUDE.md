# SENA1996-AI Elite Coding Standards v9.0.4

**MANDATORY RULES - STRICT ENFORCEMENT**

---

## RULE 0: SELF-REMINDER (CRITICAL)

**YOU MUST** acknowledge at the start of EVERY coding response:
> "Following SENA Elite Standards v9.0.4"

This rule cannot be skipped. It ensures all other rules are remembered.

---

## CRITICAL RULES (🔴 MUST FIX) - 15 Rules

### Error Handling (4)
- **YOU MUST** use `Result<T, SenaError>` for all fallible operations
- **NEVER** use `unwrap()` in production code - use `?` or `ok_or()`
- **NEVER** silently ignore errors with `let _ =`
- **YOU MUST** propagate errors with `?` operator

### Code Quality (4)
- **NEVER** add comments - code MUST self-explain through names
- **YOU MUST** use descriptive names (no abbreviations)
- **YOU MUST** keep functions under 20 lines
- **NEVER** use mutable global state (`static mut`)

### Safety (4)
- **NEVER** use `unsafe` without SAFETY comment justification
- **YOU MUST** validate all external input
- **NEVER** hardcode secrets or API keys
- **YOU MUST** sanitize file paths (prevent traversal)

### Build (3)
- **YOU MUST** have zero warnings: `cargo build --release`
- **YOU MUST** pass all tests: `cargo test`
- **YOU MUST** pass clippy: `cargo clippy -- -D warnings`

---

## IMPORTANT RULES (⚠️ SHOULD FIX) - 20 Rules

### SOLID Principles (5)
- Single Responsibility - one struct/module = one reason to change
- Open/Closed - open for extension, closed for modification
- Liskov Substitution - implementations substitutable for traits
- Interface Segregation - small focused traits, not fat interfaces
- Dependency Inversion - depend on abstractions (traits)

### Rust Idioms (5)
- Prefer `&str` over `String` in function parameters
- Prefer iterators over `for` loops
- Use `impl Trait` for flexible return types
- Implement `From/Into` for type conversions
- Use `derive` macros appropriately

### Performance (5)
- Be aware of Big O complexity - avoid O(n²)
- Avoid unnecessary allocations in loops
- Use `Vec::with_capacity()` for known sizes
- Use early return / guard clauses
- Profile before optimizing

### Clean Code (5)
- DRY - Don't Repeat Yourself
- KISS - Keep It Simple
- YAGNI - You Aren't Gonna Need It
- Separation of Concerns
- ETC - Easier To Change

---

## BEST PRACTICES (ℹ️ NICE TO HAVE) - 15 Rules

### Testing (4)
- Unit tests for public functions
- Test edge cases (empty, max, invalid)
- Use `cargo fmt` for formatting
- High test coverage for critical paths

### Async (3)
- Use async ONLY for I/O-bound operations
- Never block in async context
- Prefer sync when possible

### Memory (4)
- Prefer borrowing over moving
- Use `Cow` for conditional ownership
- Limit lifetime annotations (use elision)
- Prefer stack over heap when possible

### Code Review (4)
- PR under 400 lines
- Atomic commits (single logical change)
- Self-review before submission
- Explain why, not what

---

## QUICK REFERENCE

| Anti-Pattern | Elite Pattern |
|--------------|---------------|
| `unwrap()` | `?` or `ok_or()` |
| `String` param | `&str` param |
| Comments | Self-documenting names |
| `for` loop | Iterator chain |
| Nested `if` | Early return |
| `panic!()` | `Result` type |
| `unsafe` | Safe Rust |
| Fat trait | Focused traits |
| God object | Single responsibility |
| Concrete types | Trait bounds |

---

## SEVERITY DISTRIBUTION

| Level | Symbol | Count | Percentage |
|-------|--------|-------|------------|
| CRITICAL | 🔴 | 15 | 30% |
| IMPORTANT | ⚠️ | 20 | 40% |
| BEST PRACTICE | ℹ️ | 15 | 30% |
| **TOTAL** | | **50** | **100%** |

---

## ENFORCEMENT CHECKLIST

Before EVERY commit:
- [ ] Zero warnings (`cargo build --release`)
- [ ] All tests pass (`cargo test`)
- [ ] No `unwrap()` in non-test code
- [ ] Descriptive names throughout
- [ ] Functions < 20 lines

---

**SENA 🦁 Elite Coding Standards v9.0.4**

IMPORTANT: These rules are MANDATORY. Claude MUST follow them.

Sources: Pragmatic Programmer, Clean Code, Rust API Guidelines, Microsoft Rust Guidelines, Idiomatic Rust
