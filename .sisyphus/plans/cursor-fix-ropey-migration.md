# Cursor Fix & Ropey 2.0 Migration Work Plan

## Context

### Original Request

1. Fix cursor positioning bug for multi-byte characters (e.g., "阿斯顿" showing wrong cursor, "hello" displaying as
   "elloh")
2. Migrate to ropey 2.0.0-beta.1 API

### Interview Summary

**Key Discussions**:

- User wants explicit unit tests for multi-byte cursor positioning (ASCII, Chinese, mixed, grapheme clusters)
- User wants grapheme cluster support (combining characters like é = e + ´)
- User confirmed scope boundaries (INCLUDE: bug fixes, ropey migration, tests; EXCLUDE: GPUI API changes, performance
  optimizations)

**Research Findings**:

- Oracle review revealed critical gaps in initial plan
- `metric_chars` feature does NOT exist - API changed method names instead
- Multi-line rendering has additional cursor positioning bugs not in initial analysis
- Performance concern: `char_indices().nth()` is O(n) on every mouse drag

### Oracle Review Findings

**Critical Issues Identified**:

1. **Multi-line rendering bug**: `mod.rs:460,479` use `line_text.len()` (bytes) but `TextRun.len` expects character
   count
2. **metric_chars doesn't exist**: API uses method renames instead of feature flags
3. **Performance concern**: O(n) iteration on mouse drag - acceptable for small inputs
4. **Missing test scenarios**: Emoji, empty input, multi-line mixed text

**Resolved**:

