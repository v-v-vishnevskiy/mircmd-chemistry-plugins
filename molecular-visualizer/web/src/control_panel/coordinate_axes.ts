// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Coordinate axes control block.
 * Live WASM: Show / Labels / Both directions / Center / Length / Thickness /
 * Font size / Adjust length / axis & label colors / label texts.
 */

import {
  applyControlStyles,
  createButton,
  createCheckbox,
  createColorField,
  createForm,
  createSliderRow,
  createTextField,
  wrapFullWidth,
  type CheckboxControl,
  type ColorFieldControl,
  type SliderRowControl,
  type TextFieldControl,
} from "@mircmd/ui-controls";
import {
  AxesCommand,
  type MolecularVisualizerController,
} from "../controller";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import type { CoordinateAxesState } from "../wasm_types";
import { hexToRgba, rgbaToHex } from "./color_utils";
import styles from "./styles.css";

type AxisId = "x" | "y" | "z";

type AxisRow = {
  root: HTMLElement;
  axisColor: ColorFieldControl;
  labelColor: ColorFieldControl;
  text: TextFieldControl;
  destroy(): void;
};

function createAxisRow(
  axis: AxisId,
  labelText: string,
  axisColor: string,
  labelColor: string,
  textValue: string,
  dispatch: (type: string, payload: unknown) => void,
): AxisRow {
  const root = document.createElement("div");
  root.className = "mircmd-form-row";

  const label = document.createElement("span");
  label.className = "mircmd-label";
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
      applyControlStyles(surface, styles);

      let applying = false;
      let disposed = false;

      const initial = (await controller.getSnapshot()).coordinate_axes;

      const dispatchCmd = (type: string, payload: unknown) => {
        if (applying || disposed || context.signal.aborted) return;
        void context.dispatch({ type, payload });
      };

      const stack = document.createElement("div");
      stack.className = "mircmd-stack";

      let dependentCheckboxes: CheckboxControl[] = [];
      let setExtrasEnabled: (enabled: boolean) => void = () => {};

      const labels = createCheckbox({
        label: "Labels",
        onChange: (checked) => dispatchCmd(AxesCommand.SetLabelsVisible, { value: checked }),
      });
      const both = createCheckbox({
        label: "Both directions",
        onChange: (checked) => dispatchCmd(AxesCommand.SetBothDirections, { value: checked }),
      });
      const center = createCheckbox({
        label: "Center",
        onChange: (checked) => dispatchCmd(AxesCommand.SetUseOrigin, { value: !checked }),
      });
      const show = createCheckbox({
        label: "Show",
        onChange: (checked) => {
          dispatchCmd(AxesCommand.SetVisible, { value: checked });
          if (applying || disposed || context.signal.aborted) return;
          for (const control of dependentCheckboxes) {
            control.setDisabled(!checked);
          }
          setExtrasEnabled(checked);
        },
      });

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
          void context.dispatch({
            type: AxesCommand.AdjustLength,
            payload: {},
          });
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

      dependentCheckboxes = [labels, both, center];
      const extraControls = [
        ...sliderRows,
        ...axisRows.flatMap((row) => [row.axisColor, row.labelColor, row.text]),
      ];

      setExtrasEnabled = (enabled: boolean) => {
        for (const control of extraControls) {
          control.setDisabled(!enabled);
        }
      };

      const applySnapshot = (snapshot: { coordinate_axes: CoordinateAxesState }) => {
        if (disposed || context.signal.aborted) return;
        const axes = snapshot.coordinate_axes;
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
          lengthRow.setValue(axes.length);
          thicknessRow.setValue(axes.thickness);
          fontSizeRow.setValue(axes.font_size);
          const byAxis = [
            { row: axisX, state: axes.x },
            { row: axisY, state: axes.y },
            { row: axisZ, state: axes.z },
          ];
          for (const { row, state } of byAxis) {
            row.axisColor.setValue(rgbaToHex(state.color));
            row.labelColor.setValue(rgbaToHex(state.label_color));
            row.text.setValue(state.label);
          }
        } finally {
          applying = false;
        }
      };

      applySnapshot({ coordinate_axes: initial });

      const unsubscribe = controller.subscribe((snapshot, changedBlocks) => {
        if (disposed || context.signal.aborted) return;
        if (
          changedBlocks.length === 0 ||
          changedBlocks.includes("coordinate_axes")
        ) {
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
