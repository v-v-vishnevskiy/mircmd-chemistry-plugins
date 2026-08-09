// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Single mutation point for visualizer state (canvas, context menu, Control Panel).
 */

import type { ProgramCommand } from "./program_context";
import type { MolecularVisualizerInstance, VisualizerState } from "./wasm_types";

export type StateChangeListener = (snapshot: VisualizerState, changedBlocks: string[]) => void;

export const AxesCommand = {
  SetVisible: "coordinate_axes.set_visible",
  SetLabelsVisible: "coordinate_axes.set_labels_visible",
  SetBothDirections: "coordinate_axes.set_both_directions",
  SetUseOrigin: "coordinate_axes.set_use_origin",
  SetLength: "coordinate_axes.set_length",
  SetThickness: "coordinate_axes.set_thickness",
  SetFontSize: "coordinate_axes.set_font_size",
  AdjustLength: "coordinate_axes.adjust_length",
  SetColor: "coordinate_axes.set_color",
  SetLabelColor: "coordinate_axes.set_label_color",
  SetText: "coordinate_axes.set_text",
} as const;

export const ViewCommand = {
  RotateScene: "view.rotate_scene",
  ScaleScene: "view.scale_scene",
  SetSceneRotation: "view.set_scene_rotation",
  SetSceneScale: "view.set_scene_scale",
} as const;

export const AtomLabelsCommand = {
  SetSymbolVisible: "atom_labels.set_symbol_visible",
  SetNumberVisible: "atom_labels.set_number_visible",
  SetSize: "atom_labels.set_size",
  SetOffset: "atom_labels.set_offset",
  SetAllVisible: "atom_labels.set_all_visible",
  SetSelectedVisible: "atom_labels.set_selected_visible",
  ToggleVisibilityForAllAtoms: "atom_labels.toggle_visibility_for_all_atoms",
  ToggleVisibilityForSelectedAtoms: "atom_labels.toggle_visibility_for_selected_atoms",
} as const;

export const AppearanceCommand = {
  SetBgColor: "appearance.set_bg_color",
  SetStyle: "appearance.set_style",
} as const;

export const ImageCommand = {
  Save: "image.save",
} as const;

export const VolumeCubeCommand = {
  AddIsosurface: "volume_cube.add_isosurface",
  SetIsosurfaceColor: "volume_cube.set_isosurface_color",
  SetIsosurfaceVisible: "volume_cube.set_isosurface_visible",
  RemoveIsosurface: "volume_cube.remove_isosurface",
} as const;

function readBoolPayload(payload: unknown): boolean {
  if (typeof payload === "boolean") return payload;
  if (payload && typeof payload === "object" && "value" in payload) {
    return Boolean((payload as { value: unknown }).value);
  }
  return Boolean(payload);
}

