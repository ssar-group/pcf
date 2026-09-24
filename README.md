# Processing Compact File

![experimental](https://img.shields.io/badge/status-experimental-blueviolet.svg?colorA=303033&colorB=a08a2c)
![Version](https://img.shields.io/badge/version-E.0.1-blueviolet.svg?colorA=303033&colorB=6315ac)
![Platform](https://img.shields.io/badge/platform-cross--platform-000000.svg?colorA=303033&colorB=fff)
[![Quality](https://github.com/ssar-group/pcf/actions/workflows/quality.yml/badge.svg)](https://github.com/ssar-group/pcf/actions/workflows/quality.yml)

**PCF (Processing Compact File)** is a compact and extensible format for working with structured data.

The idea behind PCF is to provide a common foundation for applications, backend services, APIs, developer tools, configuration files, caches, and other systems that need to store or exchange structured data.

PCF is not tied to a specific programming language, framework, or platform. It can be used directly or as a building block for larger projects and ecosystems.

> [!NOTE]
> PCF is currently experimental and under active development. The format, APIs, and internal architecture may change as the project evolves.

## Architecture

PCF is split into several layers. Each layer has a specific purpose and can evolve without unnecessarily affecting the others.

```text
┌──────────────────────────────────────┐
│        Applications / Services       │
├──────────────────────────────────────┤
│       Ecosystem Libraries /          │
│           Integrations               │
├──────────────────────────────────────┤
│        PCF Standard Libraries        │
├──────────────────────────────────────┤
│             PCF Core                 │
└──────────────────────────────────────┘
```

### PCF Core

The core contains the parts that define how PCF works.

This includes the parser, processing pipeline, file format definitions, diagnostics, runtime components, and the data structures shared across the project.

The core is kept independent from any particular application or ecosystem. It should provide the basic building blocks without making assumptions about how they will be used.

### Standard Libraries

Standard Libraries contain functionality that is useful across different PCF projects.

They can provide utilities for data processing, serialization, formatting, networking, file handling, and other common tasks.

Keeping these libraries separate from the core makes it possible to add functionality without making the base format more complicated.

### Ecosystem Libraries

Ecosystem Libraries provide integrations for specific projects or environments.

For example, the SSAR ecosystem can maintain its own PCF libraries for applications and services without adding SSAR-specific functionality to the PCF core.

This keeps PCF general-purpose while still allowing other projects to build on top of it.

## Design Principles

PCF follows a few simple principles:

- **Simple** — the format should be easy to read, understand, and implement.
- **Compact** — avoid unnecessary overhead when representing data.
- **Structured** — provide a consistent and predictable data model.
- **Extensible** — allow additional functionality without changing the core unnecessarily.
- **Portable** — work across platforms and programming languages.
- **Modular** — keep the core, libraries, and integrations separated.

## Repository

This repository contains the main PCF workspace and the components used to build the project.

This includes:

- PCF core processing and parsing
- File format and data structure definitions
- Serialization and deserialization
- Standard Libraries
- Ecosystem integrations
- Documentation and technical specifications
- Development and testing tools

The project is organized as a Rust workspace, with the different components maintained as separate crates.

SSAR-specific functionality is intentionally kept outside the base PCF implementation. When needed, it can be provided through separate libraries and integrations.

## Contributing

PCF is still being developed, and contributions are welcome.

You can contribute by:

- Reporting bugs
- Suggesting features or improvements
- Opening pull requests
- Improving the documentation
- Discussing or improving the PCF specification
- Building Standard Libraries
- Creating ecosystem integrations
- Testing PCF in different applications and backend architectures

For bugs and feature requests, use [GitHub Issues](https://github.com/ssar-group/pcf/issues).

This project follows the [SSAR Open Source Code of Conduct](https://docs.ssar-group.com/opensource/code-of-conduct?ver=3).

For questions or other inquiries, contact [contactus@ssar-group.com](mailto:contactus@ssar-group.com).

## License

PCF is released under the MIT License. See [`LICENSE`](./LICENSE) for the full license text.

The MIT License applies only to the open-source contents of this repository. Proprietary software, technologies, assets, trademarks, and other components owned by SSAR Group or its partners are not covered unless explicitly stated otherwise.

---

_Copyright © 2026 SSAR Group._
