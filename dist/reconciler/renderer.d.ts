import * as React from "react";
export type Root = {
    render: (children: React.ReactNode) => void;
    unmount: () => void;
};
export type RootProps = {
    width: number;
    height: number;
    title?: string;
};
export declare function createRoot({ width, height, title }: RootProps): Root;
//# sourceMappingURL=renderer.d.ts.map