function readNumber(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function readRotationPayload(payload: unknown): { pitch: number; yaw: number; roll: number } {
  if (payload && typeof payload === "object") {
    const data = payload as { pitch?: unknown; yaw?: unknown; roll?: unknown };
    return {
      pitch: readNumber(data.pitch),
      yaw: readNumber(data.yaw),
      roll: readNumber(data.roll),
    };
  }
  return { pitch: 0, yaw: 0, roll: 0 };
}

function readFactorPayload(payload: unknown): number {
  if (typeof payload === "number") return payload;
  if (payload && typeof payload === "object" && "factor" in payload) {
    return readNumber((payload as { factor: unknown }).factor, 1);
  }
  return 1;
}

function readValuePayload(payload: unknown, fallback = 0): number {
  if (typeof payload === "number") return readNumber(payload, fallback);
  if (payload && typeof payload === "object" && "value" in payload) {
    return readNumber((payload as { value: unknown }).value, fallback);
  }
  return fallback;
}

function readAxisColorPayload(payload: unknown): {
  axis: "x" | "y" | "z";
  color: [number, number, number, number];
} {
  const data =
    payload && typeof payload === "object"
      ? (payload as { axis?: unknown; color?: unknown })
      : {};
  const axisRaw = typeof data.axis === "string" ? data.axis.toLowerCase() : "x";
  const axis: "x" | "y" | "z" =
    axisRaw === "y" || axisRaw === "z" ? axisRaw : "x";
  const color = Array.isArray(data.color) ? data.color : [];
  return {
    axis,
    color: [
      readNumber(color[0]),
      readNumber(color[1]),
      readNumber(color[2]),
      readNumber(color[3], 1),
    ],
  };
}

function readAxisTextPayload(payload: unknown): {
  axis: "x" | "y" | "z";
  text: string;
} {
  const data =
    payload && typeof payload === "object"
      ? (payload as { axis?: unknown; text?: unknown })
      : {};
  const axisRaw = typeof data.axis === "string" ? data.axis.toLowerCase() : "x";
  const axis: "x" | "y" | "z" =
    axisRaw === "y" || axisRaw === "z" ? axisRaw : "x";
  return {
    axis,
    text: typeof data.text === "string" ? data.text : "",
  };
}

export class MolecularVisualizerController {
  private disposed = false;
  private readonly listeners = new Set<StateChangeListener>();
  private chain: Promise<void> = Promise.resolve();

  constructor(private readonly visualizer: MolecularVisualizerInstance) {}

  getSnapshot(): VisualizerState {
    return this.visualizer.get_state();
  }

  subscribe(listener: StateChangeListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  async execute(command: ProgramCommand): Promise<void> {
    if (this.disposed) return;

    await this.enqueue(async () => {
      if (this.disposed) return;

      switch (command.type) {
        case AxesCommand.SetVisible:
          await this.visualizer.set_coordinate_axes_visible(readBoolPayload(command.payload));
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.SetLabelsVisible:
          await this.visualizer.set_coordinate_axes_labels_visible(
            readBoolPayload(command.payload),
          );
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.SetBothDirections:
          await this.visualizer.set_coordinate_axes_both_directions(
            readBoolPayload(command.payload),
          );
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.SetUseOrigin:
          await this.visualizer.set_coordinate_axes_use_origin(readBoolPayload(command.payload));
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.SetLength:
          await this.visualizer.set_coordinate_axes_length(readValuePayload(command.payload));
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.AdjustLength:
          await this.visualizer.adjust_coordinate_axes_length();
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.SetThickness:
          await this.visualizer.set_coordinate_axes_thickness(readValuePayload(command.payload));
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.SetFontSize:
          await this.visualizer.set_coordinate_axes_labels_size(
            readValuePayload(command.payload) / 100,
          );
          this.notify(["coordinate_axes"]);
          break;
        case AxesCommand.SetColor: {
          const { axis, color } = readAxisColorPayload(command.payload);
          await this.visualizer.set_coordinate_axis_color(
            axis,
            color[0],
            color[1],
            color[2],
            color[3],
          );
          this.notify(["coordinate_axes"]);
          break;
        }
        case AxesCommand.SetLabelColor: {
          const { axis, color } = readAxisColorPayload(command.payload);
          await this.visualizer.set_coordinate_axis_label_color(
            axis,
            color[0],
            color[1],
            color[2],
            color[3],
          );
          this.notify(["coordinate_axes"]);
          break;
        }
        case AxesCommand.SetText: {
          const { axis, text } = readAxisTextPayload(command.payload);
          await this.visualizer.set_coordinate_axis_text(axis, text);
          this.notify(["coordinate_axes"]);
          break;
        }
        case ViewCommand.RotateScene: {
          const { pitch, yaw, roll } = readRotationPayload(command.payload);
          this.visualizer.rotate_scene(pitch, yaw, roll);
          this.notify(["view"]);
          break;
        }
        case ViewCommand.ScaleScene:
          this.visualizer.scale_scene(readFactorPayload(command.payload));
          this.notify(["view"]);
          break;
        case ViewCommand.SetSceneRotation: {
          const { pitch, yaw, roll } = readRotationPayload(command.payload);
          this.visualizer.set_scene_rotation(pitch, yaw, roll);
          this.notify(["view"]);
          break;
        }
        case ViewCommand.SetSceneScale:
          this.visualizer.set_scene_scale(readFactorPayload(command.payload));
          this.notify(["view"]);
          break;
        case AtomLabelsCommand.SetSymbolVisible:
          this.visualizer.set_atom_labels_symbol_visible(readBoolPayload(command.payload));
          this.notify(["atom_labels"]);
          break;
        case AtomLabelsCommand.SetNumberVisible:
          this.visualizer.set_atom_labels_number_visible(readBoolPayload(command.payload));
          this.notify(["atom_labels"]);
          break;
        case AtomLabelsCommand.SetSize:
          this.visualizer.set_atom_labels_size(readValuePayload(command.payload));
          this.notify(["atom_labels"]);
          break;
        case AtomLabelsCommand.SetOffset:
          this.visualizer.set_atom_labels_offset(readValuePayload(command.payload));
          this.notify(["atom_labels"]);
          break;
        case AtomLabelsCommand.SetAllVisible:
          this.visualizer.set_all_atom_labels_visible(readBoolPayload(command.payload));
          this.notify(["atom_labels"]);
          break;
        case AtomLabelsCommand.SetSelectedVisible:
          this.visualizer.set_selected_atom_labels_visible(readBoolPayload(command.payload));
          this.notify(["atom_labels"]);
          break;
        case AtomLabelsCommand.ToggleVisibilityForAllAtoms:
          this.visualizer.toggle_all_atom_labels_visible();
          this.notify(["atom_labels"]);
          break;
        case AtomLabelsCommand.ToggleVisibilityForSelectedAtoms:
          this.visualizer.toggle_selected_atom_labels_visible();
          this.notify(["atom_labels"]);
          break;
        default:
          break;
      }
    });
  }

  resize(width: number, height: number): void {
    if (this.disposed) return;
    this.visualizer.resize(width, height);
  }

  async rotateScene(pitch: number, yaw: number, roll: number): Promise<void> {
    if (this.disposed) return;
    this.visualizer.rotate_scene(pitch, yaw, roll);
    this.notify(["view"]);
  }

  async scaleScene(factor: number): Promise<void> {
    if (this.disposed) return;
    this.visualizer.scale_scene(factor);
    this.notify(["view"]);
  }

  async toggleAtomSelection(x: number, y: number): Promise<void> {
    if (this.disposed) return;
    await this.enqueue(async () => {
      if (this.disposed) return;
      await this.visualizer.toggle_atom_selection(x, y);
    });
  }

  async newCursorPosition(x: number, y: number) {
    if (this.disposed) return null;
    return this.visualizer.new_cursor_position(x, y);
  }

  async toggleProjection(): Promise<void> {
    if (this.disposed) return;
    await this.enqueue(async () => {
      if (this.disposed) return;
      await this.visualizer.toggle_projection();
    });
  }

  async dispose(): Promise<void> {
    if (this.disposed) return;
    this.disposed = true;
    this.listeners.clear();
    // WASM free() is not exported yet.
  }

  private notify(changedBlocks: string[]): void {
    if (this.disposed) return;
    const snapshot = this.getSnapshot();
    for (const listener of this.listeners) {
      listener(snapshot, changedBlocks);
    }
  }

  private enqueue(task: () => Promise<void>): Promise<void> {
    const next = this.chain.then(task, task);
    this.chain = next.then(
      () => undefined,
      () => undefined,
    );
    return next;
  }
}
