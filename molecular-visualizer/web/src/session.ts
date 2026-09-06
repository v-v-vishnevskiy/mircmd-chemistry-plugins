// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * ProgramSession for molecular visualizer.
 */

import { createControlPanelContribution } from "./control_panel";
import { readRgba } from "./control_panel/color_utils";
import { createPanelUi } from "./control_panel/panel_ui";
import {
  ImageCommand,
  type MolecularVisualizerController,
} from "./controller";
import type {
  ControlPanelContribution,
  ProgramCommand,
  ProgramCommandContext,
  ProgramPluginContext,
  ProgramSession,
} from "./program_context";

const FILE_NAME_SANITIZE_RE = /[^\w _-]|(\s)(?=\1+)/g;

function formatError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error && error.message) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message: unknown }).message;
    if (typeof message === "string" && message) return message;
  }
  return String(error);
}

export class MolecularVisualizerSession implements ProgramSession {
  private disposed = false;
  readonly controlPanel: ControlPanelContribution;

  constructor(
    private readonly controller: MolecularVisualizerController,
    private readonly cleanup: () => void | Promise<void>,
    private readonly pluginContext: ProgramPluginContext,
  ) {
    this.controlPanel = createControlPanelContribution(controller, createPanelUi());
  }

  async execute(command: ProgramCommand, context: ProgramCommandContext): Promise<void> {
    if (this.disposed) return;
    if (command.type === ImageCommand.Save) {
      await this.saveImage(command.payload, context);
      return;
    }
    await this.controller.execute(command);
  }

  async dispose(): Promise<void> {
    if (this.disposed) return;
    this.disposed = true;
    try {
      await this.cleanup();
    } finally {
      await this.controller.dispose();
    }
  }

  private async saveImage(payload: unknown, context: ProgramCommandContext): Promise<void> {
    const options = readSavePayload(payload);
    const path = expandFilename(
      options.filename,
      this.pluginContext.node.name,
      options.iParam,
      context.instanceIndex,
    );
    try {
      const image = await this.controller.exportImage(
        options.scaleFactor,
        options.bgColor,
        options.cropToContent,
      );
      const bytes = await encodeImage(image.width, image.height, image.data, path);
      await this.pluginContext.fs.writeFile(path, bytes);
      this.pluginContext.log.info(`${path} saved successfully`);
    } catch (error) {
      this.pluginContext.log.error(`Error saving image ${path}: ${formatError(error)}`);
    }
  }
}

function readSavePayload(payload: unknown): {
  filename: string;
  scaleFactor: number;
  bgColor: [number, number, number, number];
  cropToContent: boolean;
  iParam: number;
} {
  const data =
    payload && typeof payload === "object"
      ? (payload as {
          t_filename?: unknown;
          scale_factor?: unknown;
          bg_color?: unknown;
          crop_to_content?: unknown;
          i_param?: unknown;
        })
      : {};
  return {
    filename: typeof data.t_filename === "string" ? data.t_filename : "%n_%i.png",
    scaleFactor:
      typeof data.scale_factor === "number" && Number.isFinite(data.scale_factor)
        ? data.scale_factor
        : 1,
    bgColor: readRgba(data.bg_color, 0),
    cropToContent: data.crop_to_content !== false,
    iParam: typeof data.i_param === "number" ? data.i_param : 1,
  };
}

function sanitizeFilename(value: string): string {
  return value
    .trim()
    .replace(/[./]/g, "_")
    .replace(/ /g, "_")
    .replace(FILE_NAME_SANITIZE_RE, "");
}

function expandFilename(
  template: string,
  nodeName: string,
  iParam: number,
  instanceIndex: number,
): string {
  const index = String(iParam + instanceIndex).padStart(6, "0");
  return template.replaceAll("%n", sanitizeFilename(nodeName)).replaceAll("%i", index);
}

function mimeFromPath(path: string): string {
  const lower = path.toLowerCase();
  if (lower.endsWith(".jpg") || lower.endsWith(".jpeg")) {
    return "image/jpeg";
  }
  return "image/png";
}

async function encodeImage(
  width: number,
  height: number,
  data: Uint8Array,
  path: string,
): Promise<Uint8Array> {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Failed to create a 2D canvas context");
  }
  const pixels = new Uint8ClampedArray(data);
  context.putImageData(new ImageData(pixels, width, height), 0, 0);
  const blob = await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((result) => {
      if (result) resolve(result);
      else reject(new Error("Failed to encode image"));
    }, mimeFromPath(path));
  });
  return new Uint8Array(await blob.arrayBuffer());
}
