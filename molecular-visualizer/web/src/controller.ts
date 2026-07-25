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
  SetSceneRotation: "view.set_scene_rotation",
  SetSceneScale: "view.set_scene_scale",
} as const;

export const AtomLabelsCommand = {
  SetSymbolVisible: "atom_labels.set_symbol_visible",
  SetNumberVisible: "atom_labels.set_number_visible",
  SetSize: "atom_labels.set_size",
  SetOffset: "atom_labels.set_offset",
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
