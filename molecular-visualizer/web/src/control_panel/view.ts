// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * View control block — rotation / scale / reset (stub reactions).
 */

import { controlsStyles, createButton } from "@mircmd/ui-controls";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import { ViewCommand, type MolecularVisualizerController } from "../controller";
import { createForm, wrapFullWidth } from "./form_row";
import { createSliderRow, type SliderRowControl } from "./slider_row";
import styles from "./styles.css";

export function createViewBlock(
  _controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "view",
    title: "View",
    initiallyExpanded: true,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      const form = createForm();

      let pitch = 0;
      let yaw = 0;
      let roll = 0;
      let scale = 1;

      const dispatchRotation = () => {
        if (context.signal.aborted) return;
        void context.dispatch({
          type: ViewCommand.SetSceneRotation,
          payload: { pitch, yaw, roll },
        });
      };

      const dispatchScale = () => {
        if (context.signal.aborted) return;
        void context.dispatch({
          type: ViewCommand.SetSceneScale,
          payload: { factor: scale },
        });
      };

      const pitchRow = createSliderRow({
        label: "Rotation X:",
        value: pitch,
        min: -180,
        max: 180,
        step: 0.1,
        decimals: 1,
        onChange: (value) => {
          pitch = value;
          dispatchRotation();
        },
      });
      const yawRow = createSliderRow({
        label: "Rotation Y:",
        value: yaw,
        min: -180,
        max: 180,
        step: 0.1,
        decimals: 1,
        onChange: (value) => {
          yaw = value;
          dispatchRotation();
        },
      });
      const rollRow = createSliderRow({
        label: "Rotation Z:",
        value: roll,
        min: -180,
        max: 180,
        step: 0.1,
        decimals: 1,
        onChange: (value) => {
          roll = value;
          dispatchRotation();
        },
      });
      const scaleRow = createSliderRow({
        label: "Scale:",
        value: scale,
        min: 0.01,
        max: 10,
        step: 0.01,
        decimals: 2,
        onChange: (value) => {
          scale = value;
          dispatchScale();
        },
      });

      const rows: SliderRowControl[] = [pitchRow, yawRow, rollRow, scaleRow];

      const reset = createButton({
        label: "Reset",
        onClick: () => {
          if (context.signal.aborted) return;
          pitch = 0;
          yaw = 0;
          roll = 0;
          scale = 1;
          pitchRow.setValue(0);
          yawRow.setValue(0);
          rollRow.setValue(0);
          scaleRow.setValue(1);
          dispatchRotation();
          dispatchScale();
        },
      });

      form.append(
        pitchRow.root,
        yawRow.root,
        rollRow.root,
        scaleRow.root,
        wrapFullWidth(reset.root),
      );
      surface.root.appendChild(form);

      return () => {
        for (const row of rows) row.destroy();
        reset.destroy();
      };
    },
  };
}
