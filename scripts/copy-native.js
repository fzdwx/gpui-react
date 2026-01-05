#!/usr/bin/env node
// Copy compiled Rust library to src/native/linux-x64 for npm

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const NATIVE_DIR = path.join(__dirname, '..', 'src', 'native', 'linux-x64');
const SOURCE_PATH = path.join(__dirname, '..', 'rust', 'target', 'release', 'libgpui_renderer.so');

function main() {
  if (!fs.existsSync(SOURCE_PATH)) {
    console.error('[copy-native] Error: libgpui_renderer.so not found.');
    console.error('       Run "cargo build --release" first.');
    process.exit(1);
  }

  if (!fs.existsSync(NATIVE_DIR)) {
    fs.mkdirSync(NATIVE_DIR, { recursive: true });
  }

  const destPath = path.join(NATIVE_DIR, 'libgpui_renderer.so');
  fs.copyFileSync(SOURCE_PATH, destPath);

  const size = fs.statSync(destPath).size;
  console.log(`[copy-native] → ${destPath} (${(size / 1024 / 1024).toFixed(2)} MB)`);
}

main();