#!/usr/bin/env node

const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

// Get platform-specific binary information
function getBinaryInfo() {
  const platform = os.platform();
  const arch = os.arch();
  
  let binaryName;
  let downloadName;
  
  switch (platform) {
    case 'win32':
      binaryName = 'rip.exe';
      downloadName = 'rip-windows-x64.exe';
      break;
    case 'darwin':
      if (arch === 'arm64') {
        binaryName = 'rip-macos-arm64';
        downloadName = 'rip-macos-arm64';
      } else {
        binaryName = 'rip-macos-x64';
        downloadName = 'rip-macos-x64';
      }
      break;
    case 'linux':
      if (arch === 'arm64') {
        binaryName = 'rip-linux-arm64';
        downloadName = 'rip-linux-arm64';
      } else {
        binaryName = 'rip-linux-x64';
        downloadName = 'rip-linux-x64';
      }
      break;
    default:
      throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
  
  return { binaryName, downloadName };
}

// Get the latest release information from GitHub
async function getLatestRelease() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'api.github.com',
      path: '/repos/saintpetejackboy/rip/releases/latest',
      headers: {
        'User-Agent': 'rip-npm-installer'
      }
    };
    
    https.get(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        try {
          const release = JSON.parse(data);
          resolve(release);
        } catch (error) {
          reject(new Error(`Failed to parse release data: ${error.message}`));
        }
      });
    }).on('error', (error) => {
      reject(new Error(`Failed to fetch release info: ${error.message}`));
    });
  });
}

// Download binary from GitHub releases
async function downloadBinary(downloadUrl, outputPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(outputPath);
    
    const request = downloadUrl.startsWith('https') ? https : http;
    
    request.get(downloadUrl, (response) => {
      // Handle redirects
      if (response.statusCode === 302 || response.statusCode === 301) {
        return downloadBinary(response.headers.location, outputPath)
          .then(resolve)
          .catch(reject);
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download binary: HTTP ${response.statusCode}`));
        return;
      }
      
      response.pipe(file);
      
      file.on('finish', () => {
        file.close();
        resolve();
      });
      
      file.on('error', (error) => {
        fs.unlink(outputPath, () => {}); // Clean up on error
        reject(error);
      });
    }).on('error', (error) => {
      reject(new Error(`Download failed: ${error.message}`));
    });
  });
}

// Main installation function
async function install() {
  try {
    console.log('🔧 Installing RIP vulnerability scanner...');
    
    const { binaryName, downloadName } = getBinaryInfo();
    const binDir = path.join(__dirname, 'bin');
    const binaryPath = path.join(binDir, binaryName);
    
    // Create bin directory
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }
    
    // Check if binary already exists
    if (fs.existsSync(binaryPath)) {
      console.log('✅ Binary already exists, skipping download');
      return;
    }
    
    console.log(`📦 Downloading ${downloadName} for ${os.platform()}-${os.arch()}...`);
    
    // Get latest release
    const release = await getLatestRelease();
    
    // Find the correct asset
    const asset = release.assets.find(asset => asset.name === downloadName);
    
    if (!asset) {
      throw new Error(`Binary not found for platform ${os.platform()}-${os.arch()}`);
    }
    
    // Download the binary
    await downloadBinary(asset.browser_download_url, binaryPath);
    
    // Make binary executable on Unix systems
    if (os.platform() !== 'win32') {
      try {
        execSync(`chmod +x "${binaryPath}"`, { stdio: 'ignore' });
      } catch (error) {
        console.warn('⚠️  Warning: Could not set executable permissions');
      }
    }
    
    console.log('✅ RIP vulnerability scanner installed successfully!');
    console.log('🚀 Run with: npx rip');
    
  } catch (error) {
    console.error('❌ Installation failed:', error.message);
    console.error('');
    console.error('💡 You can try:');
    console.error('   1. Building from source: git clone https://github.com/saintpetejackboy/rip && cd rip && cargo build --release');
    console.error('   2. Downloading manually from: https://github.com/saintpetejackboy/rip/releases');
    console.error('   3. Reporting this issue: https://github.com/saintpetejackboy/rip/issues');
    
    process.exit(1);
  }
}

// Only run if this script is executed directly
if (require.main === module) {
  install();
}

module.exports = { install };