import ReactReconciler from "react-reconciler";
import { hostConfig } from "./host-config";
import { ConcurrentRoot } from "react-reconciler/constants";
export const reconciler = ReactReconciler(hostConfig);
export function _render(element, root) {
    const container = reconciler.createContainer(root, ConcurrentRoot, null, false, null, "", console.error, console.error, console.error, console.error, null);
    reconciler.updateContainer(element, container, null, () => { });
    return container;
}
