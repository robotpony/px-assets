# CLI Output Visuals Plan

A plan for making `px` output informative, scannable, and pleasant to read. The goal: Cargo-level polish without over-engineering.

## Current State

Every command prints plain text. No colours, no alignment, no grouping. Here's what each command outputs today:

### `px build` (directory)

```
player-stand -> /tmp/out/player-stand.png + /tmp/out/player-stand.json
robot-idle -> /tmp/out/robot-idle.png + /tmp/out/robot-idle.json
cabinet -> /tmp/out/cabinet.png + /tmp/out/cabinet.json
...
Built 16 asset(s) to /tmp/out
```

**Problems:**
- Full absolute paths dominate every line, burying the asset name
- No visual distinction between asset types (shapes, prefabs, maps)
- No size or dimension info (was the sprite 8x8 or 128x128?)
- Summary line is plain

### `px build --sheet`

```
Packed 15 sprite(s) into /tmp/out/sheet.png + /tmp/out/sheet.json
```

**Problems:**
- Single terse line, no per-asset detail
- No sheet dimensions or packing stats

### `px build` (discovery mode, no args)

```
Discovered 12 shapes, 2 prefabs, 1 maps    (stderr)
player-stand -> /tmp/out/player-stand.png ...
...
Built 16 asset(s) to /tmp/out
```

**Problems:**
- Discovery summary goes to stderr, build output to stdout (correct for piping but looks disjointed in terminal)
- "1 maps" should be "1 map"

### `px validate`

```
Validating 11 asset(s)...
Validation passed.
```

Or with issues:

```
  error[E001]: Shape 'robot' references undefined palette 'neon'
    help: Available palettes: default, dark
  warning[W002]: Unused stamp 'legacy' in style 'main'
Validation failed: 1 error(s), 1 warning(s)
```

**Problems:**
- No colour for error/warning severity
- Codes like `E001` are good but would benefit from colour
- "asset(s)" phrasing is clunky

### `px init`

```
Created px.yaml with 3 source directories (15 assets found)
```

**Problems:**
- No detail about what was discovered
- Doesn't show the generated file content or even a sample

### Error output (via miette)

```
Error: px::build

  x Build error: px.yaml already exists
  help: Use --force to overwrite
```

**This is already good.** Miette handles fancy error formatting with colours and Unicode. No changes needed here.

## Design Principles

1. **Cargo as reference**: Use Cargo's output conventions where they fit. Right-aligned coloured verbs, clean indentation, terse but informative.
2. **Colour is information, not decoration**: Green = success/action, yellow = warning, red = error, cyan = file paths, bold = names. That's it.
3. **Quiet by default, verbose on request**: Normal output shows the essentials. `--verbose` (future) adds dimensions, file sizes, timing.
4. **Respect pipes**: Detect `isatty`. No colours or Unicode when output is piped. Use a library that handles this automatically.
5. **No progress bars**: Builds are fast (sub-second). Progress bars would just flash and disappear.

## Proposed Output

### Library choice

**`owo-colors`** with `supports-color` for detection. Lightweight (zero-alloc), no macros, works with `format!`. Avoids the heavier `colored` crate.

Alternative: raw ANSI escapes behind an `isatty` check. Simpler, zero dependencies, but more manual. Either works.

### `px build` (directory)

```
   Compiling player-stand (8x12, 2 frames)
   Compiling player-walk-1 (8x12)
   Compiling player-walk-2 (8x12)
   Compiling robot-idle (10x12)
   Compiling robot-alert (10x12)
   Composing cabinet (16x18)
   Composing terminal (8x12)
    Charting level-example (160x42, 6 shapes)
    Finished 16 assets -> dist/
```

Formatting rules:
- **Verb** is right-aligned to 12 chars, bold green. "Compiling" for shapes, "Composing" for prefabs, "Charting" for maps.
- **Asset name** is bold white.
- **Dimensions** in parentheses, dim/grey. Width x height in pixels (post-scale).
- **Extra info** after dimensions: frame count for multi-frame shapes, shape count for maps.
- **Summary line** uses "Finished" verb. Shows total count and output directory (relative path when possible).
- Paths are NOT shown per-line. The output directory is stated once at the end.

### `px build --sheet`

```
   Compiling player-stand (8x12)
   Compiling player-walk-1 (8x12)
   ...
     Packing 15 sprites into sheet (128x64)
    Finished sheet.png + sheet.json -> dist/
```

- "Packing" line appears after all individual assets, shows sprite count and sheet dimensions.
- Omits per-sprite output lines when `--sheet` is active (they all go into the sheet anyway). Or keeps them; user preference.

### `px build` (discovery mode)

```
  Discovered 12 shapes, 2 prefabs, 1 map (using px.yaml)
   Compiling player-stand (8x12)
   ...
    Finished 16 assets -> dist/
```

- "Discovered" verb, bold cyan, same right-alignment.
- Pluralization fixed ("1 map" not "1 maps").
- Manifest note integrated naturally.

