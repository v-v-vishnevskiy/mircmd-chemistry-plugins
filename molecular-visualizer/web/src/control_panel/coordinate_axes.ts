// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Coordinate axes control block.
 * Live WASM: Show / Labels / Both directions / Center / Length / Thickness /
 * Font size / Adjust length / axis & label colors / label texts.
 */

import {
  controlsStyles,
  createButton,
  createCheckbox,
  createColorField,
  createTextField,
  type CheckboxControl,
  type ColorFieldControl,
  type TextFieldControl,
} from "@mircmd/ui-controls";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import {
  AxesCommand,
  type MolecularVisualizerController,
} from "../controller";
import type { Rgba } from "../wasm_types";
import { createForm, wrapFullWidth } from "./form_row";
import { createSliderRow, type SliderRowControl } from "./slider_row";
import styles from "./styles.css";

type AxisId = "x" | "y" | "z";

type AxisRow = {
  root: HTMLElement;
  axisColor: ColorFieldControl;
  labelColor: ColorFieldControl;
  text: TextFieldControl;
  destroy(): void;
};

function hexToRgba(hex: string): [number, number, number, number] {
  const raw = hex.replace("#", "");
  const r = Number.parseInt(raw.slice(0, 2), 16) / 255;
  const g = Number.parseInt(raw.slice(2, 4), 16) / 255;
  const b = Number.parseInt(raw.slice(4, 6), 16) / 255;
  return [r, g, b, 1];
}

