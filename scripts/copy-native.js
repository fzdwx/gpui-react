#!/usr/bin/env node
// Copy compiled Rust library to src/native/{platform} for npm

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import os from "node:os";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Platform detection
function getPlatformInfo() {
    const platform = os.platform();
    const arch = os.arch();

    // Map Node.js platform names to our naming convention
    const platformMap = {
        linux: {
            x64: {
                name: "linux-x64",
                lib: "libgpui_renderer.so",
                target: "x86_64-unknown-linux-gnu",
            },
        },
        darwin: {
            x64: {
                name: "darwin-x64",
                lib: "libgpui_renderer.dylib",
                target: "x86_64-apple-darwin",
            },
            arm64: {
                name: "darwin-arm64",
                lib: "libgpui_renderer.dylib",
                target: "aarch64-apple-darwin",
            },
        },
        win32: {
            x64: { name: "win32-x64", lib: "gpui_renderer.dll", target: "x86_64-pc-windows-msvc" },
        },
    };

    const platformInfo = platformMap[platform]?.[arch];
    if (!platformInfo) {
        throw new Error(`Unsupported platform: ${platform}-${arch}`);
    }

    return platformInfo;
}

function main() {
    const platformInfo = getPlatformInfo();
    const { name: platformName, lib: libName, target: rustTarget } = platformInfo;

    console.log(`[copy-native] Platform: ${platformName}`);

    const RUST_DIR = path.join(__dirname, "..", "rust");
    const NATIVE_DIR = path.join(__dirname, "..", "src", "native", platformName);

    // Try to find the built library in different locations
    const possiblePaths = [
        // Specific target build
        path.join(RUST_DIR, "target", rustTarget, "release", libName),
        // Default target build
        path.join(RUST_DIR, "target", "release", libName),
    ];

    let sourcePath = null;
    for (const p of possiblePaths) {
        if (fs.existsSync(p)) {
            sourcePath = p;
            break;
        }
    }

    if (!sourcePath) {
        console.error(`[copy-native] Error: ${libName} not found in any of these locations:`);
        possiblePaths.forEach((p) => console.error(`         - ${p}`));
        console.error('       Run "cargo build --release" first.');
        process.exit(1);
    }

    if (!fs.existsSync(NATIVE_DIR)) {
        fs.mkdirSync(NATIVE_DIR, { recursive: true });
    }

    const destPath = path.join(NATIVE_DIR, libName);
    fs.copyFileSync(sourcePath, destPath);

    const size = fs.statSync(destPath).size;
    console.log(`[copy-native] ✓ ${libName} → ${destPath} (${(size / 1024 / 1024).toFixed(2)} MB)`);
}

main();