### `px validate`

```
  Validating 11 assets...
      error[E001]: Shape 'robot' references undefined palette 'neon'
             help: Available palettes: default, dark
    warning[W002]: Unused stamp 'legacy' in style 'main'
      Failed 1 error, 1 warning
```

Or clean:

```
  Validating 11 assets...
     Passed all clear
```

Formatting rules:
- "error" in bold red, "warning" in bold yellow.
- Diagnostic codes in dim.
- "help:" in cyan.
- Summary: "Passed" in green or "Failed" in red, both bold.

### `px init`

```
   Scanning .
  Discovered shapes/, prefabs/, maps/
    Created px.yaml (3 sources, 15 assets)
```

- Shows what directories were found.
- "Created" in bold green.

### Error output

No changes. Miette already handles this well.

## Implementation

### New module: `src/output.rs`

A small formatting helper, not a framework. Approximately:

```rust
use std::io::{self, Write, IsTerminal};

pub struct Printer {
    use_color: bool,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            use_color: io::stderr().is_terminal(),
        }
    }

    /// Print a status line: right-aligned verb + message
    /// e.g. "   Compiling player-stand (8x12)"
    pub fn status(&self, verb: &str, message: &str) { ... }

    /// Print a success status (green verb)
    pub fn success(&self, verb: &str, message: &str) { ... }

    /// Print a warning status (yellow verb)
    pub fn warning(&self, verb: &str, message: &str) { ... }

    /// Print an error status (red verb)
    pub fn error(&self, verb: &str, message: &str) { ... }
}
```

Key decisions:
- All status output goes to **stderr** (matching Cargo's convention; stdout is reserved for machine-readable output like JSON).
- Right-align verb to 12 characters.
- Colour codes: green (success/action), yellow (warning), red (error), cyan (info/paths).
- `IsTerminal` is in std since Rust 1.70, so no extra dependency needed for detection.

### Dependency choice

Option A: **Zero new dependencies.** Use raw ANSI escape codes (`\x1b[1;32m` etc.) behind `IsTerminal` check. The `Printer` module is ~60 lines.

Option B: **Add `owo-colors`** (~15KB, well-maintained). Gives `.green().bold()` method chaining. Slightly more readable code, handles edge cases.

Recommendation: **Option A.** The formatting is simple enough. Four colours, bold, and dim. Raw ANSI keeps the dependency list short, and `Printer` encapsulates all of it.

### Changes per command

**`build.rs`:**
- Create `Printer` at top of `run()`
- Replace `println!("  {} -> ...")` with `printer.status("Compiling", &format!("{} ({}x{})", name, w, h))`
- Replace discovery summary `eprintln!` with `printer.status("Discovered", ...)`
- Replace final summary with `printer.success("Finished", ...)`
- Fix "1 maps" pluralization

**`validate.rs`:**
- Create `Printer`, use for "Validating" and "Passed"/"Failed" lines
- Update `print_diagnostics()` in `validation/mod.rs` to accept `&Printer` for coloured error/warning output

**`init.rs`:**
- Create `Printer`, use for "Scanning", "Discovered", "Created" lines

### Pluralization helper

Small utility, either in `output.rs` or inline:

```rust
fn plural(n: usize, singular: &str, plural: &str) -> String {
    if n == 1 { format!("{} {}", n, singular) } else { format!("{} {}", n, plural) }
}
```

### Relative path display

When the output directory is inside or relative to cwd, show the relative path. When it's absolute and outside cwd, show the absolute path. `pathdiff::diff_paths` or a simple `strip_prefix` handles this.

No new dependency needed; `std::path::Path::strip_prefix` with a fallback to the absolute path is sufficient.

## What This Plan Does NOT Include

- Progress bars or spinners (builds are too fast)
- `--quiet` / `--verbose` flags (future work)
- Coloured asset-type labels in build output (adds visual noise, not worth it yet)
- Interactive/TUI features
- Timing information ("Built in 0.3s") — nice-to-have but not in this pass

## Files Modified

- `src/output.rs` — new, ~80 lines
- `src/lib.rs` — add `pub mod output;`
- `src/cli/build.rs` — replace println/eprintln with Printer calls, add dimensions
- `src/cli/validate.rs` — use Printer for status lines
- `src/cli/init.rs` — use Printer for status lines
- `src/validation/mod.rs` — coloured diagnostics

## Verification

1. `cargo run -- build examples/mission-improbable/ -o /tmp/test` — coloured, aligned output
2. `cargo run -- build examples/mission-improbable/ -o /tmp/test 2>/dev/null` — no output (all goes to stderr)
3. `cargo run -- build examples/mission-improbable/ -o /tmp/test 2>&1 | cat` — no ANSI escapes (piped)
4. `cargo run -- validate examples/mission-improbable/` — coloured pass/fail
5. `cargo run -- init /tmp/test-init` — coloured status lines
6. `cargo test` — all existing tests pass (tests don't check output formatting)
