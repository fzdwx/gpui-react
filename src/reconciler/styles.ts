/**
 * Style Utilities for React-GPUI Renderer
 *
 * Handles conversion between React style props and GPUI style values
 */

import type { GPUIEventHandlerProps } from "../events";

export interface StyleProps extends GPUIEventHandlerProps {
    // Text properties (inheritable)
    color?: string;
    fontSize?: number | string;
    fontWeight?: string | number;
    fontFamily?: string;
    lineHeight?: number | string;
    textAlign?: "left" | "center" | "right";
    letterSpacing?: number | string;

    // Other inheritable properties
    cursor?: string;
    visibility?: "visible" | "hidden";

    // Non-inheritable properties
    backgroundColor?: string;
    width?: number | string;
    height?: number | string;

    // Size constraints
    minWidth?: number | string;
    maxWidth?: number | string;
    minHeight?: number | string;
    maxHeight?: number | string;
    aspectRatio?: number;

    // Margin (shorthand and individual)
    margin?: number | string;
    marginTop?: number | string;
    marginRight?: number | string;
    marginBottom?: number | string;
    marginLeft?: number | string;

    // Padding (shorthand and individual)
    padding?: number | string;
    paddingTop?: number | string;
    paddingRight?: number | string;
    paddingBottom?: number | string;
    paddingLeft?: number | string;

    // Position
    position?: "relative" | "absolute";
    top?: number | string;
    right?: number | string;
    bottom?: number | string;
    left?: number | string;
    inset?: number | string;

    // Overflow
    overflow?: "visible" | "hidden" | "scroll" | "clip";
    overflowX?: "visible" | "hidden" | "scroll" | "clip";
    overflowY?: "visible" | "hidden" | "scroll" | "clip";

    // Border (shorthand and individual)
    border?: string;
    borderWidth?: number | string;
    borderStyle?: "solid" | "dashed";
    borderColor?: string;
    borderTop?: string;
    borderRight?: string;
    borderBottom?: string;
    borderLeft?: string;
    borderTopWidth?: number | string;
    borderRightWidth?: number | string;
    borderBottomWidth?: number | string;
    borderLeftWidth?: number | string;
    borderTopColor?: string;
    borderRightColor?: string;
    borderBottomColor?: string;
    borderLeftColor?: string;
    borderRadius?: number | string;

    // Box Shadow
    boxShadow?: string;

    // Flexbox
    display?: "flex" | "block" | "inline" | "inline-block";
    flexDirection?: "row" | "column" | "row-reverse" | "column-reverse";
    flexWrap?: "nowrap" | "wrap" | "wrap-reverse";
    flexGrow?: number;
    flexShrink?: number;
    flexBasis?: number | string;
    justifyContent?:
        | "flex-start"
        | "center"
        | "flex-end"
        | "space-between"
        | "space-around"
        | "space-evenly";
    alignItems?: "flex-start" | "center" | "flex-end" | "stretch" | "baseline";
    alignSelf?: "auto" | "flex-start" | "center" | "flex-end" | "stretch" | "baseline";
    alignContent?:
        | "flex-start"
        | "center"
        | "flex-end"
        | "space-between"
        | "space-around"
        | "stretch";
    gap?: number | string;
    rowGap?: number | string;
    columnGap?: number | string;

    // Other
    opacity?: number;
    src?: string;
    alt?: string;

    // Focus properties
    tabIndex?: number; // -1 = programmatic focus only, 0+ = Tab navigation order

    // Hover styles (pseudo-class) - excludes event handlers
    _hover?: Omit<StyleProps, "_hover" | keyof GPUIEventHandlerProps>;
}

/**
 * Named color map (common colors only for MVP)
 */
