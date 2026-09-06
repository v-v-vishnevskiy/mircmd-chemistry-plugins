// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Appearance control block — background color and named styles.
 */

import {
  applyControlStyles,
  createColorField,
  createForm,
  createLabeledRow,
  createSelect,
} from "@mircmd/ui-controls";
import {
  AppearanceCommand,
  type MolecularVisualizerController,
} from "../controller";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import type { VisualizerState } from "../wasm_types";
import { hexToRgba, rgbaToHex } from "./color_utils";

function styleOptions(snapshot: VisualizerState) {
  const names = snapshot.appearance.style_names ?? [];
  const options = names.map((name) => ({ value: name, label: name }));
  if (options.length === 0) {
    return [{ value: snapshot.appearance.style, label: snapshot.appearance.style }];
  }
  return options;
}

export function createAppearanceBlock(
  controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "appearance",
    title: "Appearance",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      applyControlStyles(surface);

      let disposed = false;
      let applying = false;
      const initial = await controller.getSnapshot();

      const bg = createColorField({
        value: rgbaToHex(initial.appearance.background),
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AppearanceCommand.SetBgColor,
            payload: { color: hexToRgba(value, 1) },
          });
        },
      });
      const bgRow = createLabeledRow({
        label: "Background color:",
        control: bg.root,
        spanControls: true,
      });

      const styleSelect = createSelect({
        value: initial.appearance.style,
        options: styleOptions(initial),
        onChange: (name) => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AppearanceCommand.SetStyle,
            payload: { name },
          });
        },
      });
      const styleRow = createLabeledRow({
        label: "Style:",
        control: styleSelect.root,
        spanControls: true,
      });

      const form = createForm();
      form.append(bgRow.root, styleRow.root);
      surface.root.appendChild(form);

      const applySnapshot = (snapshot: VisualizerState) => {
        if (disposed || context.signal.aborted) return;
        applying = true;
        try {
          bg.setValue(rgbaToHex(snapshot.appearance.background));
          styleSelect.setOptions(styleOptions(snapshot));
          styleSelect.setValue(snapshot.appearance.style);
        } finally {
          applying = false;
        }
      };

      applySnapshot(initial);
      const unsubscribe = controller.subscribe((snapshot, changedBlocks) => {
        if (disposed || context.signal.aborted) return;
        if (changedBlocks.length === 0 || changedBlocks.includes("appearance")) {
          applySnapshot(snapshot);
        }
      });

      return () => {
        disposed = true;
        unsubscribe();
        bg.destroy();
        styleSelect.destroy();
        bgRow.destroy();
        styleRow.destroy();
      };
    },
  };
}
