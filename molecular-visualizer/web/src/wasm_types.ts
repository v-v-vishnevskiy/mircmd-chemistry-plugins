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

export interface Rgba {
  r: number;
  g: number;
  b: number;
  a: number;
}

export interface TransformState {
  pitch: number;
  yaw: number;
  roll: number;
  scale: number;
  perspective: boolean;
}

export interface AtomLabelsState {
  symbol_visible: boolean;
  number_visible: boolean;
  size: number;
  offset: number;
}

export interface SurfaceState {
  id: number;
  inverted: boolean;
  visible: boolean;
  color: Rgba;
}

export interface SurfaceGroupState {
  id: number;
  value: number;
  visible: boolean;
  readonly surfaces: SurfaceState[];
}

export interface CubesAndSurfacesState {
  available: boolean;
  readonly groups: SurfaceGroupState[];
}

export interface CoordinateAxisState {
  color: Rgba;
  label_color: Rgba;
  label: string;
}

export interface CoordinateAxesState {
  visible: boolean;
  labels_visible: boolean;
  both_directions: boolean;
  use_origin: boolean;
  length: number;
  thickness: number;
  font_size: number;
  auto_adjust_available: boolean;
  x: CoordinateAxisState;
  y: CoordinateAxisState;
  z: CoordinateAxisState;
}

export interface AppearanceState {
  background: Rgba;
  style: string;
}

export interface VisualizerState {
  transform: TransformState;
  atom_labels: AtomLabelsState;
  cubes_and_surfaces: CubesAndSurfacesState;
  coordinate_axes: CoordinateAxesState;
  appearance: AppearanceState;
}

export interface MolecularVisualizerInstance {
  resize(width: number, height: number): void;
  scale_scene(factor: number): void;
  rotate_scene(pitch: number, yaw: number, roll: number): void;
  set_scene_rotation(pitch: number, yaw: number, roll: number): void;
  set_scene_scale(factor: number): void;
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