const NAMED_COLORS: Record<string, number> = {
    black: 0x000000,
    white: 0xffffff,
    red: 0xff0000,
    green: 0x00ff00,
    blue: 0x0000ff,
    yellow: 0xffff00,
    cyan: 0x00ffff,
    magenta: 0xff00ff,
    gray: 0x808080,
    grey: 0x808080,
    orange: 0xffa500,
    purple: 0x800080,
    pink: 0xffc0cb,
    brown: 0xa52a2a,
    navy: 0x000080,
    teal: 0x008080,
    olive: 0x808000,
    maroon: 0x800000,
    lime: 0x00ff00,
    aqua: 0x00ffff,
    silver: 0xc0c0c0,
    transparent: 0x000000,
};

/**
 * Parse CSS color string to GPUI RGB value
 * Supports: hex (#RRGGBB, #RGB), rgb(r, g, b), rgba(r, g, b, a), named colors
 */
export function parseColor(color: string): number {
    if (!color) {
        return 0x000000;
    }

    if (color.startsWith("#")) {
        const hex = color.slice(1);
        if (hex.length === 3) {
            const r = parseInt(hex[0] + hex[0], 16);
            const g = parseInt(hex[1] + hex[1], 16);
            const b = parseInt(hex[2] + hex[2], 16);
            return (r << 16) | (g << 8) | b;
        } else if (hex.length === 6) {
            const r = parseInt(hex.slice(0, 2), 16);
            const g = parseInt(hex.slice(2, 4), 16);
            const b = parseInt(hex.slice(4, 6), 16);
            return (r << 16) | (g << 8) | b;
        }
    }

    const rgbMatch = color.match(/rgb\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)/);
    if (rgbMatch) {
        const r = parseInt(rgbMatch[1], 10);
        const g = parseInt(rgbMatch[2], 10);
        const b = parseInt(rgbMatch[3], 10);
        return (r << 16) | (g << 8) | b;
    }

    const rgbaMatch = color.match(/rgba\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*[\d.]+\s*\)/);
    if (rgbaMatch) {
        const r = parseInt(rgbaMatch[1], 10);
        const g = parseInt(rgbaMatch[2], 10);
        const b = parseInt(rgbaMatch[3], 10);
        return (r << 16) | (g << 8) | b;
    }

    const namedColor = NAMED_COLORS[color.toLowerCase()];
    if (namedColor !== undefined) {
        return namedColor;
    }

    console.warn(`Unknown color format: ${color}, using black`);
    return 0x000000;
}

/**
 * Parse size value to pixels
 * Supports: px, em, rem, %, number (assumed px)
 */
export function parseSize(size: number | string): number {
    if (typeof size === "number") {
        return size;
    }

    const s = size.trim();
    if (s.endsWith("px")) {
        return parseFloat(s.slice(0, -2));
    }
    if (s.endsWith("em")) {
        return parseFloat(s.slice(0, -2)) * 16;
    }
    if (s.endsWith("rem")) {
        return parseFloat(s.slice(0, -3)) * 16;
    }
    if (s.endsWith("%")) {
        return parseFloat(s.slice(0, -1));
    }
    const parsed = parseFloat(s);
    if (!isNaN(parsed)) {
        return parsed;
    }

    console.warn(`Unknown size format: ${size}, using 0`);
    return 0;
}

/**
 * Parse font weight to number (100-900)
 */
export function parseFontWeight(weight: string | number): number {
    if (typeof weight === "number") {
        return weight;
    }

    const normalized = weight.toLowerCase();
    const weightMap: Record<string, number> = {
        normal: 400,
        bold: 700,
        lighter: 300,
        bolder: 900,
    };

    return weightMap[normalized] || parseInt(normalized, 10) || 400;
}

/**
 * Parse margin/padding shorthand
 * Supports: 1-4 values (CSS-like syntax)
 * Returns: [top, right, bottom, left]
 */
export function parseSpacing(value: number | string): [number, number, number, number] {
    if (typeof value === "number") {
        return [value, value, value, value];
    }

    const parts = value.trim().split(/\s+/);
    const values = parts.map((p) => parseSize(p));

    switch (values.length) {
        case 1:
            return [values[0], values[0], values[0], values[0]];
        case 2:
            return [values[0], values[1], values[0], values[1]]; // top/bottom, left/right
        case 3:
            return [values[0], values[1], values[2], values[1]]; // top, left/right, bottom
        case 4:
            return [values[0], values[1], values[2], values[3]]; // top, right, bottom, left
        default:
            const parsed = parseSize(value);
            return [parsed, parsed, parsed, parsed];
    }
}

