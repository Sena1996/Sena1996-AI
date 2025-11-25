# SENA1996-AI Elite Coding Standards v2.1

**MANDATORY RULES - STRICT ENFORCEMENT**

---

## RULE 0: SELF-REMINDER (CRITICAL)

**YOU MUST** acknowledge at the start of EVERY coding response:
> "Following SENA Elite Standards v2.1"

This rule cannot be skipped. It ensures all other rules are remembered.

---

## CRITICAL RULES (🔴 MUST FIX)

### Error Handling
- **YOU MUST** use `Result<T, SenaError>` for all fallible operations
- **NEVER** use `unwrap()` in production code - use `?` or `ok_or()`
- **NEVER** silently ignore errors with `let _ =`
- **YOU MUST** propagate errors with `?` operator

### Code Quality
- **NEVER** add comments - code MUST self-explain through names
- **YOU MUST** use descriptive names (no abbreviations)
- **YOU MUST** keep functions under 20 lines
- **NEVER** use mutable global state (`static mut`)

### Safety
- **NEVER** use `unsafe` without SAFETY comment justification
- **YOU MUST** validate all external input
- **NEVER** hardcode secrets or API keys

### Build
- **YOU MUST** have zero warnings: `cargo build --release`
- **YOU MUST** pass all tests: `cargo test`
- **YOU MUST** pass clippy: `cargo clippy -- -D warnings`

---

## IMPORTANT RULES (⚠️ SHOULD FIX)

### Rust Idioms
- Prefer `&str` over `String` in function parameters
- Prefer iterators over `for` loops
- Use `impl Trait` for flexible return types
- Implement `From/Into` for type conversions
- Use `derive` macros appropriately

### Performance
- Be aware of Big O complexity - avoid O(n²)
- Avoid unnecessary allocations in loops
- Use `Vec::with_capacity()` for known sizes
- Use early return / guard clauses
- Profile before optimizing

### Architecture
- Follow Single Responsibility Principle (SRP)
- Depend on abstractions (traits), not concrete types
- Dependencies point inward (domain has no external deps)
- Use Builder pattern for complex construction

### Clean Code
- DRY - Don't Repeat Yourself
- KISS - Keep It Simple
- YAGNI - You Aren't Gonna Need It
- Separation of Concerns

---

## BEST PRACTICES (ℹ️ NICE TO HAVE)

### Testing
- Unit tests for public functions
- Test edge cases (empty, max, invalid)
- Use `cargo fmt` for formatting

### Async
- Use async ONLY for I/O-bound operations
- Never block in async context
- Prefer sync when possible

### Memory
- Prefer borrowing over moving
- Use `Cow` for conditional ownership
- Limit lifetime annotations (use elision)

### Code Review
- PR under 400 lines
- Atomic commits (single logical change)
- Self-review before submission

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

---

## SEVERITY DISTRIBUTION

| Level | Count | Action |
|-------|-------|--------|
| 🔴 CRITICAL | 12 | MUST fix |
| ⚠️ IMPORTANT | 16 | SHOULD fix |
| ℹ️ BEST PRACTICE | 12 | Nice to have |
| **TOTAL** | **40** | |

---

## ENFORCEMENT CHECKLIST

Before EVERY commit:
- [ ] Zero warnings (`cargo build --release`)
- [ ] All tests pass (`cargo test`)
- [ ] No `unwrap()` in non-test code
- [ ] Descriptive names throughout
- [ ] Functions < 20 lines

---

**SENA 🦁 Elite Coding Standards v2.1**

IMPORTANT: These rules are MANDATORY. Claude MUST follow them.

Sources: Pragmatic Programmer, Clean Code, Rust API Guidelines, Microsoft Rust Guidelines
