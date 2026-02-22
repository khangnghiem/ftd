---
description: Write tests before implementation (TDD)
---

# Test Workflow

// turbo-all

1. **Identify what to test**: Read the acceptance criteria from the spec or PR description.

2. **Write tests FIRST** in the relevant crate:
   - Parser features → `crates/fd-core/src/parser.rs` (in `mod tests`)
   - Emitter features → `crates/fd-core/src/emitter.rs` (in `mod tests`)
   - Layout features → `crates/fd-core/src/layout.rs` (in `mod tests`)
   - Sync features → `crates/fd-editor/src/sync.rs` (in `mod tests`)
   - Tool features → `crates/fd-editor/src/tools.rs` (in `mod tests`)
   - Hit testing → `crates/fd-render/src/hit.rs` (in `mod tests`)
   - Integration → `crates/fd-editor/tests/` (fixture-based)
   - Integration → `crates/fd-core/tests/` (roundtrip fixtures)

3. **Test naming convention**:

   ```rust
   #[test]
   fn parse_<feature>() { ... }

   #[test]
   fn emit_<feature>() { ... }

   #[test]
   fn roundtrip_<feature>() { ... }

   #[test]
   fn layout_<feature>() { ... }

   #[test]
   fn sync_<direction>_<feature>() { ... }

   #[test]
   fn tool_<toolname>_<behavior>() { ... }
   ```

4. **Run tests** to confirm they fail (red):

   ```bash
   cargo test --workspace
   ```

5. **Implement** the feature to make tests pass (green).

6. **Run again** to confirm green:

   ```bash
   cargo test --workspace
   ```

7. **Check for regressions**:

   ```bash
   cargo test --workspace -- --nocapture 2>&1 | tail -20
   ```

8. **Edge cases to always test**:
   - Empty / missing input
   - Nested structures (groups within groups)
   - Special characters in strings / IDs
   - Round-trip: parse → emit → re-parse produces identical graph
   - Boundary values (0-size rect, empty path)
   - Invalid / malformed `.fd` input (error paths)
   - Modifier key combos (Shift, Alt, Ctrl+Meta)

---

## Test Coverage Rules

Every requirement marked as done in `docs/CHANGELOG.md` **MUST** have at least:

| Layer           | Required Tests                                   |
| --------------- | ------------------------------------------------ |
| Parser feature  | `parse_<feature>` + `roundtrip_<feature>`        |
| Emitter feature | `emit_<feature>` + `roundtrip_<feature>`         |
| Tool feature    | `tool_<name>_<behavior>` for each modifier combo |
| Sync feature    | `sync_<direction>_<feature>`                     |
| Layout feature  | `layout_<feature>` or `resolve_<feature>`        |

### Coverage Tracking

After adding tests, update the **Test Matrix** section in `docs/CHANGELOG.md` to map each requirement to its test functions.

### Golden-File Roundtrip Tests

Every `.fd` file in `examples/` should have a matching roundtrip assertion:

```rust
#[test]
fn roundtrip_<example_name>() {
    let input = include_str!("../../examples/<name>.fd");
    let graph = parse_document(input).unwrap();
    let output = emit_document(&graph);
    let graph2 = parse_document(&output).unwrap();
    let output2 = emit_document(&graph2);
    assert_eq!(output, output2, "roundtrip must be stable");
}
```

### Negative Tests

Every parser feature should include at least one invalid-input test:

```rust
#[test]
fn parse_<feature>_invalid() {
    let result = parse_document("<malformed input>");
    assert!(result.is_err());
}
```