/**
 * Parse border shorthand
 * Supports: "1px solid #000", "2px dashed red"
 * Returns: { width, style, color }
 */
export function parseBorder(value: string): { width: number; style: string; color: number } | null {
    if (!value) return null;

    const parts = value.trim().split(/\s+/);
    let width = 1;
    let style = "solid";
    let color = 0x000000;

    for (const part of parts) {
        // Check if it's a size
        if (/^\d/.test(part)) {
            width = parseSize(part);
        }
        // Check if it's a style
        else if (part === "solid" || part === "dashed" || part === "dotted" || part === "none") {
            style = part;
        }
        // Otherwise it's a color
        else {
            color = parseColor(part);
        }
    }

    return { width, style, color };
}

/**
 * Parse box-shadow shorthand
 * Supports: "2px 2px 4px rgba(0,0,0,0.5)", "2px 2px 4px 1px #000"
 * Returns: { offsetX, offsetY, blur, spread, color }
 */
export function parseBoxShadow(
    value: string
): { offsetX: number; offsetY: number; blur: number; spread: number; color: number } | null {
    if (!value || value === "none") return null;

    // Extract color first (can be hex, rgb, rgba, or named)
    let color = 0x000000;
    let remaining = value;

    // Match rgba/rgb first
    const rgbaMatch = value.match(/rgba?\s*\([^)]+\)/);
    if (rgbaMatch) {
        color = parseColor(rgbaMatch[0]);
        remaining = value.replace(rgbaMatch[0], "").trim();
    } else {
        // Match hex or named color at the end
        const parts = value.trim().split(/\s+/);
        const lastPart = parts[parts.length - 1];
        if (lastPart.startsWith("#") || NAMED_COLORS[lastPart.toLowerCase()] !== undefined) {
            color = parseColor(lastPart);
            parts.pop();
            remaining = parts.join(" ");
        }
    }

    // Parse numeric values
    const numParts = remaining
        .trim()
        .split(/\s+/)
        .filter((p) => p);
    const nums = numParts.map((p) => parseSize(p));

    return {
        offsetX: nums[0] || 0,
        offsetY: nums[1] || 0,
        blur: nums[2] || 0,
        spread: nums[3] || 0,
        color,
    };
}

/**
 * Parse margin/padding individual values
 */
export function parseSpacingIndividual(
    top?: number | string,
    right?: number | string,
    bottom?: number | string,
    left?: number | string
): [number, number, number, number] {
    return [
        top !== undefined ? parseSize(top) : 0,
        right !== undefined ? parseSize(right) : 0,
        bottom !== undefined ? parseSize(bottom) : 0,
        left !== undefined ? parseSize(left) : 0,
    ];
}

/**
 * Map React style props to GPUI style object
 */