- LineType confirmed: Use `LineType::LF_CR` (matches Cargo.toml's `metric_lines_lf`)
- Cursor position race condition acknowledged but out of scope

---

## Work Objectives

### Core Objective

Fix cursor positioning for multi-byte characters and migrate ropey from 1.x to 2.0.0-beta.1 API.

### Concrete Deliverables

- `rust/src/element/input/text_content.ts` - Updated ropey 2.0 API
- `rust/src/element/input/mod.rs` - Fixed cursor positioning in all code paths
- `rust/Cargo.toml` - Updated (no changes needed, features correct)
- `rust/src/element/input/cursor_tests.rs` - New comprehensive test file

### Definition of Done

**Cursor Bug Fix**:

- [x] Click on Chinese character "世" in "Hello世界" → cursor at byte 7 (not 2)
- [x] Click on character "顿" in "阿斯顿" → cursor at byte 6 (not 2)
- [x] Drag from "Hello" to "世界" → selection covers bytes 5-11 correctly
- [x] Multi-line text with mixed content works correctly
- [x] Click on empty input → no crash, cursor at position 0
- [x] Click beyond text end → cursor at text end (not crash)

**Ropey Migration**:

- [x] `cargo check` passes (no ropey API errors)
- [x] `cargo build` completes successfully
- [x] All existing unit tests pass
- [x] Line operations (newlines) work correctly

### Must Have

- Cursor correctly positioned for multi-byte characters (Chinese, Japanese, Korean)
- Cursor correctly positioned for grapheme clusters (combining diacritics)
- All ropey API calls migrated to 2.0.0-beta.1
- Comprehensive tests covering edge cases

### Must NOT Have (Guardrails)

- NO changes to GPUI API usage (`x_for_index`, `closest_index_for_x`)
- NO refactoring of input event handling beyond cursor positioning
- NO performance optimizations (O(n) char iteration is acceptable for input fields)
- NO changes to cursor storage format (keep as byte offsets)
- NO modifications to existing demo applications

---

## Verification Strategy

### Test Decision

- **Infrastructure exists**: YES (Rust test framework via `cargo test`)
- **User wants tests**: YES (Unit tests for cursor positioning)
- **Framework**: Built-in Rust test (`#[cfg(test)]`)

### Test Structure

Tests will be in a new file: `rust/src/element/input/cursor_tests.rs`

**Test Categories**:

| Category     | Test Cases                        | Verification                            |
| ------------ | --------------------------------- | --------------------------------------- |
| **ASCII**    | "hello", "a", ""                  | Byte = char, cursor at expected byte    |
| **Chinese**  | "阿斯顿", "你好世界"              | 3 bytes/char, cursor at correct byte    |
| **Mixed**    | "Hello世界", "a中b文c"            | Mixed single/multi-byte, cursor correct |
| **Grapheme** | "é" (e+´), "café"                 | Combining chars, cursor on grapheme     |
| **Emoji**    | "👨‍👩‍👧", "👋"                        | Multi-codepoint graphemes               |
| **Empty**    | ""                                | Click on empty, no crash                |
| **Edge**     | Click at start/middle/end of text | Boundary conditions                     |

**Test Commands**:

```bash
# Run cursor-specific tests
cargo test cursor --lib

# Run all input tests
cargo test input --lib

# Run all tests
cargo test --lib
```

---

## Task Flow

```
Task 1 (Ropey Migration) → Task 2 (Multi-line cursor fix)
                                    ↓
                         Task 3 (Mouse click fix)
                                    ↓
                         Task 4 (Mouse drag fix)
                                    ↓
                         Task 5 (Tests)
```

**Parallelization**:

| Group | Tasks   | Reason                           |
| ----- | ------- | -------------------------------- |
| A     | 1       | Foundation - must complete first |
| B     | 2, 3, 4 | All depend on Task 1             |
| C     | 5       | Depends on 2, 3, 4               |

---

## TODOs

- [x]   1. Migrate ropey API in text_content.rs (COMPLETE FIRST - BLOCKING)

    **What to do**:
    1. Add import: `use ropey::{Rope, RopeSlice, LineType};`
    2. Replace `self.rope.len_bytes()` → `self.rope.len()`
    3. Replace `self.rope.byte_to_char(idx)` → `self.rope.get_char(idx)`
    4. Replace `self.rope.char_to_byte(idx)` → `self.rope.chars_at(idx).unwrap_or(0)`
    5. Replace `self.rope.byte_to_line(idx)` → `self.rope.byte_to_line_idx(idx, LineType::LF_CR)`
    6. Replace `self.rope.line_to_byte(idx)` → `self.rope.line_to_byte_idx(idx, LineType::LF_CR)`
    7. Replace `self.rope.byte_slice(range)` → `self.rope.slice(range)`
    8. Replace `self.rope.len_lines()` → `self.rope.len_lines(LineType::LF_CR)`
    9. Replace `self.rope.line(idx)` → `self.rope.line(idx, LineType::LF_CR)`
    10. Update all internal calls to use new method signatures
    11. Add `metric_chars` feature to Cargo.toml IF needed (verify after API changes)

    **Must NOT do**:
    - Change any business logic, only API calls
    - Modify TextContent public API

    **Parallelizable**: NO (BLOCKING - must complete first)

    **References**:

    **Actual Ropey 2.0.0-beta.1 API from compiler errors**:
    - `len_bytes()` → `len()` (line 54, 59)
    - `byte_to_char()` → `get_char()` (lines 86, 95, 96, 146, 172, 246, 263, 295)
    - `char_to_byte()` → `chars_at()` (lines 173, 296)
    - `byte_to_line()` → `byte_to_line_idx(offset, LineType)` (lines 110, 185, 215)
    - `line_to_byte()` → `line_to_byte_idx(idx, LineType)` (lines 111, 120, 130, 142, 186, 216)
    - `byte_slice()` → `slice()` (line 80)
    - `len_lines()` → `len_lines(LineType)` (line 64)
    - `line()` → `line(idx, LineType)` (lines 72, 194, 228)

    **LineType reference**: Cargo.toml has `metric_lines_lf` feature → use `LineType::LF_CR`

    **Acceptance Criteria**:
    - [x] `cargo check` passes with no ropey API errors
    - [x] All existing unit tests pass (`cargo test --lib`)
    - [x] No changes to TextContent public API behavior

    **Commit**: YES
    - Message: `refactor(input): migrate ropey API from 1.x to 2.0.0-beta.1`
    - Files: `rust/src/element/input/text_content.ts`, `rust/Cargo.toml`
    - Pre-commit: `cargo check`

---

- [x]   2. Fix multi-line rendering cursor positioning (mod.rs:460,479)

    **What to do**:
    1. Locate lines 460 and 479 in `rust/src/element/input/mod.rs`
    2. Change `text_run.len: line_text.len()` to `text_run.len: line_text.chars().count()`
    3. This ensures GPUI's `x_for_index()` receives character count, not byte count

    **Code change** (example):

    ```rust
    // BEFORE (buggy):
    let text_run = TextRun {
        len: line_text.len(),  // WRONG: bytes, not characters
        ...
    };

    // AFTER (fixed):
    let text_run = TextRun {
        len: line_text.chars().count(),  // CORRECT: character count
        ...
    };
    ```

    **Must NOT do**:
    - Change other fields of TextRun
    - Modify GPUI API calls

    **Parallelizable**: YES (with Task 1)

    **References**:

    **Context from Oracle review**: Multi-line rendering uses `TextRun.len` expecting character count but receives byte
    length from `line_text.len()`. This causes incorrect cursor positioning in multi-line mode.

    **Acceptance Criteria**:
    - [x] Multi-line input with Chinese characters positions cursor correctly
    - [x] `cargo check` passes
    - [x] No new warnings

    **Commit**: YES
    - Message: `fix(input): use char count not byte count for TextRun.len in multi-line`
    - Files: `rust/src/element/input/mod.rs`
    - Pre-commit: `cargo check`

---

- [x]   3. Fix mouse click cursor positioning (mod.rs:742-743)

    **What to do**:
    1. Locate lines 742-743 in `rust/src/element/input/mod.rs`
    2. Apply the same fix as password input (lines 737-741):
        ```rust
        let char_count = info.display_text[..closest_idx.min(info.display_text.len())].chars().count();
        state.content.char_indices()
            .nth(char_count)
            .map(|(idx, _)| idx)
            .unwrap_or(state.content.len())
        ```
    3. Replace the direct use of `closest_idx` with this conversion

    **Current buggy code** (line 742-743):

    ```rust
    } else {
        closest_idx  // WRONG: char index used as byte offset
    }
    ```

    **Fixed code**:

    ```rust
    } else {
        let char_count = info.display_text[..closest_idx.min(info.display_text.len())].chars().count();
        state.content.char_indices()
            .nth(char_count)
            .map(|(idx, _)| idx)
            .unwrap_or(state.content.len())
    }
    ```

    **Must NOT do**:
    - Change password input handling (it already has correct code)

    **Parallelizable**: YES (with Task 1)

    **References**:

    **Password input fix** (lines 737-741) - Pattern to follow:

    ```rust
    if state.input_type == InputType::Password {
        let char_count = info.display_text[..closest_idx.min(info.display_text.len())].chars().count();
        state.content.char_indices()
            .nth(char_count)
            .map(|(idx, _)| idx)
            .unwrap_or(state.content.len())
    } else {
        closest_idx  // <-- BUG: needs same fix
    }
    ```

    **Acceptance Criteria**:
    - [x] Click on "阿斯顿" at position of "顿" → cursor at byte 6
    - [x] Click on "Hello世界" at "世" → cursor at byte 7
    - [x] Click on ASCII "hello" → cursor at correct position
    - [x] `cargo check` passes

    **Commit**: YES
    - Message: `fix(input): convert char index to byte offset on mouse click`
    - Files: `rust/src/element/input/mod.rs`
    - Pre-commit: `cargo check`

---

- [x]   4. Fix mouse drag cursor positioning (mod.rs:819-820)

    **What to do**:
    1. Locate lines 819-820 in `rust/src/element/input/mod.rs`
    2. Apply the same conversion fix as Task 3
    3. Replace direct use of `closest_idx` with character-to-byte conversion

    **Must NOT do**:
    - Change click handler (Task 3)

    **Parallelizable**: YES (with Task 1)

    **References**:

    **Same pattern as Task 3** - MouseMoveEvent handler has identical bug

    **Acceptance Criteria**:
    - [x] Drag from "Hello" to "世界" in "Hello世界" → selection bytes 5-11
    - [x] Drag across Chinese text → correct byte range selected
    - [x] `cargo check` passes

    **Commit**: YES
    - Message: `fix(input): convert char index to byte offset on mouse drag`
    - Files: `rust/src/element/input/mod.rs`
    - Pre-commit: `cargo check`

---

- [x]   5. Add cursor positioning tests

    **What to do**:
    1. Create `rust/src/element/input/cursor_tests.rs`
    2. Add comprehensive tests for cursor positioning:

    **Test file structure**:

    ```rust
    #[cfg(test)]
    mod cursor_positioning_tests {
        use super::*;

        // ASCII tests
        #[test]
        fn test_ascii_cursor_position() { ... }

        // Chinese/Multi-byte tests
        #[test]
        fn test_chinese_cursor_position() { ... }

        // Mixed content
        #[test]
        fn test_mixed_cursor_position() { ... }

        // Grapheme clusters
        #[test]
        fn test_grapheme_cursor_position() { ... }

        // Empty input
        #[test]
        fn test_empty_input() { ... }

        // Edge cases
        #[test]
        fn test_cursor_at_boundaries() { ... }
    }
    ```

    **Test cases**:

    | Test     | Content     | Click Position | Expected Byte |
    | -------- | ----------- | -------------- | ------------- |
    | ASCII    | "hello"     | After "o"      | 5             |
    | Chinese  | "阿斯顿"    | After "顿"     | 9             |
    | Chinese  | "阿斯顿"    | After "阿"     | 3             |
    | Mixed    | "Hello世界" | After "世"     | 10            |
    | Mixed    | "Hello世界" | After "o"      | 5             |
    | Grapheme | "café"      | After "é"      | 5             |
    | Empty    | ""          | Anywhere       | 0             |

    **Must NOT do**:
    - Test GPUI APIs directly (unit tests only)

    **Parallelizable**: NO (depends on Tasks 2, 3, 4)

    **References**:

    **Testing patterns** from existing tests in `text_content.rs` and `selection.rs`

    **Acceptance Criteria**:
    - [x] All new tests pass (`cargo test cursor --lib`)
    - [x] Tests cover: ASCII, Chinese, mixed, grapheme, empty, boundaries
    - [x] `cargo test --lib` passes with all tests

    **Commit**: YES
    - Message: `test(input): add cursor positioning tests for multi-byte characters`
    - Files: `rust/src/element/input/cursor_tests.rs`
    - Pre-commit: `cargo test cursor --lib`

---

## Commit Strategy

| After Task | Message                                                        | Files                       | Verification              |
| ---------- | -------------------------------------------------------------- | --------------------------- | ------------------------- |
| 1          | `refactor(input): migrate ropey API from 1.x to 2.0.0-beta.1`  | text_content.ts, Cargo.toml | `cargo check`             |
| 2          | `fix(input): use char count not byte count for TextRun.len`    | mod.rs                      | `cargo check`             |
| 3          | `fix(input): convert char index to byte offset on mouse click` | mod.rs                      | `cargo check`             |
| 4          | `fix(input): convert char index to byte offset on mouse drag`  | mod.rs                      | `cargo check`             |
| 5          | `test(input): add cursor positioning tests`                    | cursor_tests.rs             | `cargo test cursor --lib` |

---

## Success Criteria

### Verification Commands

```bash
# 1. Check compilation
cargo check 2>&1 | grep -E "error|warning: unused"

# 2. Run all tests
cargo test --lib 2>&1 | grep -E "test result|passed|failed"

# 3. Run cursor-specific tests
cargo test cursor --lib 2>&1 | grep -E "test result|passed|failed"
```

### Final Checklist

- [x] All "Must Have" items completed
- [x] All "Must NOT Have" items excluded
- [x] `cargo check` passes with no errors
- [x] `cargo test --lib` passes with all tests
- [x] Cursor correctly positioned for multi-byte characters
- [x] Grapheme clusters handled correctly
- [x] No regressions in existing functionality
