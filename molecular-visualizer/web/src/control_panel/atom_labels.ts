// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Atom labels control block (stub reactions).
 */

import {
  controlsStyles,
  createButton,
  createCheckbox,
} from "@mircmd/ui-controls";
import type { Cleanup, ControlPanelBlock } from "../program_context";
import {
  AtomLabelsCommand,
  type MolecularVisualizerController,
} from "../controller";
import { createForm, createLabeledRow } from "./form_row";
import { createSliderRow } from "./slider_row";
import styles from "./styles.css";

export function createAtomLabelsBlock(
  _controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "atom_labels",
    title: "Atom labels",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      const form = createForm();

      const showGroup = document.createElement("div");
      showGroup.className = "cp-inline-group";
      const symbol = createCheckbox({
        label: "Symbol",
        checked: true,
        onChange: (value) => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.SetSymbolVisible,
            payload: { value },
          });
        },
      });
      const number = createCheckbox({
        label: "Number",
        checked: true,
        onChange: (value) => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.SetNumberVisible,
            payload: { value },
          });
        },
      });
      showGroup.append(symbol.root, number.root);
      const showRow = createLabeledRow({
        label: "Show:",
        control: showGroup,
        spanControls: true,
      });

      const sizeRow = createSliderRow({
        label: "Size:",
        value: 16,
        min: 1,
        max: 100,
        step: 1,
        decimals: 0,
        onChange: (value) => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.SetSize,
            payload: { value },
          });
        },
      });

      const offsetRow = createSliderRow({
        label: "Offset:",
        value: 0.5,
        min: 0.01,
        max: 10,
        step: 0.1,
        decimals: 2,
        onChange: (value) => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.SetOffset,
            payload: { value },
          });
        },
      });

      const toggleGroup = document.createElement("div");
      toggleGroup.className = "cp-inline-group";
      const toggleAll = createButton({
        label: "All",
        onClick: () => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.ToggleVisibilityForAllAtoms,
            payload: {},
          });
        },
      });
      const toggleSelected = createButton({
        label: "Selected",
        onClick: () => {
          if (context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.ToggleVisibilityForSelectedAtoms,
            payload: {},
          });
        },
      });
      toggleGroup.append(toggleAll.root, toggleSelected.root);
      const toggleRow = createLabeledRow({
        label: "Toggle:",
        control: toggleGroup,
        spanControls: true,
      });

      form.append(showRow.root, sizeRow.root, offsetRow.root, toggleRow.root);
      surface.root.appendChild(form);

      return () => {
        symbol.destroy();
        number.destroy();
        sizeRow.destroy();
        offsetRow.destroy();
        toggleAll.destroy();
        toggleSelected.destroy();
        showRow.destroy();
        toggleRow.destroy();
      };
    },
  };
}
