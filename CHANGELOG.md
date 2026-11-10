# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-11-10

### Fixed

- **server:** Honor dedupe_components in get_design_context ([25668c9](https://github.com/alvinindra/figma-mcp-rust/commit/25668c92fecfbb63111f54738453410cc9a6372c))

### Documentation

- **readme:** Add antigravity setup ([45f3d8d](https://github.com/alvinindra/figma-mcp-rust/commit/45f3d8d798c84ca3200ea57c44739733a9369a6a))
- Add changelog ([1f31230](https://github.com/alvinindra/figma-mcp-rust/commit/1f31230d77d5508db30aeacdeff60a3bd7928e81))

### Build & CI

- **release:** Retry MCP Registry publish without re-running npm ([1b10632](https://github.com/alvinindra/figma-mcp-rust/commit/1b10632640fe1430226915e3bb167679f27e5ad2))
- **release:** Build release notes from conventional commits ([7bb4392](https://github.com/alvinindra/figma-mcp-rust/commit/7bb439244b0d90f4848d779ae89b8bbf0b64d231))

**Full diff**: [v0.2.1...v0.2.2](https://github.com/alvinindra/figma-mcp-rust/compare/v0.2.1...v0.2.2)

## [0.2.1] - 2026-10-12

### Fixed

- **server:** Upgrade rmcp to 3.5.1 for MCP 2026-07-28 clients ([8876297](https://github.com/alvinindra/figma-mcp-rust/commit/8876297b8cc08fc83b7f02c77f3cb11455d4e3ce))

**Full diff**: [v0.2.0...v0.2.1](https://github.com/alvinindra/figma-mcp-rust/compare/v0.2.0...v0.2.1)

## [0.2.0] - 2026-08-19

### Performance

- **plugin:** Stop re-serializing subtrees and memoize style lookups ([29d0a2d](https://github.com/alvinindra/figma-mcp-rust/commit/29d0a2d73373ff92f0d554da26868c866526a13b))
- **server:** Overlap save_screenshots round-trips ([2460c9a](https://github.com/alvinindra/figma-mcp-rust/commit/2460c9a0f6b72b566e9df1b252407fa790a908de))

### Dependencies

- Upgrade rmcp to 3.1.3 ([21bcbe9](https://github.com/alvinindra/figma-mcp-rust/commit/21bcbe92753a8705aa21e4c2794c8bcda10a650d))

**Full diff**: [v0.1.7...v0.2.0](https://github.com/alvinindra/figma-mcp-rust/compare/v0.1.7...v0.2.0)

## [0.1.7] - 2026-08-04

### Added

- **plugin:** Add Claude Code skills for the bundled workflows ([dc22f1d](https://github.com/alvinindra/figma-mcp-rust/commit/dc22f1df9847508ee475aa04cf735a62cdfdb8e8))
- **server:** Explain bridge requirement in MCP server instructions ([541f61a](https://github.com/alvinindra/figma-mcp-rust/commit/541f61a2bd045c4e3cfc57af369f8b2da284ec53))

### Fixed

- **plugin:** Sync version metadata across manifests and guard in CI ([969826e](https://github.com/alvinindra/figma-mcp-rust/commit/969826e1d8309ce7ac1cab26d5aae243cd688cc9))

### Documentation

- Tidy punctuation and complete the prompts table ([0f04468](https://github.com/alvinindra/figma-mcp-rust/commit/0f04468509786990697524c5aadb542eb092a28c))

**Full diff**: [v0.1.6...v0.1.7](https://github.com/alvinindra/figma-mcp-rust/compare/v0.1.6...v0.1.7)

## [0.1.6] - 2026-08-04

### Added

- **plugin:** Add Claude Code plugin marketplace support ([9c0789a](https://github.com/alvinindra/figma-mcp-rust/commit/9c0789a46f988293b749754a506cd347ac3ad21a))

### Fixed

- **transport:** Survive malformed JSON-RPC frames on stdin ([7861ee7](https://github.com/alvinindra/figma-mcp-rust/commit/7861ee7dda2138e463d9ac3949ce5f5b7078c1a5))

**Full diff**: [v0.1.5...v0.1.6](https://github.com/alvinindra/figma-mcp-rust/compare/v0.1.5...v0.1.6)

## [0.1.5] - 2026-07-14

### Fixed

- Edge-case bugs in bridge, follower, handler, and file tools ([97fc80b](https://github.com/alvinindra/figma-mcp-rust/commit/97fc80b3bd18b4613ffc2ada9584056da46ef080))

### Documentation

- Fix Star History embed to use official svg endpoint ([19325e9](https://github.com/alvinindra/figma-mcp-rust/commit/19325e9dfbf9935eb876685dce0f3ba90d54c48d))

**Full diff**: [v0.1.4...v0.1.5](https://github.com/alvinindra/figma-mcp-rust/compare/v0.1.4...v0.1.5)

## [0.1.4] - 2026-05-26

### Added

- **effects:** Support NoiseEffect read + write (set_effects + serializers) ([5453dcf](https://github.com/alvinindra/figma-mcp-rust/commit/5453dcfa69f744b4b27d93069ebedb7af5339bb9))

**Full diff**: [v0.1.3...v0.1.4](https://github.com/alvinindra/figma-mcp-rust/compare/v0.1.3...v0.1.4)

## [0.1.3] - 2026-05-26

### Fixed

- **server.json:** Shorten description to <=100 chars for MCP registry ([ce46e97](https://github.com/alvinindra/figma-mcp-rust/commit/ce46e97d5533432cf378965556c96709afd70e31))

**Full diff**: [v0.1.2...v0.1.3](https://github.com/alvinindra/figma-mcp-rust/compare/v0.1.2...v0.1.3)

## [0.1.2] - 2026-05-26

### Build & CI

- Route darwin-x64 build to macos-14 cross-compile, bump 0.1.2 ([6b8619e](https://github.com/alvinindra/figma-mcp-rust/commit/6b8619e2c33157698e591c243637615398e5a7b4))

**Full diff**: [v0.1.1...v0.1.2](https://github.com/alvinindra/figma-mcp-rust/compare/v0.1.1...v0.1.2)

## [0.1.0] - 2026-05-26

### Added

- Port figma-mcp-go to Rust with 73 tools, race-free bridge, and declarative tool table ([68d836a](https://github.com/alvinindra/figma-mcp-rust/commit/68d836a7a2bbb02edc2931d6ea561ab36256c6b8))


