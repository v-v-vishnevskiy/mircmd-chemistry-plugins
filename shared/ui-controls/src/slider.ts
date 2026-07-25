export type SliderOptions = {
  label?: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  /** Fires continuously while dragging (QSlider.valueChanged). */
  onInput?: (value: number) => void;
  /** Fires when the user commits (pointer up / change). */
  onChange?: (value: number) => void;
};

export type SliderControl = {
  root: HTMLElement;
  input: HTMLInputElement;
  setValue(value: number): void;
  setRange(min: number, max: number, step?: number): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

function readNumber(input: HTMLInputElement, fallback: number): number {
  const raw = Number(input.value);
  return Number.isFinite(raw) ? raw : fallback;
}

export function createSlider(options: SliderOptions): SliderControl {
  const root = document.createElement(options.label ? "label" : "div");
  root.className = "mircmd-slider";

  if (options.label) {
    const text = document.createElement("span");
    text.className = "mircmd-slider-label";
    text.textContent = options.label;
    root.appendChild(text);
  }

  const input = document.createElement("input");
  input.type = "range";
  input.min = String(options.min ?? 0);
  input.max = String(options.max ?? 100);
  input.step = String(options.step ?? 1);
  input.value = String(options.value);
  input.disabled = options.disabled ?? false;
  if (options.disabled) {
    root.classList.add("disabled");
  }
  root.appendChild(input);

  const onInput = () => {
    options.onInput?.(readNumber(input, options.value));
  };
  const onChange = () => {
    options.onChange?.(readNumber(input, options.value));
  };
  if (options.onInput) {
    input.addEventListener("input", onInput);
  }
  if (options.onChange) {
    input.addEventListener("change", onChange);
  }

  return {
    root,
    input,
    setValue(value: number) {
      input.value = String(value);
    },
    setRange(min: number, max: number, step?: number) {
      input.min = String(min);
      input.max = String(max);
      if (step !== undefined) {
        input.step = String(step);
      }
    },
    setDisabled(disabled: boolean) {
      input.disabled = disabled;
      root.classList.toggle("disabled", disabled);
    },
    destroy() {
      if (options.onInput) {
        input.removeEventListener("input", onInput);
      }
      if (options.onChange) {
        input.removeEventListener("change", onChange);
      }
      root.remove();
    },
  };
}
