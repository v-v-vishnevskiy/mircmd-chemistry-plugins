// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Label + slider + number field row (Qt add_slider parity).
 * Root uses display:contents so rows share a parent .cp-form grid.
 */

import {
  createNumberField,
  createSlider,
  type NumberFieldControl,
  type SliderControl,
} from "@mircmd/ui-controls";

export type SliderRowOptions = {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  decimals?: number;
  disabled?: boolean;
  onChange?: (value: number) => void;
};

export type SliderRowControl = {
  root: HTMLElement;
  setValue(value: number): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

function roundTo(value: number, decimals: number): number {
  const factor = 10 ** decimals;
  return Math.round(value * factor) / factor;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export function createSliderRow(options: SliderRowOptions): SliderRowControl {
  const decimals = options.decimals ?? 2;
  let applying = false;
  let current = options.value;

  const root = document.createElement("div");
  root.className = "cp-form-row";

  const label = document.createElement("span");
  label.className = "cp-label";
  label.textContent = options.label;

  let slider!: SliderControl;
  let number!: NumberFieldControl;

  const emit = (value: number) => {
    const next = roundTo(clamp(value, options.min, options.max), decimals);
    applying = true;
    try {
      slider.setValue(next);
      number.setValue(next);
      current = next;
    } finally {
      applying = false;
    }
    options.onChange?.(next);
  };

  slider = createSlider({
    value: options.value,
    min: options.min,
    max: options.max,
    step: options.step,
    disabled: options.disabled,
    onInput: (value) => {
      if (applying) return;
      emit(value);
    },
  });

  number = createNumberField({
    value: options.value,
    min: options.min,
    max: options.max,
    step: options.step,
    disabled: options.disabled,
    onChange: (value) => {
      if (applying) return;
      emit(value);
    },
  });

  root.append(label, slider.root, number.root);

  if (options.disabled) {
    root.classList.add("disabled");
  }

  return {
    root,
    setValue(value: number) {
      const next = roundTo(clamp(value, options.min, options.max), decimals);
      applying = true;
      try {
        slider.setValue(next);
        number.setValue(next);
        current = next;
      } finally {
        applying = false;
      }
    },
    setDisabled(disabled: boolean) {
      slider.setDisabled(disabled);
      number.setDisabled(disabled);
      root.classList.toggle("disabled", disabled);
    },
    destroy() {
      slider.destroy();
      number.destroy();
      root.remove();
      void current;
    },
  };
}
