// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

import type { ProgramPluginContext, ProgramSession } from "./program_context";
import {
  MolecularVisualizerController,
  AxesCommand,
  AtomLabelsCommand,
} from "./controller";
import { MolecularVisualizerSession } from "./session";
import type {
  AtomInfo,
  WasmModule,
} from "./wasm_types";

let wasm_module: WasmModule | null = null;

function supportedTypes(): string[] {
  return ["mircmd:chemistry:atomic_coordinates", "mircmd:chemistry:volume_cube"];
}

async function run(
  ctx: ProgramPluginContext,
  node_type: string,
  data: Uint8Array,
): Promise<ProgramSession> {
  clear_root(ctx.root);

  if (ctx.signal.aborted) {
    return emptyDisposedSession();
  }

  if (!wasm_module) {
    const module_url = new URL("./molecular_visualizer.js", import.meta.url);
    wasm_module = (await import(module_url.href)) as WasmModule;
    if (ctx.signal.aborted) {
      return emptyDisposedSession();
    }
    const wasm_url = new URL("./molecular_visualizer_bg.wasm", import.meta.url);
    await wasm_module.default(wasm_url);
    if (ctx.signal.aborted) {
      return emptyDisposedSession();
    }
  }

  const canvas = create_canvas(ctx.root);
  const container = canvas.parentElement as HTMLElement;
  const overlay = create_overlay(container);

  const visualizer = await wasm_module.MolecularVisualizer.create(canvas, node_type, data);
  if (ctx.signal.aborted) {
    // No free() yet; drop references and return a disposed session.
    return emptyDisposedSession();
  }

  visualizer.render();
  const controller = new MolecularVisualizerController(visualizer);

  let is_dragging = false;
  let has_dragged = false;
  let is_async_busy = false;
  let last_mouse_x = 0;
  let last_mouse_y = 0;
  const rotation_sensitivity = 0.5;
  const zoom_sensitivity = 0.001;

  const resize_observer = new ResizeObserver(() => {
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    const width = Math.floor(rect.width * dpr);
    const height = Math.floor(rect.height * dpr);

    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
      controller.resize(width, height);
    }
  });
  resize_observer.observe(canvas);

  const onMouseDown = (event: MouseEvent) => {
    if (event.button === 0) {
      is_dragging = true;
      has_dragged = false;
      last_mouse_x = event.clientX;
      last_mouse_y = event.clientY;
    }
  };

  const onClick = async (event: MouseEvent) => {
    if (event.button === 0 && !has_dragged && !is_async_busy) {
      is_async_busy = true;
      try {
        const rect = canvas.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        const canvas_x = Math.floor((event.clientX - rect.left) * dpr);
        const canvas_y = Math.floor((event.clientY - rect.top) * dpr);
        await controller.toggleAtomSelection(canvas_x, canvas_y);
      } finally {
        is_async_busy = false;
      }
    }
  };

  const onContextMenu = async (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    const state = controller.getSnapshot();

    ctx.contextMenu.open({
      event,
      items: [
        {
          label: "Atom labels",
          children: [
            {
              label: "Symbol",
              checkable: true,
              checked: state.atom_labels.symbol_visible,
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.SetSymbolVisible,
                  payload: { value: !state.atom_labels.symbol_visible },
                });
              },
            },
            {
              label: "Number",
              checkable: true,
              checked: state.atom_labels.number_visible,
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.SetNumberVisible,
                  payload: { value: !state.atom_labels.number_visible },
                });
              },
            },
            { label: "", separator: true },
            {
              label: "Show all",
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.SetAllVisible,
                  payload: { value: true },
                });
              },
            },
            {
              label: "Hide all",
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.SetAllVisible,
                  payload: { value: false },
                });
              },
            },
            { label: "", separator: true },
            {
              label: "Show selected",
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.SetSelectedVisible,
                  payload: { value: true },
                });
              },
            },
            {
              label: "Hide selected",
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.SetSelectedVisible,
                  payload: { value: false },
                });
              },
            },
            { label: "", separator: true },
            {
              label: "Toggle all",
              shortcut: "L",
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.ToggleVisibilityForAllAtoms,
                  payload: {},
                });
              },
            },
            {
              label: "Toggle selected",
              shortcut: "Ctrl+L",
              action: async () => {
                await controller.execute({
                  type: AtomLabelsCommand.ToggleVisibilityForSelectedAtoms,
                  payload: {},
                });
              },
            },
          ],
        },
        {
          label: "Coordinate axes",
          children: [
            {
              label: "Show",
              checkable: true,
              checked: state.coordinate_axes.visible,
              action: async () => {
                await controller.execute({
                  type: AxesCommand.SetVisible,
                  payload: { value: !state.coordinate_axes.visible },
                });
              },
            },
            {
              label: "Labels",
              checkable: true,
              checked: state.coordinate_axes.labels_visible,
              action: async () => {
                await controller.execute({
                  type: AxesCommand.SetLabelsVisible,
                  payload: { value: !state.coordinate_axes.labels_visible },
                });
              },
            },
            {
              label: "Both directions",
              checkable: true,
              checked: state.coordinate_axes.both_directions,
              action: async () => {
                await controller.execute({
                  type: AxesCommand.SetBothDirections,
                  payload: { value: !state.coordinate_axes.both_directions },
                });
              },
            },
            {
              label: "Center",
              checkable: true,
              checked: !state.coordinate_axes.use_origin,
              action: async () => {
                await controller.execute({
                  type: AxesCommand.SetUseOrigin,
                  payload: { value: !state.coordinate_axes.use_origin },
                });
              },
            },
          ],
        },
        {
          label: "Bonds",
          children: [
            { label: "Add selected", action: () => {} },
            { label: "Remove selected", action: () => {} },
            { label: "Toggle selected", shortcut: "B", action: () => {} },
            { label: "Build dynamically...", action: () => {} },
            { label: "Rebuild all", action: () => {} },
            { label: "Rebuild default", action: () => {} },
          ],
        },
        {
          label: "Selection",
          children: [
            { label: "Select all atoms", action: () => {} },
            { label: "Toggle all atoms", shortcut: "Ctrl+A", action: () => {} },
            { label: "Toggle selected", action: () => {} },
          ],
        },
        {
          label: "Calculate",
          children: [
            { label: "Interatomic distance", action: () => {} },
            { label: "Interatomic angle", action: () => {} },
            { label: "Out-of-plane angle", action: () => {} },
            { label: "Auto parameter", shortcut: "P", action: () => {} },
            { label: "Selected fragments", action: () => {} },
          ],
        },
        {
          label: "Cloaking",
          children: [
            { label: "Cloak all selected", action: () => {} },
            { label: "Cloak all not selected", action: () => {} },
            { label: "Cloak all H atoms", action: () => {} },
            { label: "Cloak not selected H atoms", action: () => {} },
            { label: "Toggle all H atoms", shortcut: "H", action: () => {} },
            { label: "Cloak atoms by type...", action: () => {} },
            { label: "", separator: true },
            { label: "Uncloak all", action: () => {} },
          ],
        },
        { label: "Save image...", shortcut: "S", action: () => {} },
        { label: "", separator: true },
        {
          label: "Coordinates set",
          children: [
            { label: "Next", shortcut: "Ctrl+Right", action: () => {} },
            { label: "Previous", shortcut: "Ctrl+Left", action: () => {} },
          ],
        },
        { label: "", separator: true },
        {
          label: "Style",
          children: [
            { label: "Next", shortcut: "Ctrl+Down", action: () => {} },
            { label: "Previous", shortcut: "Ctrl+Up", action: () => {} },
          ],
        },
        { label: "", separator: true },
        {
          label: "Toggle projection",
          shortcut: "Ctrl+P",
          action: async () => {
            await controller.toggleProjection();
          },
        },
      ],
      data: {},
    });
  };

  const onMouseMove = async (event: MouseEvent) => {
    if (is_dragging) {
      has_dragged = true;
      const delta_x = event.clientX - last_mouse_x;
      const delta_y = event.clientY - last_mouse_y;

      last_mouse_x = event.clientX;
      last_mouse_y = event.clientY;

      const yaw = delta_x * rotation_sensitivity;
      const pitch = delta_y * rotation_sensitivity;

      await controller.rotateScene(pitch, yaw, 0);
      overlay.style.display = "none";
    } else if (!is_async_busy) {
      is_async_busy = true;
      try {
        const rect = canvas.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        const canvas_x = Math.floor((event.clientX - rect.left) * dpr);
        const canvas_y = Math.floor((event.clientY - rect.top) * dpr);
        const atom = await controller.newCursorPosition(canvas_x, canvas_y);
        const overlay_x = event.clientX - rect.left;
        const overlay_y = event.clientY - rect.top;
        update_overlay(overlay, atom, overlay_x, overlay_y, container);
      } finally {
        is_async_busy = false;
      }
    }
  };

  const onMouseUp = (event: MouseEvent) => {
    if (event.button === 0) {
      is_dragging = false;
    }
  };

  const onMouseLeave = () => {
    is_dragging = false;
    overlay.style.display = "none";
  };

  const onWheel = (event: WheelEvent) => {
    event.preventDefault();
    const factor = 1.0 - event.deltaY * zoom_sensitivity;
    void controller.scaleScene(factor);
  };

  canvas.addEventListener("mousedown", onMouseDown);
  canvas.addEventListener("click", onClick);
  canvas.addEventListener("contextmenu", onContextMenu);
  canvas.addEventListener("mousemove", onMouseMove);
  canvas.addEventListener("mouseup", onMouseUp);
  canvas.addEventListener("mouseleave", onMouseLeave);
  canvas.addEventListener("wheel", onWheel, { passive: false });

  const cleanup = () => {
    resize_observer.disconnect();
    canvas.removeEventListener("mousedown", onMouseDown);
    canvas.removeEventListener("click", onClick);
    canvas.removeEventListener("contextmenu", onContextMenu);
    canvas.removeEventListener("mousemove", onMouseMove);
    canvas.removeEventListener("mouseup", onMouseUp);
    canvas.removeEventListener("mouseleave", onMouseLeave);
    canvas.removeEventListener("wheel", onWheel);
    ctx.contextMenu.close();
    overlay.remove();
  };

  if (ctx.signal.aborted) {
    cleanup();
    await controller.dispose();
    return emptyDisposedSession();
  }

  return new MolecularVisualizerSession(controller, cleanup);
}

