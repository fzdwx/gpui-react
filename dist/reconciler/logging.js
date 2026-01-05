const LOG_LEVELS = ['error', 'warn', 'info', 'debug', 'trace'];
function getLogLevel() {
    const envLevel = process.env.RUST_LOG?.toLowerCase();
    if (envLevel && LOG_LEVELS.includes(envLevel)) {
        return envLevel;
    }
    return 'info';
}
function getLocalTime() {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    const hours = String(now.getHours()).padStart(2, '0');
    const minutes = String(now.getMinutes()).padStart(2, '0');
    const seconds = String(now.getSeconds()).padStart(2, '0');
    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}
function shouldLog(level) {
    const currentLevel = getLogLevel();
    return LOG_LEVELS.indexOf(level) <= LOG_LEVELS.indexOf(currentLevel);
}
function formatMessage(level, message, args) {
    const timestamp = getLocalTime();
    const argsStr = args && args.length > 0 ? '\n' + args.map(a => JSON.stringify(a, null, 2)).join('\n') : '';
    return `${timestamp} ${level.toUpperCase()} ${message}${argsStr}`;
}
export function initLogging() {
    const level = getLogLevel();
    console.log(`${getLocalTime()} INFO  init: Logging system initialized (level: ${level})`);
}
export function log(level, message, ...args) {
    if (!shouldLog(level))
        return;
    const formatted = formatMessage(level, message, args);
    switch (level) {
        case 'error':
            console.error(formatted);
            break;
        case 'warn':
            console.warn(formatted);
            break;
        case 'info':
            console.log(formatted);
            break;
        case 'debug':
            console.debug(formatted);
            break;
        case 'trace':
            console.trace(formatted);
            break;
    }
}
export function trace(message, ...args) {
    log('trace', message, ...args);
}
export function debug(message, ...args) {
    log('debug', message, ...args);
}
export function info(message, ...args) {
    log('info', message, ...args);
}
export function warn(message, ...args) {
    log('warn', message, ...args);
}
export function error(message, ...args) {
    log('error', message, ...args);
}
