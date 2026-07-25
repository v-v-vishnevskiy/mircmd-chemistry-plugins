// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Control Panel for cartesian editor (General / Decimals).
 */

import { controlsStyles, createNumberField } from "@mircmd/ui-controls";
import type { ControlPanelBlock, ControlPanelContribution, Cleanup } from "./program_context";

export const CARTESIAN_EDITOR_BROADCAST_KEY = "mircmd:cartesian-editor:v1";

export type CartesianEditorState = {
  decimals: number;
};

export const DecimalsCommand = {
  Set: "cartesian.set_decimals",
} as const;

export function createCartesianControlPanel(
  getState: () => CartesianEditorState,
): ControlPanelContribution {
  const generalBlock: ControlPanelBlock = {
    id: "general",
    title: "General",
    initiallyExpanded: true,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);

      const field = createNumberField({
        label: "Decimals",
        value: getState().decimals,
        min: 1,
        max: 15,
        step: 1,
        onChange: (raw) => {
          if (context.signal.aborted) return;
          const value = Math.min(15, Math.max(1, Math.round(raw || 6)));
          field.setValue(value);
          void context.dispatch({ type: DecimalsCommand.Set, payload: value });
        },
      });

      const row = document.createElement("div");
      row.className = "mircmd-row";
      row.appendChild(field.root);
      surface.root.appendChild(row);

      return () => {
        field.destroy();
      };
    },
  };

  return {
    title: "Cartesian Editor",
    allowApplyToAll: true,
    broadcastKey: CARTESIAN_EDITOR_BROADCAST_KEY,
    blocks: [generalBlock],
  };
}
