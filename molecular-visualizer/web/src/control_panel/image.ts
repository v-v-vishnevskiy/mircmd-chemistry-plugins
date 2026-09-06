// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Image export control block.
 */

import {
  applyControlStyles,
  createButton,
  createCheckbox,
  createColorField,
  createForm,
  createLabeledRow,
  createNumberField,
  createPathField,
  createSliderRow,
  wrapFullWidth,
} from "@mircmd/ui-controls";
import { ImageCommand, type MolecularVisualizerController } from "../controller";
import type { Cleanup, ControlPanelBlock, ProgramFs } from "../program_context";
import { hexToRgba, rgbaToHex } from "./color_utils";
import { DEFAULT_IMAGE_FILENAME, type ImagePanelUi } from "./panel_ui";

const IMAGE_FILTERS = [
  { name: "PNG", extensions: ["png"] },
  { name: "JPEG", extensions: ["jpg", "jpeg"] },
];

function joinCwd(cwd: string, filename: string): string {
  if (!cwd) return filename;
  const sep = cwd.includes("\\") && !cwd.includes("/") ? "\\" : "/";
  if (cwd.endsWith("/") || cwd.endsWith("\\")) return `${cwd}${filename}`;
  return `${cwd}${sep}${filename}`;
}

async function resolveDefaultFilename(fs: ProgramFs, ui: ImagePanelUi): Promise<void> {
  if (ui.filename !== DEFAULT_IMAGE_FILENAME) return;
  ui.filename = joinCwd(await fs.getCwd(), DEFAULT_IMAGE_FILENAME);
}

export function createImageBlock(
  controller: MolecularVisualizerController,
  ui: ImagePanelUi,
): ControlPanelBlock {
  return {
    id: "image",
    title: "Image",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      applyControlStyles(surface);

      const form = createForm();
      let disposed = false;
      let saving = false;

      if (ui.bgColor === undefined) {
        const initialBg = (await controller.getSnapshot()).appearance.background;
        ui.bgColor = rgbaToHex({ ...initialBg, a: 0 }, true);
      }
      await resolveDefaultFilename(context.fs, ui);

      const scaleRow = createSliderRow({
        label: "Scale factor:",
        value: ui.scaleFactor,
        min: 0.1,
        max: 20,
        step: 0.1,
        decimals: 1,
        onChange: (value) => {
          ui.scaleFactor = value;
        },
      });

      const bg = createColorField({
        value: ui.bgColor,
        alpha: true,
        onChange: (value) => {
          ui.bgColor = value;
        },
      });
      const bgRow = createLabeledRow({
        label: "Background color:",
        control: bg.root,
        spanControls: true,
      });

      const crop = createCheckbox({
        checked: ui.cropToContent,
        onChange: (value) => {
          ui.cropToContent = value;
        },
      });
      const cropRow = createLabeledRow({
        label: "Crop to content:",
        control: crop.root,
        spanControls: true,
      });

      const iStarts = createNumberField({
        value: ui.iParam,
        min: 1,
        max: 100000,
        step: 1,
        onChange: (value) => {
          ui.iParam = Math.min(100000, Math.max(1, Math.round(value || 1)));
        },
      });
      const iRow = createLabeledRow({
        label: "%i starts from:",
        control: iStarts.root,
        spanControls: true,
      });

      const path = createPathField({
        value: ui.filename,
        placeholder: "path template",
        onInput: (value) => {
          ui.filename = value;
        },
        onBrowse: async () => {
          const selected = await context.fs.showSaveDialog({
            defaultPath: ui.filename,
            filters: IMAGE_FILTERS,
          });
          if (selected) ui.filename = selected;
          return selected;
        },
      });

      const save = createButton({
        label: "Save",
        onClick: () => {
          if (disposed || saving || context.signal.aborted) return;
          saving = true;
          save.setDisabled(true);
          void context
            .dispatch({
              type: ImageCommand.Save,
              payload: {
                t_filename: ui.filename,
                scale_factor: ui.scaleFactor,
                bg_color: hexToRgba(ui.bgColor ?? "#00000000", 0),
                crop_to_content: ui.cropToContent,
                i_param: ui.iParam,
              },
            })
            .finally(() => {
              saving = false;
              if (!disposed) save.setDisabled(false);
            });
        },
      });

      form.append(
        scaleRow.root,
        bgRow.root,
        cropRow.root,
        iRow.root,
        wrapFullWidth(path.root),
        wrapFullWidth(save.root),
      );
      surface.root.appendChild(form);

      return () => {
        disposed = true;
        scaleRow.destroy();
        bg.destroy();
        crop.destroy();
        iStarts.destroy();
        path.destroy();
        save.destroy();
        bgRow.destroy();
        cropRow.destroy();
        iRow.destroy();
      };
    },
  };
}
