import { RustLib } from "./rust";

export type { ElementData, WindowOptions } from "./rust";

const rustLib = new RustLib();

export { rustLib };
