// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Control Panel contribution for molecular visualizer.
 * View starts expanded.
 */

import type { MolecularVisualizerController } from "../controller";
import type { ControlPanelContribution } from "../program_context";
import { createAppearanceBlock } from "./appearance";
import { createAtomLabelsBlock } from "./atom_labels";
import { createCoordinateAxesBlock } from "./coordinate_axes";
import { createCubesAndSurfacesBlock } from "./cubes_and_surfaces";
import { createImageBlock } from "./image";
import type { PanelUi } from "./panel_ui";
import { createViewBlock } from "./view";

export const MOLECULAR_VISUALIZER_BROADCAST_KEY = "mircmd:molecular-visualizer:v1";

export function createControlPanelContribution(
  controller: MolecularVisualizerController,
  panelUi: PanelUi,
): ControlPanelContribution {
  return {
    title: "Molecular Visualizer",
    allowApplyToAll: true,
    broadcastKey: MOLECULAR_VISUALIZER_BROADCAST_KEY,
    blocks: [
      createViewBlock(controller),
      createAtomLabelsBlock(controller),
      createCubesAndSurfacesBlock(controller, panelUi.cubes),
      createImageBlock(controller, panelUi.image),
      createCoordinateAxesBlock(controller),
      createAppearanceBlock(controller),
    ],
  };
}
