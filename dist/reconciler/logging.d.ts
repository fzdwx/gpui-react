type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'trace';
export declare function initLogging(): void;
export declare function log(level: LogLevel, message: string, ...args: unknown[]): void;
export declare function trace(message: string, ...args: unknown[]): void;
export declare function debug(message: string, ...args: unknown[]): void;
export declare function info(message: string, ...args: unknown[]): void;
export declare function warn(message: string, ...args: unknown[]): void;
export declare function error(message: string, ...args: unknown[]): void;
export {};
//# sourceMappingURL=logging.d.ts.map