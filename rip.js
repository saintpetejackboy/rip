#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

// Determine binary name based on platform
function getBinaryName() {
  const platform = os.platform();
  const arch = os.arch();
  
  switch (platform) {
    case 'win32':
      return 'rip.exe';
    case 'darwin':
      return arch === 'arm64' ? 'rip-macos-arm64' : 'rip-macos-x64';
    case 'linux':
      return arch === 'arm64' ? 'rip-linux-arm64' : 'rip-linux-x64';
    default:
      return 'rip';
  }
}

// Path to the binary
const binaryPath = path.join(__dirname, 'bin', getBinaryName());

// Spawn the Rust binary with all arguments
const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false
});

child.on('error', (error) => {
  console.error('Failed to start RIP:', error.message);
  process.exit(1);
});

child.on('exit', (code) => {
  process.exit(code || 0);
});