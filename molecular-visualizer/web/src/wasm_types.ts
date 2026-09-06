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
  readonly style_names: string[];
}

export interface RenderedImage {
  width: number;
  height: number;
  readonly data: Uint8Array;
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
  set_atom_labels_symbol_visible(value: boolean): void;
  set_atom_labels_number_visible(value: boolean): void;
  set_atom_labels_size(value: number): void;
  set_atom_labels_offset(value: number): void;
  set_all_atom_labels_visible(value: boolean): void;
  set_selected_atom_labels_visible(value: boolean): void;
  toggle_all_atom_labels_visible(): void;
  toggle_selected_atom_labels_visible(): void;
  start_pick(x: number, y: number): Promise<number>;
  apply_hover(atom_index: number): AtomInfo | undefined;
  apply_selection(atom_index: number): void;
  toggle_projection(): void;
  get_state(): VisualizerState;
  width(): number;
  height(): number;
  add_isosurface(
    value: number,
    r1: number,
    g1: number,
    b1: number,
    a1: number,
    r2: number,
    g2: number,
    b2: number,
    a2: number,
    inverse: boolean,
  ): void;
  set_isosurface_color(id: number, r: number, g: number, b: number, a: number): void;
  set_isosurface_visible(
    id: number,
    visible: boolean,
    apply_to_children: boolean,
    apply_to_parents: boolean,
  ): void;
  remove_isosurface(id: number): void;
  set_background_color(r: number, g: number, b: number, a: number): void;
  set_style(name: string): void;
  set_next_style(): void;
  set_prev_style(): void;
  render_to_image(
    width: number,
    height: number,
    r: number,
    g: number,
    b: number,
    a: number,
    crop: boolean,
  ): Promise<RenderedImage>;
  set_coordinate_axes_visible(value: boolean): void;
  set_coordinate_axes_labels_visible(value: boolean): void;
  set_coordinate_axes_both_directions(value: boolean): void;
  set_coordinate_axes_use_origin(value: boolean): void;
  set_coordinate_axes_length(value: number): void;
  adjust_coordinate_axes_length(): void;
  set_coordinate_axes_thickness(value: number): void;
  set_coordinate_axes_labels_size(value: number): void;
  set_coordinate_axis_color(
    axis: string,
    r: number,
    g: number,
    b: number,
    a: number,
  ): void;
  set_coordinate_axis_label_color(
    axis: string,
    r: number,
    g: number,
    b: number,
    a: number,
  ): void;
  set_coordinate_axis_text(axis: string, text: string): void;
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
