# Developer Notice

This directory contains the project-level development rules and conventions used by **PCF (Processing Compact File)**.

If you are changing the project, take a look here before committing. These files are meant to keep the repository consistent without putting project-specific rules directly into the main README.

## Contributions

PCF is currently experimental and its structure may change as the project develops.

When contributing:

- Keep changes focused on what you are actually changing.
- Follow the existing Rust workspace and crate structure.
- Keep the core independent from ecosystem-specific code.
- Do not add SSAR-specific behaviour to PCF Core.
- Keep documentation and specifications in sync with implementation changes.
- Make sure new code is formatted and tested before opening a pull request.
- Do not remove licensing or attribution information.

## Commit Labels

PCF uses short labels at the beginning of commit titles. The label should describe the main reason for the commit.

Use the most specific label that fits. `Misc` is there for changes that genuinely do not fit another category.

### `[Core]`

Changes to the PCF Core itself.

This includes the parser, processing logic, core data structures, format definitions, and other fundamental parts of PCF.

```text
[Core] Implement PCF language foundation
[Core] Update expression parsing
```

### `[Feature]`

Adds a new user-facing or project-level capability.

Use this when the change introduces something that did not exist before and is not specific to another area.

```text
[Feature] Add PCF validation support
```

### `[Fix]`

Fixes something that was not working as intended.

```text
[Fix] Handle invalid PCF tokens correctly
```

### `[Refactor]`

Changes the implementation without intentionally changing its behaviour.

Use this for reorganising code, simplifying internals, splitting modules, or improving the architecture.

```text
[Refactor] Split parser into dedicated modules
```

### `[Std]`

Changes to PCF Standard Libraries.

```text
[Std] Add file processing utilities
```

### `[Ecosystem]`

Changes related to integrations or libraries built for a specific ecosystem.

PCF itself should remain independent from those integrations.

```text
[Ecosystem] Add SSAR PCF integration
```

### `[Docs]`

Documentation changes.

This includes the README, specifications, guides, examples, and documentation comments when the change is primarily documentation.

```text
[Docs] Update PCF format specification
```

### `[Test]`

Adds or changes tests without being primarily a feature or bug fix.

```text
[Test] Add parser edge cases
```

### `[Build]`

Changes to the build system, Cargo configuration, dependencies, workspace configuration, or packaging.

```text
[Build] Update workspace dependencies
```

### `[CI]`

Changes to continuous integration or GitHub Actions.

```text
[CI] Add Rust workspace checks
```

### `[Chore]`

Maintenance work that is useful but does not change the actual PCF implementation.

```text
[Chore] Update development tooling
```

### `[Misc]`

Small or miscellaneous changes that do not fit any of the labels above.

Do not use `[Misc]` just because the change contains several files. If the change has a clear purpose, use the corresponding label.

```text
[Misc] Update repository metadata
[Misc] Clean up project configuration
```

## Choosing a Label

When a commit could fit several labels, choose the one that best describes the main purpose of the change.

For example:

```text
[Core] Fix parser error handling
```

is better than:

```text
[Fix] Update parser
```

when the important part of the change is the Core parser itself.

Likewise:

```text
[Refactor] Reorganize parser modules
```

is preferred when the parser is being reorganized without changing its behaviour.

Keep commit titles short and specific. The description can be used when the change needs more context.

A useful format is:

```text
[Label] Short description

Optional explanation of what changed and why.
```

## Source File Headers

PCF is a Rust project. Rust source files do not use C/C++-style header files, and there is currently no need to introduce artificial `.h` files just to provide declarations.

For this reason, **PCF does not require a separate header file system**.

The same applies to the PCF format itself: format definitions and language structures should live in the appropriate Rust modules and documentation rather than in a traditional header file.

If PCF later introduces a public C/C++ API, FFI layer, or another language binding that genuinely requires header files, those headers should be treated as part of that specific binding rather than as part of PCF Core.

### Source File Copyright Header

For now, PCF source files do not need the large OpenStudio-style header containing project, module, repository, and version metadata.

Rust already provides strong module and package metadata through `Cargo.toml`, and repeating the same information in every source file creates maintenance overhead.

If a source file requires a copyright or license notice for legal or distribution reasons, use the project's approved SPDX/license convention rather than inventing a per-file format.

## Project Separation

The PCF repository is intended to contain the general-purpose PCF implementation.

Keep ecosystem-specific code separate when it is not required by the PCF specification or Core implementation.

For example, SSAR-specific functionality should normally live in an ecosystem library rather than being added directly to PCF Core.

The goal is to keep PCF useful outside of SSAR while still allowing SSAR and other ecosystems to build on top of it.

## Repository

Repository: https://github.com/ssar-group/pcf

Website: https://ssar-group.com

---

_Copyright © 2026 SSAR Group._
