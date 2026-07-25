// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Handwritten WASM bindings until generated wasm-bindgen types are used.
 * TODO: move interfaces out of plugin.ts; add free() and snapshot/setter methods.
 */

export interface AtomInfo {
  symbol: string;
  tag: number;
}

export interface CoordinateAxesState {
  visible: boolean;
  labels_visible: boolean;
  both_directions: boolean;
  use_origin: boolean;
  // TODO: length, thickness, colors, label texts, auto-adjust
}

export interface VisualizerState {
  coordinate_axes: CoordinateAxesState;
  // TODO: view, atom_labels, appearance, cubes_and_surfaces
}

export interface MolecularVisualizerInstance {
  resize(width: number, height: number): void;
  scale_scene(factor: number): void;
  rotate_scene(pitch: number, yaw: number, roll: number): void;
  new_cursor_position(x: number, y: number): Promise<AtomInfo | null>;
  toggle_atom_selection(x: number, y: number): Promise<void>;
  toggle_projection(): Promise<void>;
  get_state(): VisualizerState;
  set_coordinate_axes_visible(value: boolean): Promise<void>;
  set_coordinate_axes_labels_visible(value: boolean): Promise<void>;
  set_coordinate_axes_both_directions(value: boolean): Promise<void>;
  set_coordinate_axes_use_origin(value: boolean): Promise<void>;
  render(): void;
  // TODO: free(): void;
}

export interface WasmModule {
  default: (wasm_url: URL) => Promise<void>;
  MolecularVisualizer: {
    create(
      canvas: HTMLCanvasElement,
      node_type: string,
      data: Uint8Array,
    ): Promise<MolecularVisualizerInstance>;
  };
}
