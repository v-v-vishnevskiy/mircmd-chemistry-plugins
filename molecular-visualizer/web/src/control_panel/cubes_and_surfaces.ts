// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Cubes and surfaces control block.
 */

import {
  applyControlStyles,
  createButton,
  createCheckbox,
  createColorField,
  createIconButton,
  createNumberField,
  createTree,
  formatNumber,
  type ColorFieldControl,
  type IconButtonControl,
  type TreeNode,
  type TreeTrailing,
} from "@mircmd/ui-controls";
import {
  VolumeCubeCommand,
  type MolecularVisualizerController,
} from "../controller";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import type {
  SurfaceGroupState,
  SurfaceState,
  VisualizerState,
} from "../wasm_types";
import { hexToRgba, rgbaToHex } from "./color_utils";
import type { CubesPanelUi } from "./panel_ui";
import styles from "./styles.css";

function listGroups(snapshot: VisualizerState): SurfaceGroupState[] {
  return snapshot.cubes_and_surfaces.groups ?? [];
}

function withExpanded(
  node: TreeNode,
  expandedById: Record<string, boolean>,
): TreeNode {
  if (Object.prototype.hasOwnProperty.call(expandedById, node.id)) {
    node.expanded = expandedById[node.id];
  }
  return node;
}

function groupsToTreeNodes(
  groups: SurfaceGroupState[],
  expandedById: Record<string, boolean>,
): TreeNode[] {
  return groups.map((group) => {
    if (group.surfaces.length === 1) {
      const only = group.surfaces[0]!;
      const suffix = only.inverted ? " (inverted)" : " (original)";
      return withExpanded(
        { id: String(group.id), label: `${formatNumber(group.value)}${suffix}` },
        expandedById,
      );
    }
    return withExpanded(
      {
        id: String(group.id),
        label: formatNumber(group.value),
        children: group.surfaces.map((surface) =>
          withExpanded(
            {
              id: String(surface.id),
              label: surface.inverted ? "inverted" : "original",
            },
            expandedById,
          ),
        ),
      },
      expandedById,
    );
  });
}

function findSurface(
  groups: SurfaceGroupState[],
  id: number,
): { group: SurfaceGroupState; surface: SurfaceState | null } | null {
  for (const group of groups) {
    if (group.id === id) return { group, surface: null };
    for (const surface of group.surfaces) {
      if (surface.id === id) return { group, surface };
    }
  }
  return null;
}

function colorTarget(
  found: { group: SurfaceGroupState; surface: SurfaceState | null },
): SurfaceState | null {
  if (found.surface) return found.surface;
  if (found.group.surfaces.length === 1) return found.group.surfaces[0]!;
  return null;
}

function syncColorField(
  color: ColorFieldControl,
  target: SurfaceState,
  currentHex: { value: string },
): void {
  const hex = rgbaToHex(target.color, true);
  if (hex === currentHex.value) return;
  currentHex.value = hex;
  color.setValue(hex);
}

function syncVisibility(
  button: IconButtonControl,
  visible: boolean,
  current: { value: boolean },
): void {
  if (visible === current.value) return;
  current.value = visible;
  button.setIcon(visible ? "visibility" : "visibility-off");
  button.setLabel(visible ? "Hide" : "Show");
}

function createTreeActions(
  node: TreeNode,
  getGroups: () => SurfaceGroupState[],
  dispatch: (type: string, payload: unknown) => void,
): TreeTrailing | null {
  const found = findSurface(getGroups(), Number(node.id));
  if (!found) return null;

  const trailing = document.createElement("div");
  trailing.className = "mircmd-tree-actions";
  const target = colorTarget(found);
  const hexState = { value: target ? rgbaToHex(target.color, true) : "" };
  const color = target
    ? createColorField({
        value: hexState.value,
        alpha: true,
        onChange: (hex) => {
          const next = colorTarget(findSurface(getGroups(), Number(node.id)) ?? found);
          if (!next) return;
          dispatch(VolumeCubeCommand.SetIsosurfaceColor, {
            id: next.id,
            color: hexToRgba(hex, next.color.a),
          });
        },
      })
    : null;
  if (color) trailing.appendChild(color.root);

  const visibleState = { value: found.surface?.visible ?? found.group.visible };
  const visibility = createIconButton({
    icon: visibleState.value ? "visibility" : "visibility-off",
    label: visibleState.value ? "Hide" : "Show",
    onClick: () => {
      const current = findSurface(getGroups(), Number(node.id));
      if (!current) return;
      const isOn = current.surface?.visible ?? current.group.visible;
      dispatch(VolumeCubeCommand.SetIsosurfaceVisible, {
        id: current.surface?.id ?? current.group.id,
        visible: !isOn,
        apply_to_children: true,
        ...(!isOn ? { apply_to_parents: true } : {}),
      });
    },
  });
  const del = createIconButton({
    icon: "backspace",
    label: "Delete",
    onClick: () => {
      const current = findSurface(getGroups(), Number(node.id));
      if (!current) return;
      dispatch(VolumeCubeCommand.RemoveIsosurface, {
        id: current.surface?.id ?? current.group.id,
      });
    },
  });
  trailing.append(visibility.root, del.root);

  return {
    root: trailing,
    update(nextNode) {
      const current = findSurface(getGroups(), Number(nextNode.id));
      if (!current) return;
      const nextColor = colorTarget(current);
      if (color && nextColor) syncColorField(color, nextColor, hexState);
      syncVisibility(visibility, current.surface?.visible ?? current.group.visible, visibleState);
    },
    destroy() {
      color?.destroy();
      visibility.destroy();
      del.destroy();
      trailing.remove();
    },
  };
}

