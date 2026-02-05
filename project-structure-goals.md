```bash
Lifelog/
├── Cargo.toml               # Workspace manifest for all member crates
├── README.md                # High‑level project overview and instructions
├── CHANGELOG.md             # Record of major changes and version history
├── CONTRIBUTING.md          # Guidelines for contributions
├── docs/                    # In‑depth documentation (design docs, API specs, guides)
│   ├── architecture.md      # Overall system architecture
│   ├── usage.md             # How to run/build/deploy the project
│   └── faq.md               # Frequently asked questions and troubleshooting
├── ci/                      # CI/CD configuration files (GitHub Actions, Travis CI, etc.)
│   ├── build.yml            # Example GitHub Actions workflow for building/testing
│   └── lint.yml             # Linting and formatting checks
├── config/                  # Shared configuration files (e.g., logging, environment settings)
│   ├── default.toml         # Default global configuration values
│   └── secrets.sample.toml  # Template for secret configuration values (not checked into VCS)
├── lifelog-logger/          # Logger binary: Captures various data modalities
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # Application entry point
│   │   └── lib.rs           # Optional: internal library for logging functionality
│   ├── tests/               # Unit and integration tests
│   ├── examples/            # Usage examples for external developers
│   └── README.md            # Module-specific documentation
├── lifelog-server/          # Server binary: Processes, stores, and serves logged data
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # Main server launcher
│   │   └── routes.rs        # Example: HTTP route definitions if using a web framework
│   ├── tests/               # Testing server endpoints and functionality
│   ├── examples/            # Example API usage or integration examples
│   └── README.md
├── lifelog-interface/       # Interface binary: Provides the user‑facing UI (CLI, GUI, etc.)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # Entry point for the interface application
│   │   └── ui.rs            # Code for rendering the user interface
│   ├── tests/               # UI logic tests (e.g., unit tests for CLI parsing)
│   ├── examples/            # Demos and usage examples
│   └── README.md
└── common/                  # Shared libraries for reusability and consistency
    ├── data-model/          # Central definitions for shared data types and schemas
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs       # Domain models, serialization (using serde), etc.
    ├── utils/               # Utility functions, error handling, logging helpers, etc.
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    └── config/              # Shared configuration parsing and management
        ├── Cargo.toml
        └── src/
            └── lib.rs       # Common config structs and loaders (e.g., using config or envy)
```