function rgbaToHex(color: Rgba): string {
  const toByte = (v: number) =>
    Math.round(Math.min(1, Math.max(0, v)) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${toByte(color.r)}${toByte(color.g)}${toByte(color.b)}`;
}

function createAxisRow(
  axis: AxisId,
  labelText: string,
  axisColor: string,
  labelColor: string,
  textValue: string,
  dispatch: (type: string, payload: unknown) => void,
): AxisRow {
  const root = document.createElement("div");
  root.className = "cp-form-row";

  const label = document.createElement("span");
  label.className = "cp-label";
  label.textContent = labelText;

  const axisColorField = createColorField({
    value: axisColor,
    onChange: (color) => {
      dispatch(AxesCommand.SetColor, { axis, color: hexToRgba(color) });
    },
  });
  const labelColorField = createColorField({
    value: labelColor,
    onChange: (color) => {
      dispatch(AxesCommand.SetLabelColor, { axis, color: hexToRgba(color) });
    },
  });
  const text = createTextField({
    value: textValue,
    onInput: (value) => {
      dispatch(AxesCommand.SetText, { axis, text: value });
    },
  });

  root.append(label, axisColorField.root, labelColorField.root, text.root);

  return {
    root,
    axisColor: axisColorField,
    labelColor: labelColorField,
    text,
    destroy() {
      axisColorField.destroy();
      labelColorField.destroy();
      text.destroy();
      root.remove();
    },
  };
}

export function createCoordinateAxesBlock(
  controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "coordinate_axes",
    title: "Coordinate axes",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      let applying = false;
      let disposed = false;

      const initial = controller.getSnapshot().coordinate_axes;

      const dispatchCmd = (type: string, payload: unknown) => {
        if (applying || disposed || context.signal.aborted) return;
        void context.dispatch({ type, payload });
      };

      const stack = document.createElement("div");
      stack.className = "cp-stack";

      const show = createCheckbox({ label: "Show" });
      const labels = createCheckbox({ label: "Labels" });
      const both = createCheckbox({ label: "Both directions" });
      const center = createCheckbox({ label: "Center" });

      const checkboxes = document.createElement("div");
      checkboxes.className = "axes-grid";
      checkboxes.append(show.root, labels.root, both.root, center.root);

      const sliderForm = createForm();
      const lengthRow = createSliderRow({
        label: "Length:",
        value: initial.length,
        min: 0.5,
        max: 100,
        step: 0.1,
        decimals: 1,
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AxesCommand.SetLength,
            payload: { value },
          });
        },
      });
      const thicknessRow = createSliderRow({
        label: "Thickness:",
        value: initial.thickness,
        min: 0.03,
        max: 1,
        step: 0.01,
        decimals: 2,
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AxesCommand.SetThickness,
            payload: { value },
          });
        },
      });
      const fontSizeRow = createSliderRow({
        label: "Font size:",
        value: initial.font_size,
        min: 16,
        max: 500,
        step: 1,
        decimals: 0,
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AxesCommand.SetFontSize,
            payload: { value },
          });
        },
      });

      const adjust = createButton({
        label: "Adjust length",
        onClick: () => {
          if (applying || disposed || context.signal.aborted) return;
          void (async () => {
            await context.dispatch({
              type: AxesCommand.AdjustLength,
              payload: {},
            });
            if (disposed || context.signal.aborted) return;
            applying = true;
            try {
              lengthRow.setValue(controller.getSnapshot().coordinate_axes.length);
            } finally {
              applying = false;
            }
          })();
        },
      });

      sliderForm.append(
        lengthRow.root,
        thicknessRow.root,
        fontSizeRow.root,
        wrapFullWidth(adjust.root),
      );

      const axesForm = createForm();
      axesForm.classList.add("cp-form-axes");
      const axisX = createAxisRow(
        "x",
        "Axis X:",
        rgbaToHex(initial.x.color),
        rgbaToHex(initial.x.label_color),
        initial.x.label,
        dispatchCmd,
      );
      const axisY = createAxisRow(
        "y",
        "Axis Y:",
        rgbaToHex(initial.y.color),
        rgbaToHex(initial.y.label_color),
        initial.y.label,
        dispatchCmd,
      );
      const axisZ = createAxisRow(
        "z",
        "Axis Z:",
        rgbaToHex(initial.z.color),
        rgbaToHex(initial.z.label_color),
        initial.z.label,
        dispatchCmd,
      );
      axesForm.append(axisX.root, axisY.root, axisZ.root);

      const sliderRows: SliderRowControl[] = [lengthRow, thicknessRow, fontSizeRow];
      const axisRows = [axisX, axisY, axisZ];

      stack.append(checkboxes, sliderForm, axesForm);
      surface.root.appendChild(stack);

      const dependentCheckboxes: CheckboxControl[] = [labels, both, center];
      const extraControls = [
        ...sliderRows,
        ...axisRows.flatMap((row) => [row.axisColor, row.labelColor, row.text]),
      ];

      const setExtrasEnabled = (enabled: boolean) => {
        for (const control of extraControls) {
          control.setDisabled(!enabled);
        }
      };

      const applySnapshot = () => {
        if (disposed || context.signal.aborted) return;
        const axes = controller.getSnapshot().coordinate_axes;
        applying = true;
        try {
          show.setChecked(axes.visible);
          labels.setChecked(axes.labels_visible);
          both.setChecked(axes.both_directions);
          // Center checked <=> axes are placed at molecule center (not origin).
          center.setChecked(!axes.use_origin);
          show.setDisabled(false);
          for (const control of dependentCheckboxes) {
            control.setDisabled(!axes.visible);
          }
          setExtrasEnabled(axes.visible);
          adjust.setDisabled(!axes.visible || !axes.auto_adjust_available);
          const byAxis = [
            { row: axisX, state: axes.x },
            { row: axisY, state: axes.y },
            { row: axisZ, state: axes.z },
          ];
          for (const { row, state } of byAxis) {
            row.axisColor.setValue(rgbaToHex(state.color));
            row.labelColor.setValue(rgbaToHex(state.label_color));
          }
        } finally {
          applying = false;
        }
      };

      applySnapshot();

      const unsubscribe = controller.subscribe((_snapshot, changedBlocks) => {
        if (disposed || context.signal.aborted) return;
        if (
          changedBlocks.length === 0 ||
          changedBlocks.includes("coordinate_axes")
        ) {
          applySnapshot();
        }
      });

      const bind = (
        control: CheckboxControl,
        type: string,
        mapValue: (checked: boolean) => boolean = (v) => v,
      ) => {
        const onChange = () => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type,
            payload: { value: mapValue(control.input.checked) },
          });
        };
        control.input.addEventListener("change", onChange);
        return () => control.input.removeEventListener("change", onChange);
      };

      const unbindShow = bind(show, AxesCommand.SetVisible);
      const unbindLabels = bind(labels, AxesCommand.SetLabelsVisible);
      const unbindBoth = bind(both, AxesCommand.SetBothDirections);
      // UI "Center" checked -> use_origin = false
      const unbindCenter = bind(center, AxesCommand.SetUseOrigin, (checked) => !checked);

      const onShowChange = () => {
        if (applying || disposed) return;
        const enabled = show.input.checked;
        for (const control of dependentCheckboxes) {
          control.setDisabled(!enabled);
        }
        setExtrasEnabled(enabled);
      };
      show.input.addEventListener("change", onShowChange);

      const onAbort = () => {
        disposed = true;
      };
      context.signal.addEventListener("abort", onAbort, { once: true });

      return () => {
        disposed = true;
        context.signal.removeEventListener("abort", onAbort);
        show.input.removeEventListener("change", onShowChange);
        unsubscribe();
        unbindShow();
        unbindLabels();
        unbindBoth();
        unbindCenter();
        show.destroy();
        labels.destroy();
        both.destroy();
        center.destroy();
        for (const row of sliderRows) row.destroy();
        adjust.destroy();
        for (const row of axisRows) row.destroy();
      };
    },
  };
}
