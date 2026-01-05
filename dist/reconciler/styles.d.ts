/**
 * Style Utilities for React-GPUI Renderer
 *
 * Handles conversion between React style props and GPUI style values
 */
export interface StyleProps {
    color?: string;
    backgroundColor?: string;
    borderColor?: string;
    fontSize?: number | string;
    fontWeight?: string | number;
    width?: number | string;
    height?: number | string;
    margin?: number | string;
    marginTop?: number | string;
    marginRight?: number | string;
    marginBottom?: number | string;
    marginLeft?: number | string;
    padding?: number | string;
    paddingTop?: number | string;
    paddingRight?: number | string;
    paddingBottom?: number | string;
    paddingLeft?: number | string;
    display?: 'flex' | 'block' | 'inline' | 'inline-block';
    flexDirection?: 'row' | 'column';
    justifyContent?: 'flex-start' | 'center' | 'flex-end' | 'space-between' | 'space-around' | 'space-evenly';
    alignItems?: 'flex-start' | 'center' | 'flex-end' | 'stretch';
    gap?: number | string;
    borderRadius?: number | string;
    opacity?: number;
    src?: string;
    alt?: string;
    onClick?: (event: MouseEvent) => void;
    onHover?: (event: MouseEvent) => void;
    onMouseEnter?: (event: MouseEvent) => void;
    onMouseLeave?: (event: MouseEvent) => void;
}
/**
 * Parse CSS color string to GPUI RGB value
 * Supports: hex (#RRGGBB, #RGB), rgb(r, g, b), rgba(r, g, b, a), named colors
 */
export declare function parseColor(color: string): number;
/**
 * Parse size value to pixels
 * Supports: px, em, rem, %, number (assumed px)
 */
export declare function parseSize(size: number | string): number;
/**
 * Parse font weight
 */
export declare function parseFontWeight(weight: string | number): string;
/**
 * Parse margin/padding shorthand
 * Supports: 1-4 values (CSS-like syntax)
 * Returns: [top, right, bottom, left]
 */
export declare function parseSpacing(value: number | string): [number, number, number, number];
/**
 * Parse margin/padding individual values
 */
export declare function parseSpacingIndividual(top?: number | string, right?: number | string, bottom?: number | string, left?: number | string): [number, number, number, number];
/**
 * Map React style props to GPUI style object
 */
export declare function mapStyleToProps(props: StyleProps): Record<string, any>;
//# sourceMappingURL=styles.d.ts.map