// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Cubes and surfaces control block (stub reactions + local tree state).
 */

import {
  controlsStyles,
  createButton,
  createCheckbox,
  createColorField,
  createNumberField,
  createTree,
  type TreeNode,
} from "@mircmd/ui-controls";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import {
  VolumeCubeCommand,
  type MolecularVisualizerController,
} from "../controller";
import styles from "./styles.css";

type SurfaceEntry = {
  id: string;
  inverted: boolean;
  color: string;
  visible: boolean;
};

type SurfaceGroup = {
  id: string;
  value: number;
  visible: boolean;
  surfaces: SurfaceEntry[];
};

function hexToRgba(hex: string): [number, number, number, number] {
  const raw = hex.replace("#", "");
  const r = Number.parseInt(raw.slice(0, 2), 16) / 255;
  const g = Number.parseInt(raw.slice(2, 4), 16) / 255;
  const b = Number.parseInt(raw.slice(4, 6), 16) / 255;
  return [r, g, b, 200 / 255];
}

function groupsToTreeNodes(groups: SurfaceGroup[]): TreeNode[] {
  return groups.map((group) => {
    if (group.surfaces.length === 1) {
      const only = group.surfaces[0]!;
      const suffix = only.inverted ? " (inverted)" : " (original)";
      return {
        id: group.id,
        label: `${group.value}${suffix}`,
        expanded: true,
      };
    }
    return {
      id: group.id,
      label: String(group.value),
      expanded: true,
      children: group.surfaces.map((surface) => ({
        id: surface.id,
        label: surface.inverted ? "inverted" : "original",
        expanded: true,
      })),
    };
  });
}

function findSurface(
  groups: SurfaceGroup[],
  id: string,
): { group: SurfaceGroup; surface: SurfaceEntry | null } | null {
  for (const group of groups) {
    if (group.id === id) return { group, surface: null };
    for (const surface of group.surfaces) {
      if (surface.id === id) return { group, surface };
    }
  }
  return null;
}

export function createCubesAndSurfacesBlock(
  _controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "cubes_and_surfaces",
    title: "Cubes and surfaces",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      let value = 0.05;
      let color1 = "#ff0000";
      let color2 = "#0000ff";
      let inverse = false;
      let groups: SurfaceGroup[] = [];
      let nextId = 1;

      const stubDispatch = (type: string, payload: unknown) => {
        if (context.signal.aborted) return;
        void context.dispatch({ type, payload });
      };

      const valueField = createNumberField({
        value,
        min: -1000,
        max: 1000,
        step: 0.01,
        onChange: (v) => {
          value = v;
        },
      });

      const colorField1 = createColorField({
        value: color1,
        onChange: (v) => {
          color1 = v;
        },
      });

      const colorField2 = createColorField({
        value: color2,
        disabled: true,
        onChange: (v) => {
          color2 = v;
        },
      });

      const refreshTree = () => {
        tree.setNodes(groupsToTreeNodes(groups));
      };

      const tree = createTree({
        nodes: [],
        selectable: false,
        indentSize: 20,
        renderTrailing: (node) => {
          const found = findSurface(groups, node.id);
          if (!found) return null;

          const trailing = document.createElement("div");
          trailing.className = "cp-tree-actions";

          const isGroupOnlyChild =
            found.surface === null && found.group.surfaces.length === 1;
          const showColor =
            found.surface !== null || isGroupOnlyChild || found.group.surfaces.length === 1;

          // Color on leaf surfaces, or on single-surface group row.
          if (found.surface || isGroupOnlyChild) {
            const target = found.surface ?? found.group.surfaces[0]!;
            if (showColor) {
              const color = createColorField({
                value: target.color,
                onChange: (hex) => {
                  target.color = hex;
                  stubDispatch(VolumeCubeCommand.SetIsosurfaceColor, {
                    id: target.id,
                    color: hexToRgba(hex),
                  });
                },
              });
              trailing.appendChild(color.root);
            }
          }

          const visibleTargetId =
            found.surface?.id ?? found.group.id;
          const visible =
            found.surface?.visible ?? found.group.visible;

          const visibility = createButton({
            label: visible ? "Hide" : "Show",
            onClick: () => {
              const next = !visible;
              if (found.surface) {
                found.surface.visible = next;
              } else {
                found.group.visible = next;
                for (const child of found.group.surfaces) {
                  child.visible = next;
                }
              }
              stubDispatch(VolumeCubeCommand.SetIsosurfaceVisible, {
                id: visibleTargetId,
                visible: next,
                apply_to_children: true,
                ...(next ? { apply_to_parents: true } : {}),
              });
              refreshTree();
            },
          });
          visibility.root.classList.add("cp-icon-button");

          const del = createButton({
            label: "×",
            onClick: () => {
              const removeId = found.surface?.id ?? found.group.id;
              if (found.surface) {
                found.group.surfaces = found.group.surfaces.filter(
                  (s) => s.id !== found.surface!.id,
                );
                if (found.group.surfaces.length === 0) {
                  groups = groups.filter((g) => g.id !== found.group.id);
                }
              } else {
                groups = groups.filter((g) => g.id !== found.group.id);
              }
              stubDispatch(VolumeCubeCommand.RemoveIsosurface, { id: removeId });
              refreshTree();
            },
          });
          del.root.classList.add("cp-icon-button");

          trailing.append(visibility.root, del.root);
          return trailing;
        },
      });

      const add = createButton({
        label: "Add",
        onClick: () => {
          if (context.signal.aborted) return;
          const groupId = `group-${nextId++}`;
          const uniqueId = nextId++;
          const surfaces: SurfaceEntry[] = [
            {
              id: `surf-${uniqueId}`,
              inverted: false,
              color: color1,
              visible: true,
            },
          ];
          if (inverse) {
            surfaces.push({
              id: `surf-${nextId++}`,
              inverted: true,
              color: color2,
              visible: true,
            });
          }
          groups = [
            ...groups,
            {
              id: groupId,
              value,
              visible: true,
              surfaces,
            },
          ];
          stubDispatch(VolumeCubeCommand.AddIsosurface, {
            value,
            color_1: hexToRgba(color1),
            color_2: hexToRgba(color2),
            inverse,
            unique_id: uniqueId,
          });
          refreshTree();
        },
      });

      const inverseBox = createCheckbox({
        label: "Inverse",
        checked: false,
        onChange: (checked) => {
          inverse = checked;
          colorField2.setDisabled(!checked);
        },
      });

      const form = document.createElement("div");
      form.className = "cp-form cp-form-volume";

      const topRow = document.createElement("div");
      topRow.className = "cp-form-row";
      topRow.append(valueField.root, colorField1.root, add.root);

      const secondRow = document.createElement("div");
      secondRow.className = "cp-form-row";
      secondRow.append(inverseBox.root, colorField2.root);

      const treeWrap = document.createElement("div");
      treeWrap.className = "cp-isosurface-tree";
      treeWrap.appendChild(tree.root);

      form.append(topRow, secondRow);

      const stack = document.createElement("div");
      stack.className = "cp-stack";
      stack.append(form, treeWrap);
      surface.root.appendChild(stack);

      return () => {
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
