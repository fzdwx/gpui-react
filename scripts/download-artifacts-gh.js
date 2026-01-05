#!/usr/bin/env node
// Download native libraries from GitHub Actions artifacts using gh CLI

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { exec } from "node:child_process";
import { promisify } from "node:util";

const execAsync = promisify(exec);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const NATIVE_DIR = path.join(__dirname, "..", "src", "native");

async function checkGhCli() {
    try {
        await execAsync("gh --version");
        return true;
    } catch (error) {
        return false;
    }
}

async function getLatestWorkflowRun() {
    console.log("[download-artifacts] Fetching latest successful workflow run...");

    const { stdout } = await execAsync(
        "gh run list --workflow=build-native.yml --status=success --limit=1 --json databaseId,conclusion,headSha,displayTitle"
    );

    const runs = JSON.parse(stdout);
    if (!runs || runs.length === 0) {
        throw new Error("No successful workflow runs found");
    }

    return runs[0];
}

async function downloadArtifact(runId, artifactName, outputDir) {
    console.log(`[download-artifacts] Downloading ${artifactName}...`);

    const tempDir = path.join(__dirname, "..", ".tmp-artifacts");
    if (!fs.existsSync(tempDir)) {
        fs.mkdirSync(tempDir, { recursive: true });
    }

    try {
        // Download artifact using gh CLI
        await execAsync(
            `gh run download ${runId} --name ${artifactName} --dir "${tempDir}/${artifactName}"`
        );

        // Move files to the correct location
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }

        const files = fs.readdirSync(path.join(tempDir, artifactName));
        for (const file of files) {
            const src = path.join(tempDir, artifactName, file);
            const dest = path.join(outputDir, file);
            fs.copyFileSync(src, dest);

            const size = fs.statSync(dest).size;
            console.log(`[download-artifacts] ✓ ${file} (${(size / 1024 / 1024).toFixed(2)} MB)`);
        }
    } catch (error) {
        console.warn(
            `[download-artifacts] Warning: Failed to download ${artifactName}: ${error.message}`
        );
    }
}

async function main() {
    try {
        // Check if gh CLI is installed
        const hasGhCli = await checkGhCli();
        if (!hasGhCli) {
            console.error("[download-artifacts] Error: GitHub CLI (gh) is not installed.");
            console.error("       Install it from: https://cli.github.com/");
            process.exit(1);
        }

        console.log("[download-artifacts] Starting download process...");

        const workflowRun = await getLatestWorkflowRun();
        console.log(
            `[download-artifacts] Found run: ${workflowRun.displayTitle} (${workflowRun.headSha.substring(0, 7)})`
        );

        const platforms = [
            { name: "linux-x64", artifact: "native-linux-x64" },
            { name: "darwin-x64", artifact: "native-darwin-x64" },
            { name: "darwin-arm64", artifact: "native-darwin-arm64" },
            { name: "win32-x64", artifact: "native-win32-x64" },
        ];

        // Download each platform artifact
        for (const platform of platforms) {
            const outputDir = path.join(NATIVE_DIR, platform.name);
            await downloadArtifact(workflowRun.databaseId, platform.artifact, outputDir);
        }

        // Clean up temp directory
        const tempDir = path.join(__dirname, "..", ".tmp-artifacts");
        if (fs.existsSync(tempDir)) {
            fs.rmSync(tempDir, { recursive: true, force: true });
        }

        console.log("[download-artifacts] ✓ All artifacts downloaded successfully!");
    } catch (error) {
        console.error("[download-artifacts] Error:", error.message);
        process.exit(1);
    }
}

main();
