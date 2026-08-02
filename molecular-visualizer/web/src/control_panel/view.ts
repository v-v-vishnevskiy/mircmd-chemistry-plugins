// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * View control block — rotation / scale / reset.
 */

import { controlsStyles, createButton } from "@mircmd/ui-controls";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import { ViewCommand, type MolecularVisualizerController } from "../controller";
import type { VisualizerState } from "../wasm_types";
import { createForm, wrapFullWidth } from "./form_row";
import { createSliderRow, type SliderRowControl } from "./slider_row";
import styles from "./styles.css";

type RotationAxis = "pitch" | "yaw" | "roll";

export function createViewBlock(
  controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "view",
    title: "View",
    initiallyExpanded: true,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      const form = createForm();
      let disposed = false;
      let applying = false;

      const initial = controller.getSnapshot().transform;
      let pitch = initial.pitch;
      let yaw = initial.yaw;
      let roll = initial.roll;
      let scale = initial.scale;
      let prevPitch = pitch;
      let prevYaw = yaw;
      let prevRoll = roll;
      let prevScale = scale;

      const dispatchRotationDelta = (axis: RotationAxis, value: number) => {
        if (applying || disposed || context.signal.aborted) return;

        pitch = axis === "pitch" ? value : pitch;
        yaw = axis === "yaw" ? value : yaw;
        roll = axis === "roll" ? value : roll;

        void context.dispatch({
          type: ViewCommand.RotateScene,
          payload: {
            pitch: pitch - prevPitch,
            yaw: yaw - prevYaw,
            roll: roll - prevRoll,
          },
        });

        if (axis === "pitch") prevPitch = value;
        else if (axis === "yaw") prevYaw = value;
        else prevRoll = value;
      };

      const dispatchScaleFactor = (value: number) => {
        if (applying || disposed || context.signal.aborted) return;

        scale = value;
        const factor = prevScale === 0 ? value : value / prevScale;
        void context.dispatch({
          type: ViewCommand.ScaleScene,
          payload: { factor },
        });
        prevScale = value;
      };

      const pitchRow = createSliderRow({
        label: "Rotation X:",
        value: pitch,
        min: -180,
        max: 180,
        step: 0.1,
        decimals: 1,
        onChange: (value) => {
          if (applying || disposed) return;
          dispatchRotationDelta("pitch", value);
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
          if (applying || disposed) return;
          dispatchRotationDelta("yaw", value);
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
          if (applying || disposed) return;
          dispatchRotationDelta("roll", value);
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
          if (applying || disposed) return;
          dispatchScaleFactor(value);
        },
      });

      const rows: SliderRowControl[] = [pitchRow, yawRow, rollRow, scaleRow];

      const applySnapshot = (snapshot: VisualizerState = controller.getSnapshot()) => {
        if (disposed || context.signal.aborted) return;
        const t = snapshot.transform;
        applying = true;
        try {
          pitch = t.pitch;
          yaw = t.yaw;
          roll = t.roll;
          scale = t.scale;
          prevPitch = t.pitch;
          prevYaw = t.yaw;
          prevRoll = t.roll;
          prevScale = t.scale;
          pitchRow.setValue(t.pitch);
          yawRow.setValue(t.yaw);
          rollRow.setValue(t.roll);
          scaleRow.setValue(t.scale);
        } finally {
          applying = false;
        }
      };

      const reset = createButton({
        label: "Reset",
        onClick: () => {
          if (applying || disposed || context.signal.aborted) return;
          pitch = 0;
          yaw = 0;
          roll = 0;
          scale = 1;
          prevPitch = 0;
          prevYaw = 0;
          prevRoll = 0;
          prevScale = 1;
          applying = true;
          try {
            pitchRow.setValue(0);
            yawRow.setValue(0);
            rollRow.setValue(0);
            scaleRow.setValue(1);
          } finally {
            applying = false;
          }
          void context.dispatch({
            type: ViewCommand.SetSceneRotation,
            payload: { pitch: 0, yaw: 0, roll: 0 },
          });
          void context.dispatch({
            type: ViewCommand.SetSceneScale,
            payload: { factor: 1 },
          });
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

      applySnapshot();

      const unsubscribe = controller.subscribe((snapshot, changedBlocks) => {
        if (disposed || context.signal.aborted) return;
        if (changedBlocks.length === 0 || changedBlocks.includes("view")) {
          applySnapshot(snapshot);
        }
      });

      const onAbort = () => {
        disposed = true;
      };
      context.signal.addEventListener("abort", onAbort, { once: true });

      return () => {
        disposed = true;
        context.signal.removeEventListener("abort", onAbort);
        unsubscribe();
        for (const row of rows) row.destroy();
        reset.destroy();
      };
    },
  };
}
