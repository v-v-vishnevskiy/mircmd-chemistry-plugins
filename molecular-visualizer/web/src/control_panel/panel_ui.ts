// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Per-window control-panel draft (not WASM). Survives Control Panel remount.
 */

export const DEFAULT_IMAGE_FILENAME = "%n_%i.png";

export type ImagePanelUi = {
  scaleFactor: number;
  filename: string;
  cropToContent: boolean;
  iParam: number;
  /** Set on first Image mount from the scene background (alpha 0). */
  bgColor?: string;
};

export type CubesPanelUi = {
  value: number;
  color1: string;
  color2: string;
  inverse: boolean;
  expandedById: Record<string, boolean>;
};

export type PanelUi = {
  image: ImagePanelUi;
  cubes: CubesPanelUi;
};

export function createPanelUi(): PanelUi {
  return {
    image: {
      scaleFactor: 1,
      filename: DEFAULT_IMAGE_FILENAME,
      cropToContent: true,
      iParam: 1,
    },
    cubes: {
      value: 0.05,
      color1: "#ff0000c8",
      color2: "#0000ffc8",
      inverse: false,
      expandedById: {},
    },
  };
}
