type LogLevel = "error" | "warn" | "info" | "debug" | "trace";

const LOG_LEVELS: LogLevel[] = ["error", "warn", "info", "debug", "trace"];

function getLogLevel(): LogLevel {
    const envLevel = process.env.RUST_LOG?.toLowerCase() as LogLevel | undefined;
    if (envLevel && LOG_LEVELS.includes(envLevel)) {
        return envLevel;
    }
    return "info";
}

function getLocalTime(): string {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, "0");
    const day = String(now.getDate()).padStart(2, "0");
    const hours = String(now.getHours()).padStart(2, "0");
    const minutes = String(now.getMinutes()).padStart(2, "0");
    const seconds = String(now.getSeconds()).padStart(2, "0");
    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

function shouldLog(level: LogLevel): boolean {
    const currentLevel = getLogLevel();
    return LOG_LEVELS.indexOf(level) <= LOG_LEVELS.indexOf(currentLevel);
}

function formatMessage(level: LogLevel, message: string, args?: unknown[]): string {
    const timestamp = getLocalTime();
    const argsStr =
        args && args.length > 0
            ? "\n" + args.map((a) => JSON.stringify(a, null, 2)).join("\n")
            : "";
    return `${timestamp} ${level.toUpperCase()} ${message}${argsStr}`;
}

export function initLogging(): void {
    const level = getLogLevel();
    console.log(`${getLocalTime()} INFO  init: Logging system initialized (level: ${level})`);
}

export function log(level: LogLevel, message: string, ...args: unknown[]): void {
    if (!shouldLog(level)) return;
    const formatted = formatMessage(level, message, args);
    switch (level) {
        case "error":
            console.error(formatted);
            break;
        case "warn":
            console.warn(formatted);
            break;
        case "info":
            console.log(formatted);
            break;
        case "debug":
            console.debug(formatted);
            break;
        case "trace":
            console.trace(formatted);
            break;
    }
}

export function trace(message: string, ...args: unknown[]): void {
    log("trace", message, ...args);
}

export function debug(message: string, ...args: unknown[]): void {
    log("debug", message, ...args);
}

export function info(message: string, ...args: unknown[]): void {
    log("info", message, ...args);
}

export function warn(message: string, ...args: unknown[]): void {
    log("warn", message, ...args);
}

export function error(message: string, ...args: unknown[]): void {
    log("error", message, ...args);
}
