// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Control Panel contribution for molecular visualizer.
 * View starts expanded.
 */

import type { ControlPanelContribution } from "../program_context";
import type { MolecularVisualizerController } from "../controller";
import { createCoordinateAxesBlock } from "./coordinate_axes";
import { createViewBlock } from "./view";
import { createAtomLabelsBlock } from "./atom_labels";
import { createAppearanceBlock } from "./appearance";
import { createCubesAndSurfacesBlock } from "./cubes_and_surfaces";
import { createImageBlock } from "./image";

export const MOLECULAR_VISUALIZER_BROADCAST_KEY = "mircmd:molecular-visualizer:v1";

export function createControlPanelContribution(
  controller: MolecularVisualizerController,
): ControlPanelContribution {
  return {
    title: "Molecular Visualizer",
    allowApplyToAll: true,
    broadcastKey: MOLECULAR_VISUALIZER_BROADCAST_KEY,
    blocks: [
      createViewBlock(controller),
      createAtomLabelsBlock(controller),
      createCubesAndSurfacesBlock(controller),
      createImageBlock(controller),
      createCoordinateAxesBlock(controller),
      createAppearanceBlock(controller),
    ],
  };
}
