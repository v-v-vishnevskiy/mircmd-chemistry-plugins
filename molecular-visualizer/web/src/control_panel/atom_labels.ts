// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Atom labels control block.
 * Live: Show Symbol / Number, Size, Offset, Toggle All / Selected.
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
import type { VisualizerState } from "../wasm_types";
import { createForm, createLabeledRow } from "./form_row";
import { createSliderRow } from "./slider_row";
import styles from "./styles.css";

export function createAtomLabelsBlock(
  controller: MolecularVisualizerController,
): ControlPanelBlock {
  return {
    id: "atom_labels",
    title: "Atom labels",
    initiallyExpanded: false,
    async mount(surface, context): Promise<Cleanup | void> {
      surface.addStyles(controlsStyles);
      surface.addStyles(styles);

      const form = createForm();
      let disposed = false;
      let applying = false;

      const initial = controller.getSnapshot().atom_labels;

      const showGroup = document.createElement("div");
      showGroup.className = "cp-inline-group";
      const symbol = createCheckbox({
        label: "Symbol",
        checked: initial.symbol_visible,
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.SetSymbolVisible,
            payload: { value },
          });
        },
      });
      const number = createCheckbox({
        label: "Number",
        checked: initial.number_visible,
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
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
        value: initial.size,
        min: 1,
        max: 100,
        step: 1,
        decimals: 0,
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.SetSize,
            payload: { value },
          });
        },
      });

      const offsetRow = createSliderRow({
        label: "Offset:",
        value: initial.offset,
        min: 0.01,
        max: 10,
        step: 0.1,
        decimals: 2,
        onChange: (value) => {
          if (applying || disposed || context.signal.aborted) return;
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
          if (applying || disposed || context.signal.aborted) return;
          void context.dispatch({
            type: AtomLabelsCommand.ToggleVisibilityForAllAtoms,
            payload: {},
          });
        },
      });
      const toggleSelected = createButton({
        label: "Selected",
        onClick: () => {
          if (applying || disposed || context.signal.aborted) return;
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

      const applySnapshot = (snapshot: VisualizerState = controller.getSnapshot()) => {
        if (disposed || context.signal.aborted) return;
        const labels = snapshot.atom_labels;
        applying = true;
        try {
          symbol.setChecked(labels.symbol_visible);
          number.setChecked(labels.number_visible);
          sizeRow.setValue(labels.size);
          offsetRow.setValue(labels.offset);
        } finally {
          applying = false;
        }
      };

      form.append(showRow.root, sizeRow.root, offsetRow.root, toggleRow.root);
      surface.root.appendChild(form);

      applySnapshot();

      const unsubscribe = controller.subscribe((snapshot, changedBlocks) => {
        if (disposed || context.signal.aborted) return;
        if (changedBlocks.length === 0 || changedBlocks.includes("atom_labels")) {
          applySnapshot(snapshot);
        }
      });

      const onAbort = () => {
        disposed = true;
      };
      context.signal.addEventListener("abort", onAbort, { once: true });

      return () => {
        disposed = true;
        context.signal.removeEventListener("abort", onAbort);
        unsubscribe();
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
