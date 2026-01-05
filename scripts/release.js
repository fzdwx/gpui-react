#!/usr/bin/env node
// Version bump and tag management for gpui-react
// Usage: node scripts/release.js [patch|minor|major] [--dry-run]

import { execSync } from 'child_process';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import readline from 'readline';

const __dirname = dirname(fileURLToPath(import.meta.url));

const packagePath = join(__dirname, '..', 'package.json');
const cargoPath = join(__dirname, '..', 'rust', 'Cargo.toml');
const workspacePath = join(__dirname, '..');

function getCurrentVersion() {
  const pkg = JSON.parse(readFileSync(packagePath, 'utf8'));
  return pkg.version;
}

function bumpVersion(version, type) {
  const [major, minor, patch] = version.split('.').map(Number);
  
  switch (type) {
    case 'major': return `${major + 1}.0.0`;
    case 'minor': return `${major}.${minor + 1}.0`;
    case 'patch': return `${major}.${minor}.${patch + 1}`;
    default: throw new Error(`Invalid bump type: ${type}. Use patch, minor, or major.`);
  }
}

function updateVersionInFile(filePath, newVersion) {
  const content = readFileSync(filePath, 'utf8');
  const ext = filePath.split('.').pop();
  
  if (ext === 'json') {
    const pkg = JSON.parse(content);
    pkg.version = newVersion;
    writeFileSync(filePath, JSON.stringify(pkg, null, 2) + '\n');
  } else if (ext === 'toml') {
    const lines = content.split('\n');
    const updated = lines.map(line => {
      if (line.startsWith('version = ')) {
        return `version = "${newVersion}"`;
      }
      return line;
    });
    writeFileSync(filePath, updated.join('\n'));
  }
}

function updateAllPlatformPackages(newVersion) {
  const platforms = [
    'core-darwin-x64', 'core-darwin-arm64',
    'core-linux-x64', 'core-linux-arm64',
    'core-win32-x64', 'core-win32-arm64'
  ];
  
  for (const platform of platforms) {
    const pkgPath = join(__dirname, '..', 'packages', platform, 'package.json');
    if (existsSync(pkgPath)) {
      updateVersionInFile(pkgPath, newVersion);
      console.log(`  Updated ${platform}`);
    }
  }
}

function runGit(command, dryRun) {
  console.log(`  Git: ${command}`);
  if (!dryRun) execSync(command, { cwd: workspacePath, stdio: 'inherit' });
}

async function confirm(message) {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  const answer = await new Promise(resolve => rl.question(message, resolve));
  rl.close();
  return answer.toLowerCase() === 'y';
}

async function createRelease() {
  const args = process.argv.slice(2);
  const bumpType = args[0] || 'patch';
  const isDryRun = args.includes('--dry-run') || args.includes('-n');
  
  console.log('\n🚀 gpui-react Release Script');
  console.log(`   Mode: ${isDryRun ? 'DRY RUN' : 'LIVE'}\n`);
  
  const currentVersion = getCurrentVersion();
  console.log(`Current version: ${currentVersion}`);
  
  const newVersion = bumpVersion(currentVersion, bumpType);
  console.log(`New version: ${newVersion}`);
  console.log(`Bump type: ${bumpType}\n`);
  
  if (isDryRun) {
    console.log('⚠️  DRY RUN - No changes will be made\n');
    return;
  }
  
  console.log('This will:');
  console.log('  1. Update version in package.json, Cargo.toml, and platform packages');
  console.log('  2. Commit changes');
  console.log(`  3. Create and push tag v${newVersion}`);
  console.log('\n');
  
  if (!await confirm('Proceed? (y/n): ')) {
    console.log('Cancelled.');
    process.exit(0);
  }
  
  updateVersionInFile(packagePath, newVersion);
  updateVersionInFile(cargoPath, newVersion);
  updateAllPlatformPackages(newVersion);
  console.log(`\n✅ Updated to ${newVersion}`);
  
  runGit('git add package.json rust/Cargo.toml packages/*/package.json', isDryRun);
  runGit(`git commit -m "Release v${newVersion}"`, isDryRun);
  runGit(`git tag -a v${newVersion} -m "Release v${newVersion}"`, isDryRun);
  runGit('git push origin main && git push origin v' + newVersion, isDryRun);
  
  console.log(`\n✅ Release v${newVersion} complete!`);
  console.log(`   Create GitHub Release at: https://github.com/fzdwx/gpui-react/releases/new?tag=v${newVersion}\n`);
}

function showStatus() {
  const version = getCurrentVersion();
  const branch = execSync('git branch --show-current', { cwd: workspacePath }).toString().trim();
  const lastTag = execSync('git describe --tags --abbrev=0 2>/dev/null || echo "No tags"', { cwd: workspacePath }).toString().trim();
  
  console.log(`\n📦 gpui-react Status`);
  console.log(`   Version: ${version}`);
  console.log(`   Branch: ${branch}`);
  console.log(`   Latest tag: ${lastTag}\n`);
  
  const status = execSync('git status --porcelain', { cwd: workspacePath }).toString().trim();
  if (status) {
    console.log('⚠️  Uncommitted changes:');
    console.log(status);
  } else {
    console.log('✅ Working tree clean');
  }
  
  try {
    const remoteTracking = execSync('git rev-parse --abbrev-ref @{u}', { cwd: workspacePath, encoding: 'utf8' }).toString().trim();
    const localSha = execSync('git rev-parse HEAD', { cwd: workspacePath }).toString().trim();
    const remoteSha = execSync(`git rev-parse ${remoteTracking}`, { cwd: workspacePath, encoding: 'utf8' }).toString().trim();
    
    if (localSha !== remoteSha) {
      console.log(`⚠️  Local (${localSha.substring(0, 7)}) differs from remote (${remoteSha.substring(0, 7)})`);
    } else {
      console.log('✅ Local and remote synchronized');
    }
  } catch {
    console.log('⚠️  No remote tracking branch');
  }
  
  console.log('');
}

const command = process.argv[2];

if (command === '--status' || command === '-s') {
  showStatus();
} else {
  createRelease().catch(err => {
    console.error('Error:', err.message);
    process.exit(1);
  });
}