export function mapStyleToProps(props: StyleProps): Record<string, any> {
    const result: Record<string, any> = {};

    // Text properties (inheritable)
    if (props.color) {
        result.textColor = parseColor(props.color);
    }

    if (props.fontSize) {
        result.textSize = parseSize(props.fontSize);
    }

    if (props.fontWeight) {
        result.fontWeight = parseFontWeight(props.fontWeight);
    }

    if (props.fontFamily) {
        result.fontFamily = props.fontFamily;
    }

    if (props.lineHeight) {
        result.lineHeight = parseSize(props.lineHeight);
    }

    if (props.textAlign) {
        result.textAlign = props.textAlign;
    }

    if (props.letterSpacing !== undefined) {
        result.letterSpacing = parseSize(props.letterSpacing);
    }

    // Other inheritable properties
    if (props.cursor) {
        result.cursor = props.cursor;
    }

    if (props.visibility) {
        result.visibility = props.visibility;
    }

    // Non-inheritable properties
    if (props.backgroundColor) {
        result.bgColor = parseColor(props.backgroundColor);
    }

    // Size
    if (props.width) {
        result.width = parseSize(props.width);
    }

    if (props.height) {
        result.height = parseSize(props.height);
    }

    // Size constraints
    if (props.minWidth !== undefined) {
        result.minWidth = parseSize(props.minWidth);
    }
    if (props.maxWidth !== undefined) {
        result.maxWidth = parseSize(props.maxWidth);
    }
    if (props.minHeight !== undefined) {
        result.minHeight = parseSize(props.minHeight);
    }
    if (props.maxHeight !== undefined) {
        result.maxHeight = parseSize(props.maxHeight);
    }
    if (props.aspectRatio !== undefined) {
        result.aspectRatio = props.aspectRatio;
    }

    // Margin (shorthand and individual)
    if (props.margin !== undefined) {
        const [mt, mr, mb, ml] = parseSpacing(props.margin);
        result.marginTop = mt;
        result.marginRight = mr;
        result.marginBottom = mb;
        result.marginLeft = ml;
    }
    // Individual margins override shorthand
    if (props.marginTop !== undefined) result.marginTop = parseSize(props.marginTop);
    if (props.marginRight !== undefined) result.marginRight = parseSize(props.marginRight);
    if (props.marginBottom !== undefined) result.marginBottom = parseSize(props.marginBottom);
    if (props.marginLeft !== undefined) result.marginLeft = parseSize(props.marginLeft);

    // Padding (shorthand and individual)
    if (props.padding !== undefined) {
        const [pt, pr, pb, pl] = parseSpacing(props.padding);
        result.paddingTop = pt;
        result.paddingRight = pr;
        result.paddingBottom = pb;
        result.paddingLeft = pl;
    }
    // Individual paddings override shorthand
    if (props.paddingTop !== undefined) result.paddingTop = parseSize(props.paddingTop);
    if (props.paddingRight !== undefined) result.paddingRight = parseSize(props.paddingRight);
    if (props.paddingBottom !== undefined) result.paddingBottom = parseSize(props.paddingBottom);
    if (props.paddingLeft !== undefined) result.paddingLeft = parseSize(props.paddingLeft);

    // Position
    if (props.position) {
        result.position = props.position;
    }
    if (props.top !== undefined) {
        result.top = parseSize(props.top);
    }
    if (props.right !== undefined) {
        result.right = parseSize(props.right);
    }
    if (props.bottom !== undefined) {
        result.bottom = parseSize(props.bottom);
    }
    if (props.left !== undefined) {
        result.left = parseSize(props.left);
    }
    // inset shorthand sets all four
    if (props.inset !== undefined) {
        const insetVal = parseSize(props.inset);
        result.top = insetVal;
        result.right = insetVal;
        result.bottom = insetVal;
        result.left = insetVal;
    }

    // Overflow
    if (props.overflow) {
        result.overflowX = props.overflow;
        result.overflowY = props.overflow;
    }
    if (props.overflowX) {
        result.overflowX = props.overflowX;
    }
    if (props.overflowY) {
        result.overflowY = props.overflowY;
    }

    // Border shorthand
    if (props.border) {
        const borderParsed = parseBorder(props.border);
        if (borderParsed) {
            result.borderTopWidth = borderParsed.width;
            result.borderRightWidth = borderParsed.width;
            result.borderBottomWidth = borderParsed.width;
            result.borderLeftWidth = borderParsed.width;
            result.borderStyle = borderParsed.style;
            result.borderColor = borderParsed.color;
        }
    }

    // Border width shorthand
    if (props.borderWidth !== undefined) {
        const [bt, br, bb, bl] = parseSpacing(props.borderWidth);
        result.borderTopWidth = bt;
        result.borderRightWidth = br;
        result.borderBottomWidth = bb;
        result.borderLeftWidth = bl;
    }

    // Border individual properties
    if (props.borderStyle) {
        result.borderStyle = props.borderStyle;
    }
    if (props.borderColor) {
        result.borderColor = parseColor(props.borderColor);
    }

    // Individual border sides (shorthand)
    if (props.borderTop) {
        const parsed = parseBorder(props.borderTop);
        if (parsed) {
            result.borderTopWidth = parsed.width;
            result.borderTopColor = parsed.color;
        }
    }
    if (props.borderRight) {
        const parsed = parseBorder(props.borderRight);
        if (parsed) {
            result.borderRightWidth = parsed.width;
            result.borderRightColor = parsed.color;
        }
    }
    if (props.borderBottom) {
        const parsed = parseBorder(props.borderBottom);
        if (parsed) {
            result.borderBottomWidth = parsed.width;
            result.borderBottomColor = parsed.color;
        }
    }
    if (props.borderLeft) {
        const parsed = parseBorder(props.borderLeft);
        if (parsed) {
            result.borderLeftWidth = parsed.width;
            result.borderLeftColor = parsed.color;
        }
    }

    // Individual border widths
    if (props.borderTopWidth !== undefined) result.borderTopWidth = parseSize(props.borderTopWidth);
    if (props.borderRightWidth !== undefined)
        result.borderRightWidth = parseSize(props.borderRightWidth);
    if (props.borderBottomWidth !== undefined)
        result.borderBottomWidth = parseSize(props.borderBottomWidth);
    if (props.borderLeftWidth !== undefined)
        result.borderLeftWidth = parseSize(props.borderLeftWidth);

    // Individual border colors
    if (props.borderTopColor) result.borderTopColor = parseColor(props.borderTopColor);
    if (props.borderRightColor) result.borderRightColor = parseColor(props.borderRightColor);
    if (props.borderBottomColor) result.borderBottomColor = parseColor(props.borderBottomColor);
    if (props.borderLeftColor) result.borderLeftColor = parseColor(props.borderLeftColor);

    if (props.borderRadius) {
        result.borderRadius = parseSize(props.borderRadius);
    }

    // Box shadow
    if (props.boxShadow) {
        const shadow = parseBoxShadow(props.boxShadow);
        if (shadow) {
            result.boxShadowOffsetX = shadow.offsetX;
            result.boxShadowOffsetY = shadow.offsetY;
            result.boxShadowBlur = shadow.blur;
            result.boxShadowSpread = shadow.spread;
            result.boxShadowColor = shadow.color;
        }
    }

    // Flexbox
    if (props.display) {
        result.display = props.display;
    }

    if (props.flexDirection) {
        result.flexDirection = props.flexDirection;
    }

    if (props.flexWrap) {
        result.flexWrap = props.flexWrap;
    }

    if (props.flexGrow !== undefined) {
        result.flexGrow = props.flexGrow;
    }

    if (props.flexShrink !== undefined) {
        result.flexShrink = props.flexShrink;
    }

    if (props.flexBasis !== undefined) {
        result.flexBasis = parseSize(props.flexBasis);
    }

    if (props.justifyContent) {
        result.justifyContent = props.justifyContent;
    }

    if (props.alignItems) {
        result.alignItems = props.alignItems;
    }

    if (props.alignSelf) {
        result.alignSelf = props.alignSelf;
    }

    if (props.alignContent) {
        result.alignContent = props.alignContent;
    }

    // Gap
    if (props.gap !== undefined) {
        result.gap = parseSize(props.gap);
        result.rowGap = parseSize(props.gap);
        result.columnGap = parseSize(props.gap);
    }
    if (props.rowGap !== undefined) {
        result.rowGap = parseSize(props.rowGap);
    }
    if (props.columnGap !== undefined) {
        result.columnGap = parseSize(props.columnGap);
    }

    // Opacity
    if (props.opacity !== undefined) {
        result.opacity = Math.max(0, Math.min(1, props.opacity));
    }

    // Image specific
    if (props.src) {
        result.src = props.src;
    }
    if (props.alt) {
        result.alt = props.alt;
    }

    // Hover styles
    if (props._hover) {
        result.hoverStyle = mapStyleToProps(props._hover);
    }

    // Focus properties
    if (props.tabIndex !== undefined) {
        result.tabIndex = props.tabIndex;
    }

    return result;
}
