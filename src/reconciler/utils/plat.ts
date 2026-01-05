import { join } from "path";
import { suffix } from "bun:ffi";

/**
 * Detect platform and architecture for native library path
 * Returns the directory name for the native library based on current platform
 */
export function getNativePlatformDir(): string {
    const platform = process.platform;
    const arch = process.arch;

    switch (platform) {
        case "linux":
            return arch === "arm64" ? "linux-arm64" : "linux-x64";
        case "darwin":
            return arch === "arm64" ? "macos-arm64" : "macos-x64";
        case "win32":
            return "windows-x64";
        default:
            throw new Error(`Unsupported platform: ${platform} ${arch}`);
    }
}

/**
 * Get the full path to the native library
 * @param baseDir - Base directory (usually import.meta.dir)
 * @returns Full path to the native library
 */
export function getNativeLibPath(baseDir: string): string {
    const libName = `libgpui_renderer.${suffix}`;
    const platformDir = getNativePlatformDir();
    return join(baseDir, "../../src/native", platformDir, libName);
}