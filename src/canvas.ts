export interface CanvasClearCommand {
    type: "clear";
    color: string;
}

export interface CanvasFillRectCommand {
    type: "fillRect";
    x: number;
    y: number;
    width: number;
    height: number;
    color: string;
}

export interface CanvasCircleCommand {
    type: "circle";
    x: number;
    y: number;
    radius: number;
    color: string;
}

export interface CanvasLineCommand {
    type: "line";
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    width: number;
    color: string;
}

export interface CanvasTextCommand {
    type: "text";
    text: string;
    x: number;
    y: number;
    size: number;
    color: string;
}

export interface CanvasPathCommand {
    type: "path";
    points: [number, number][];
    width: number;
    color: string;
}

export type CanvasDrawCommand =
    | CanvasClearCommand
    | CanvasFillRectCommand
    | CanvasCircleCommand
    | CanvasLineCommand
    | CanvasTextCommand
    | CanvasPathCommand;

export interface CanvasProps {
    width: number;
    height: number;
    backgroundColor?: string;
    drawCommands?: CanvasDrawCommand[];
    style?: React.CSSProperties;
    onMouseDown?: (event: MouseEvent) => void;
    onMouseMove?: (event: MouseEvent) => void;
    onMouseUp?: (event: MouseEvent) => void;
    onClick?: (event: MouseEvent) => void;
}

export interface MouseEvent {
    clientX: number;
    clientY: number;
    button: number;
}

function isCanvasDrawCommand(cmd: any): cmd is CanvasDrawCommand {
    return cmd && typeof cmd.type === "string";
}

export function createDrawCommands(commands: CanvasDrawCommand[]): string {
    return JSON.stringify(commands);
}

export function clear(color: string): CanvasClearCommand {
    return { type: "clear", color };
}

export function fillRect(
    x: number,
    y: number,
    width: number,
    height: number,
    color: string
): CanvasFillRectCommand {
    return { type: "fillRect", x, y, width, height, color };
}

export function circle(x: number, y: number, radius: number, color: string): CanvasCircleCommand {
    return { type: "circle", x, y, radius, color };
}

export function line(
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    width: number,
    color: string
): CanvasLineCommand {
    return { type: "line", x1, y1, x2, y2, width, color };
}

export function text(
    content: string,
    x: number,
    y: number,
    size: number,
    color: string
): CanvasTextCommand {
    return { type: "text", text: content, x, y, size, color };
}

export function path(points: [number, number][], width: number, color: string): CanvasPathCommand {
    return { type: "path", points, width, color };
}
