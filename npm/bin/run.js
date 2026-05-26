#!/usr/bin/env node
'use strict';

// Cross-platform launcher for the Rust-built figma-mcp-rust binary.

const { spawnSync } = require('node:child_process');
const { existsSync } = require('node:fs');
const path = require('node:path');

const platform = process.platform; // darwin | linux | win32
const arch = process.arch;         // x64 | arm64

const SUPPORTED = new Set([
  'darwin-x64',
  'darwin-arm64',
  'linux-x64',
  'linux-arm64',
  'win32-x64',
  'win32-arm64',
]);
const target = `${platform}-${arch}`;

if (!SUPPORTED.has(target)) {
  process.stderr.write(`[figma-mcp-rust] Unsupported platform: ${target}\n`);
  process.exit(1);
}

const binaryName = platform === 'win32' ? 'figma-mcp-rust.exe' : 'figma-mcp-rust';
const binaryPath = path.join(__dirname, target, binaryName);

if (!existsSync(binaryPath)) {
  process.stderr.write(
    '[figma-mcp-rust] Binary not found. Try reinstalling: npm install @alvinindra/figma-mcp-rust\n'
  );
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
process.exit(result.status ?? 1);
