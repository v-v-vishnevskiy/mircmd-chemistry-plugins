// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Appearance control block (stub reactions).
 */

import {
  controlsStyles,
  createColorField,
  createSelect,
} from "@mircmd/ui-controls";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import {
  AppearanceCommand,
  type MolecularVisualizerController,
} from "../controller";
import { createForm, createLabeledRow } from "./form_row";
import styles from "./styles.css";

function hexToRgba(hex: string): [number, number, number, number] {
  const raw = hex.replace("#", "");
  const r = Number.parseInt(raw.slice(0, 2), 16) / 255;
  const g = Number.parseInt(raw.slice(2, 4), 16) / 255;
  const b = Number.parseInt(raw.slice(4, 6), 16) / 255;
  return [r, g, b, 1];
}

export function createAppearanceBlock(
  _controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "appearance",
    title: "Appearance",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      const form = createForm();

      const bg = createColorField({
        value: "#ffffff",
        onChange: (value) => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: AppearanceCommand.SetBgColor,
            payload: { color: hexToRgba(value) },
          });
        },
      });
      const bgRow = createLabeledRow({
        label: "Background color:",
        control: bg.root,
        spanControls: true,
      });

      const styleSelect = createSelect({
        value: "Default",
        options: [{ value: "Default", label: "Default" }],
        onChange: (name) => {
          if (context.signal.aborted) return;
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

      form.append(bgRow.root, styleRow.root);
      surface.root.appendChild(form);

      return () => {
        bg.destroy();
        styleSelect.destroy();
        bgRow.destroy();
        styleRow.destroy();
      };
    },
  };
}
