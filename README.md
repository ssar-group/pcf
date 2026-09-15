# Processing Compact File

![experimental](https://img.shields.io/badge/status-experimental-blueviolet.svg?colorA=303033&colorB=a08a2c)
![Version](https://img.shields.io/badge/version-E.0.1-blueviolet.svg?colorA=303033&colorB=6315ac)
![Platform](https://img.shields.io/badge/platform-cross--platform-000000.svg?colorA=303033&colorB=fff)

**PCF (Processing Compact File)** is a compact and extensible format and processing system for working with structured data.

The goal is simple: provide a common foundation for applications, backend services, APIs, developer tools, configuration files, caches, and other systems that need to store or exchange structured data.

PCF is designed to remain independent from any particular programming language, framework, or platform. It can be used on its own, or as a foundation for larger ecosystems.

> [!NOTE]
> PCF is currently experimental and under active development. APIs, the specification, and the internal architecture may change as the project evolves.

## Architecture

PCF is built in layers. Each layer builds on the one below it, while keeping its own responsibilities and dependencies clearly separated.

```text
┌──────────────────────────────────────┐
│          Applications / Services     │
├──────────────────────────────────────┤
│        Ecosystem Libraries           │
│             Integrations             │
├──────────────────────────────────────┤
│         PCF Standard Libraries       │
├──────────────────────────────────────┤
│              PCF Core                │
└──────────────────────────────────────┘
```

### PCF Core

The core is the foundation of PCF.

It contains the parser, processing logic, file format definitions, and the fundamental data structures used throughout the project.

The core is intentionally kept small and independent. It should not depend on a specific ecosystem or application.

### Standard Libraries

Standard Libraries provide functionality that is useful for most PCF-based projects.

This includes things such as data structures, utilities, file and data processing, serialization, formatting, networking, and other common backend functionality.

They are developed separately from the core so they can evolve without unnecessarily changing the underlying PCF format.

### Ecosystem Libraries

Ecosystem Libraries provide integrations for specific environments or projects.

For example, the SSAR ecosystem can provide its own PCF libraries for SSAR applications and services without adding SSAR-specific functionality to PCF itself.

Keeping these libraries separate allows PCF to remain useful as a general-purpose project while still being extensible for more specialized ecosystems.

## Design Principles

PCF is built around a few straightforward principles:

- **Simple** — the format should be easy to understand and work with.
- **Compact** — avoid unnecessary overhead when representing data.
- **Structured** — provide a consistent and predictable data model.
- **Extensible** — allow libraries and ecosystems to add functionality without modifying the core.
- **Portable** — PCF should not depend on a specific platform or programming language.
- **Modular** — keep the core, standard libraries, and ecosystem integrations clearly separated.

## Repository

This repository contains the foundation of the PCF ecosystem, including:

- PCF core processing and parsing
- File format and data structure definitions
- Serialization and deserialization
- Standard Libraries
- Ecosystem libraries and integrations
- Documentation and technical specifications
- Development and testing tools

The project is organized as a Rust workspace, with the different components maintained as separate crates.

SSAR-specific functionality is intentionally kept outside of the base PCF workspace. It can be developed and maintained as separate libraries when needed.

## Contributing

PCF is still evolving, and contributions can help shape both the implementation and the specification.

You can contribute by:

- Reporting bugs or suggesting features
- Opening pull requests
- Improving the documentation
- Discussing or improving the PCF specification
- Building Standard Libraries
- Creating ecosystem integrations
- Testing PCF with different applications and backend architectures

For bugs and feature requests, use [GitHub Issues](https://github.com/ssar-group).

This project follows the [SSAR Open Source Code of Conduct](https://docs.ssar-group.com/opensource/code-of-conduct?ver=3).

For questions or other inquiries, contact [contactus@ssar-group.com](mailto:contactus@ssar-group.com).

## License

PCF is released under the MIT License. See [`LICENSE`](./LICENSE) for the full license text.

The MIT License applies only to the open-source contents of this repository. Proprietary software, technologies, assets, trademarks, and other components owned by SSAR Group or its partners are not covered unless explicitly stated otherwise.

---

_Copyright © 2026 SSAR Group._
