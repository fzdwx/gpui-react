// postinstall.js - Setup native binaries for gpui-react
// This script runs automatically after npm/bun install

const fs = require('fs');
const path = require('path');

const platformMap = {
  'darwin': { x64: 'darwin-x64', arm64: 'darwin-arm64' },
  'linux': { x64: 'linux-x64', arm64: 'linux-arm64' },
  'win32': { x64: 'win32-x64', arm64: 'win32-arm64' }
};

function getPlatform() {
  const os = process.platform;
  const arch = process.arch === 'x64' ? 'x64' : (process.arch === 'arm64' ? 'arm64' : 'x64');
  return { os, arch };
}

function findNativeBinary() {
  const { os, arch } = getPlatform();
  const platformKey = platformMap[os]?.[arch];
  
  if (!platformKey) {
    console.warn(`[gpui-react] Unsupported platform: ${os}-${arch}`);
    return null;
  }

  // Check in optionalDependencies first
  const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8'));
  const optionalDep = `@gpui-react/core-${platformKey}`;
  
  if (packageJson.optionalDependencies?.[optionalDep]) {
    const nodeModulesPath = path.join(__dirname, '..', 'node_modules', optionalDep);
    if (fs.existsSync(nodeModulesPath)) {
      const files = fs.readdirSync(nodeModulesPath);
      const binary = files.find(f => f.endsWith('.dylib') || f.endsWith('.so') || f.endsWith('.dll'));
      if (binary) {
        return path.join(nodeModulesPath, binary);
      }
    }
  }

  // Fallback: check native directory
  const nativeDir = path.join(__dirname, '..', 'native', platformKey);
  if (fs.existsSync(nativeDir)) {
    const files = fs.readdirSync(nativeDir);
    return files.map(f => path.join(nativeDir, f)).find(f => fs.statSync(f).isFile());
  }

  return null;
}

function setupNativeBinary() {
  const binaryPath = findNativeBinary();
  
  if (binaryPath) {
    const destDir = path.join(__dirname, '..', 'native');
    if (!fs.existsSync(destDir)) {
      fs.mkdirSync(destDir, { recursive: true });
    }
    
    const destPath = path.join(destDir, path.basename(binaryPath));
    fs.copyFileSync(binaryPath, destPath);
    console.log(`[gpui-react] Native binary copied to: ${destPath}`);
  } else {
    console.log('[gpui-react] No pre-built binary found. Using local build.');
  }
}

// Run on install (not during dev)
if (process.env.NODE_ENV !== 'development') {
  setupNativeBinary();
}