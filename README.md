# Processing Compact File

![experimental](https://img.shields.io/badge/status-experimental-blueviolet.svg?colorA=303033&colorB=a08a2c)
![Version](https://img.shields.io/badge/version-E.0.1-blueviolet.svg?colorA=303033&colorB=6315ac)
![Platform](https://img.shields.io/badge/platform-cross--platform-000000.svg?colorA=303033&colorB=fff)

**PCF (Processing Compact File)** is a compact, extensible system for representing, processing, and exchanging structured data. It is language, framework, and platform-agnostic, and is designed to sit underneath applications, backend services, APIs, developer tools, configuration systems, and caches alike.

## Architecture

PCF is organized in four layers, each depending only on the one below it:

```text
┌──────────────────────────────────────┐
│          Applications / Services     │
├──────────────────────────────────────┤
│           Ecosystem Libraries        │
│              Integrations            │
├──────────────────────────────────────┤
│         PCF Standard Libraries       │
├──────────────────────────────────────┤
│               PCF Core               │
└──────────────────────────────────────┘
```

**PCF Core** — parsing, processing, and the file format / data structure definitions. No external or ecosystem dependencies.

**Standard Libraries** — common functionality built on top of the core: data structures and utilities, file and data processing, serialization and formatting, networking, and other backend primitives. Versioned and evolved independently of the core, while staying compatible with the underlying specification.

**Ecosystem Libraries** — integrations for specific environments. The SSAR ecosystem, for instance, maintains its own PCF libraries for SSAR applications and services. These stay fully separate from the core and Standard Libraries, so PCF can support both general-purpose projects and specialized ecosystems without coupling the two.

## Design Principles

- **Simple** — an easy-to-understand format and processing model.
- **Compact** — minimal overhead.
- **Structured** — a consistent, predictable data model.
- **Extensible** — libraries and ecosystems build on top without touching the core.
- **Portable** — no dependency on a specific platform or technology.
- **Modular** — core, standard libraries, and ecosystem integrations stay strictly separated.

## What's in this repository

- PCF core processing and parsing logic
- File format and data structure definitions
- Serialization / deserialization
- Standard Libraries
- Ecosystem-specific libraries and integrations
- Documentation and technical specifications
- Development and testing resources

## Contributing

- Report bugs and request features via [GitHub Issues](https://github.com/ssar-group)
- Submit changes through pull requests
- Improve the PCF specification and documentation
- Build Standard Libraries or ecosystem integrations
- Test PCF against different applications and backend architectures, and report compatibility issues

This project follows the [SSAR Open Source Code of Conduct](https://docs.ssar-group.com/opensource/code-of-conduct?ver=3). Questions: [contactus@ssar-group.com](mailto:contactus@ssar-group.com).

## License

PCF is released under the MIT License — see [`LICENSE`](./LICENSE) for the full text.

The MIT License covers the open-source contents of this repository only. Proprietary software, technologies, assets, trademarks, and components owned by SSAR Group or its partners are not covered unless explicitly stated otherwise.

---

_Copyright © 2026 SSAR Group._
