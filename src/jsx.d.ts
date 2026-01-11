/**
 * Custom JSX intrinsic element extensions for gpui-react
 *
 * Extends HTML input element with gpui-specific props
 */

import "react";

declare module "react" {
    interface InputHTMLAttributes<T> {
        /** Enable multi-line mode (textarea-like behavior) */
        multiLine?: boolean;
        /** Number of visible rows for multi-line input */
        rows?: number;
    }
}
