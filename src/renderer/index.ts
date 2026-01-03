import * as React from "react";
import { reconciler } from "./host-config";
import { init, createWindow, run } from "./gpui-binding";

export type Root = {
  render: (children: React.ReactNode) => void;
  unmount: () => void;
  run: () => void;
};

export function createRoot(): Root {
  init(800, 600);
  createWindow(800, 600);

  const rootContainer = reconciler.createContainer(
    1,
    null,
    false,
    null,
    "",
    () => {},
    null
  );

  return {
    render(children: React.ReactNode) {
      reconciler.updateContainer(
        children,
        rootContainer,
        null,
        null,
        () => {
          console.log("Render complete");
        }
      );
    },
    unmount() {
      reconciler.updateContainer(null, rootContainer, null, null, () => {});
    },
    run() {
      run();
    },
  };
}
