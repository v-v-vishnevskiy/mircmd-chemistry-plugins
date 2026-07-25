// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Image export control block (stub reactions; Browse is no-op until host saveFile).
 */

import {
  controlsStyles,
  createButton,
  createCheckbox,
  createColorField,
  createNumberField,
  createTextField,
} from "@mircmd/ui-controls";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import { ImageCommand, type MolecularVisualizerController } from "../controller";
import { createForm, createLabeledRow, wrapFullWidth } from "./form_row";
import { createSliderRow } from "./slider_row";
import styles from "./styles.css";

function hexToRgba(hex: string): [number, number, number, number] {
  const raw = hex.replace("#", "");
  const r = Number.parseInt(raw.slice(0, 2), 16) / 255;
  const g = Number.parseInt(raw.slice(2, 4), 16) / 255;
  const b = Number.parseInt(raw.slice(4, 6), 16) / 255;
  return [r, g, b, 0];
}

export function createImageBlock(
  _controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "image",
    title: "Image",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      const form = createForm();

      let scaleFactor = 1;
      let bgColor = "#ffffff";
      let cropToContent = true;
      let iParam = 1;

      const scaleRow = createSliderRow({
        label: "Scale factor:",
        value: scaleFactor,
        min: 0.1,
        max: 20,
        step: 0.1,
        decimals: 1,
        onChange: (value) => {
          scaleFactor = value;
        },
      });

      const bg = createColorField({
        value: bgColor,
        onChange: (value) => {
          bgColor = value;
        },
      });
      const bgRow = createLabeledRow({
        label: "Background color:",
        control: bg.root,
        spanControls: true,
      });

      const crop = createCheckbox({
        label: "",
        checked: cropToContent,
        onChange: (value) => {
          cropToContent = value;
        },
      });
      crop.root.replaceChildren(crop.input);
      const cropRow = createLabeledRow({
        label: "Crop to content:",
        control: crop.root,
        spanControls: true,
      });

      const iStarts = createNumberField({
        value: iParam,
        min: 1,
        max: 100000,
        step: 1,
        onChange: (value) => {
          iParam = Math.min(100000, Math.max(1, Math.round(value || 1)));
        },
      });
      const iRow = createLabeledRow({
        label: "%i starts from:",
        control: iStarts.root,
        spanControls: true,
      });

      const path = createTextField({
        value: "%n_%i.png",
        placeholder: "path template",
      });
      const browse = createButton({
        label: "Browse...",
        onClick: () => {
          // Stub: host save dialog / saveFile capability comes later.
        },
      });
      const pathRow = document.createElement("div");
      pathRow.style.display = "grid";
      pathRow.style.gridTemplateColumns = "1fr auto";
      pathRow.style.gap = "8px";
      pathRow.style.alignItems = "center";
      pathRow.append(path.root, browse.root);

      const save = createButton({
        label: "Save",
        onClick: () => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: ImageCommand.Save,
            payload: {
              t_filename: path.input.value,
              scale_factor: scaleFactor,
              bg_color: hexToRgba(bgColor),
              crop_to_content: cropToContent,
              i_param: iParam,
            },
          });
        },
      });

      form.append(
        scaleRow.root,
        bgRow.root,
        cropRow.root,
        iRow.root,
        wrapFullWidth(pathRow),
        wrapFullWidth(save.root),
      );
      surface.root.appendChild(form);

      return () => {
        scaleRow.destroy();
        bg.destroy();
        crop.destroy();
        iStarts.destroy();
        path.destroy();
        browse.destroy();
        save.destroy();
        bgRow.destroy();
        cropRow.destroy();
        iRow.destroy();
      };
    },
  };
}
