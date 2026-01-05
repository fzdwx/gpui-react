/**
 * Style Utilities for React-GPUI Renderer
 *
 * Handles conversion between React style props and GPUI style values
 */
/**
 * Parse CSS color string to GPUI RGB value
 * Supports: hex (#RRGGBB, #RGB), rgb(r, g, b), rgba(r, g, b, a), named colors
 */
export function parseColor(color) {
    if (!color) {
        return 0x000000;
    }
    if (color.startsWith('#')) {
        const hex = color.slice(1);
        if (hex.length === 3) {
            const r = parseInt(hex[0] + hex[0], 16);
            const g = parseInt(hex[1] + hex[1], 16);
            const b = parseInt(hex[2] + hex[2], 16);
            return (r << 16) | (g << 8) | b;
        }
        else if (hex.length === 6) {
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
export function parseSize(size) {
    if (typeof size === 'number') {
        return size;
    }
    if (typeof size === 'string') {
        const s = size.trim();
        if (s.endsWith('px')) {
            return parseFloat(s.slice(0, -2));
        }
        if (s.endsWith('em')) {
            return parseFloat(s.slice(0, -2)) * 16;
        }
        if (s.endsWith('rem')) {
            return parseFloat(s.slice(0, -3)) * 16;
        }
        if (s.endsWith('%')) {
            return parseFloat(s.slice(0, -1));
        }
        const parsed = parseFloat(s);
        if (!isNaN(parsed)) {
            return parsed;
        }
    }
    console.warn(`Unknown size format: ${size}, using 0`);
    return 0;
}
/**
 * Parse font weight
 */
export function parseFontWeight(weight) {
    if (typeof weight === 'number') {
        return weight.toString();
    }
    const normalized = weight.toLowerCase();
    const weightMap = {
        'normal': '400',
        'bold': '700',
        'lighter': '300',
        'bolder': '900',
    };
    return weightMap[normalized] || normalized;
}
/**
 * Parse margin/padding shorthand
 * Supports: 1-4 values (CSS-like syntax)
 * Returns: [top, right, bottom, left]
 */
export function parseSpacing(value) {
    const parsed = parseSize(value);
    return [parsed, parsed, parsed, parsed];
}
/**
 * Parse margin/padding individual values
 */
export function parseSpacingIndividual(top, right, bottom, left) {
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
export function mapStyleToProps(props) {
    const result = {};
    if (props.color) {
        result.textColor = parseColor(props.color);
    }
    if (props.backgroundColor) {
        result.bgColor = parseColor(props.backgroundColor);
    }
    if (props.borderColor) {
        result.borderColor = parseColor(props.borderColor);
    }
    if (props.fontSize) {
        result.textSize = parseSize(props.fontSize);
    }
    if (props.fontWeight) {
        result.fontWeight = parseFontWeight(props.fontWeight);
    }
    if (props.width) {
        result.width = parseSize(props.width);
    }
    if (props.height) {
        result.height = parseSize(props.height);
    }
    if (props.margin !== undefined) {
        const [mt, mr, mb, ml] = parseSpacing(props.margin);
        result.marginTop = mt;
        result.marginRight = mr;
        result.marginBottom = mb;
        result.marginLeft = ml;
    }
    else {
        if (props.marginTop !== undefined)
            result.marginTop = parseSize(props.marginTop);
        if (props.marginRight !== undefined)
            result.marginRight = parseSize(props.marginRight);
        if (props.marginBottom !== undefined)
            result.marginBottom = parseSize(props.marginBottom);
        if (props.marginLeft !== undefined)
            result.marginLeft = parseSize(props.marginLeft);
    }
    if (props.padding !== undefined) {
        const [pt, pr, pb, pl] = parseSpacing(props.padding);
        result.paddingTop = pt;
        result.paddingRight = pr;
        result.paddingBottom = pb;
        result.paddingLeft = pl;
    }
    else {
        if (props.paddingTop !== undefined)
            result.paddingTop = parseSize(props.paddingTop);
        if (props.paddingRight !== undefined)
            result.paddingRight = parseSize(props.paddingRight);
        if (props.paddingBottom !== undefined)
            result.paddingBottom = parseSize(props.paddingBottom);
        if (props.paddingLeft !== undefined)
            result.paddingLeft = parseSize(props.paddingLeft);
    }
    if (props.display) {
        result.display = props.display;
    }
    if (props.flexDirection) {
        result.flexDirection = props.flexDirection;
    }
    if (props.justifyContent) {
        result.justifyContent = props.justifyContent;
    }
    if (props.alignItems) {
        result.alignItems = props.alignItems;
    }
    if (props.gap !== undefined) {
        result.gap = parseSize(props.gap);
    }
    if (props.borderRadius) {
        result.borderRadius = parseSize(props.borderRadius);
    }
    if (props.opacity !== undefined) {
        result.opacity = Math.max(0, Math.min(1, props.opacity));
    }
    return result;
}
/**
 * Named color map (common colors only for MVP)
 */
const NAMED_COLORS = {
    'black': 0x000000,
    'white': 0xffffff,
    'red': 0xff0000,
    'green': 0x00ff00,
    'blue': 0x0000ff,
    'yellow': 0xffff00,
    'cyan': 0x00ffff,
    'magenta': 0xff00ff,
    'gray': 0x808080,
    'grey': 0x808080,
    'transparent': 0x000000,
};