function emptyDisposedSession(): ProgramSession {
  return {
    execute: async () => {},
    dispose: async () => {},
  };
}

function clear_root(root: ShadowRoot): void {
  root.textContent = "";
}

function create_canvas(root: ShadowRoot): HTMLCanvasElement {
  const container = document.createElement("div");
  container.style.width = "100%";
  container.style.height = "100%";
  container.style.overflow = "hidden";
  container.style.position = "relative";

  const canvas = document.createElement("canvas");
  canvas.style.display = "block";
  canvas.style.width = "100%";
  canvas.style.height = "100%";

  container.appendChild(canvas);
  root.appendChild(container);

  const rect = container.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;

  return canvas;
}

function create_overlay(container: HTMLElement): HTMLDivElement {
  const overlay = document.createElement("div");
  overlay.style.position = "absolute";
  overlay.style.backgroundColor = "#44444499";
  overlay.style.color = "#D8D8D8";
  overlay.style.padding = "6px 10px";
  overlay.style.borderRadius = "6px";
  overlay.style.fontSize = "13px";
  overlay.style.fontFamily = "system-ui, -apple-system, sans-serif";
  overlay.style.pointerEvents = "none";
  overlay.style.display = "none";
  overlay.style.whiteSpace = "nowrap";
  overlay.style.zIndex = "1000";

  container.appendChild(overlay);
  return overlay;
}

function update_overlay(
  overlay: HTMLDivElement,
  atom: AtomInfo | null,
  x: number,
  y: number,
  container: HTMLElement,
): void {
  if (!atom) {
    overlay.style.display = "none";
    return;
  }

  overlay.textContent = `Atom: ${atom.symbol}${atom.tag}`;
  overlay.style.display = "block";

  const offset_x = 6;
  const offset_y = -6;
  const container_rect = container.getBoundingClientRect();

  let left = x + offset_x;
  let top = y + offset_y - overlay.offsetHeight;

  if (left + overlay.offsetWidth > container_rect.width) {
    left = x - offset_x - overlay.offsetWidth;
  }

  if (top < 0) {
    top = y + offset_x;
  }

  overlay.style.left = `${left}px`;
  overlay.style.top = `${top}px`;
}

export function instantiate(): {
  run: (
    ctx: ProgramPluginContext,
    node_type: string,
    data: Uint8Array,
  ) => Promise<ProgramSession>;
  supportedTypes: () => string[];
} {
  return { run, supportedTypes };
}