export function createCubesAndSurfacesBlock(
  controller: MolecularVisualizerController,
  ui: CubesPanelUi,
): ControlPanelBlock {
  return {
    id: "cubes_and_surfaces",
    title: "Cubes and surfaces",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      applyControlStyles(surface, styles);

      let disposed = false;
      let applying = false;
      let groups: SurfaceGroupState[] = [];

      const dispatch = (type: string, payload: unknown) => {
        if (disposed || applying || context.signal.aborted) return;
        void context.dispatch({ type, payload });
      };

      const valueField = createNumberField({
        value: ui.value,
        min: -1000,
        max: 1000,
        step: 0.01,
        onChange: (next) => {
          ui.value = next;
        },
      });
      const colorField1 = createColorField({
        value: ui.color1,
        alpha: true,
        onChange: (hex) => {
          ui.color1 = hex;
        },
      });
      const colorField2 = createColorField({
        value: ui.color2,
        alpha: true,
        disabled: !ui.inverse,
        onChange: (hex) => {
          ui.color2 = hex;
        },
      });

      const tree = createTree({
        nodes: [],
        selectable: false,
        indentSize: 20,
        renderTrailing: (node) => createTreeActions(node, () => groups, dispatch),
        onToggle: (node, expanded) => {
          ui.expandedById[node.id] = expanded;
        },
      });

      const add = createButton({
        label: "Add",
        onClick: () => {
          dispatch(VolumeCubeCommand.AddIsosurface, {
            value: ui.value,
            color_1: hexToRgba(ui.color1, 200 / 255),
            color_2: hexToRgba(ui.color2, 200 / 255),
            inverse: ui.inverse,
          });
        },
      });
      const inverseBox = createCheckbox({
        label: "Inverse",
        checked: ui.inverse,
        onChange: (checked) => {
          ui.inverse = checked;
          colorField2.setDisabled(!checked);
        },
      });

      const form = document.createElement("div");
      form.className = "mircmd-form cp-form-volume";
      const topRow = document.createElement("div");
      topRow.className = "mircmd-form-row";
      topRow.append(valueField.root, colorField1.root, add.root);
      const secondRow = document.createElement("div");
      secondRow.className = "mircmd-form-row";
      secondRow.append(inverseBox.root, colorField2.root);
      form.append(topRow, secondRow);

      const treeWrap = document.createElement("div");
      treeWrap.className = "cp-isosurface-tree";
      treeWrap.appendChild(tree.root);

      const stack = document.createElement("div");
      stack.className = "mircmd-stack";
      stack.append(form, treeWrap);
      surface.root.appendChild(stack);

      const applySnapshot = (snapshot: VisualizerState) => {
        if (disposed || context.signal.aborted) return;
        applying = true;
        try {
          const available = snapshot.cubes_and_surfaces.available;
          stack.classList.toggle("disabled", !available);
          valueField.setDisabled(!available);
          colorField1.setDisabled(!available);
          colorField2.setDisabled(!available || !ui.inverse);
          add.setDisabled(!available);
          inverseBox.setDisabled(!available);
          tree.setDisabled(!available);
          groups = listGroups(snapshot);
          tree.setNodes(groupsToTreeNodes(groups, ui.expandedById));
        } finally {
          applying = false;
        }
      };

      const snapshot = await controller.getSnapshot();
      applySnapshot(snapshot);
      const unsubscribe = controller.subscribe((snapshot, changedBlocks) => {
        if (disposed || context.signal.aborted) return;
        if (changedBlocks.length === 0 || changedBlocks.includes("cubes_and_surfaces")) {
          applySnapshot(snapshot);
        }
      });

      return () => {
        disposed = true;
        unsubscribe();
        valueField.destroy();
        colorField1.destroy();
        colorField2.destroy();
        add.destroy();
        inverseBox.destroy();
        tree.destroy();
      };
    },
  };
}
