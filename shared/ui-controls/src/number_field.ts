export type NumberFieldOptions = {
  label?: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  onChange?: (value: number) => void;
};

export type NumberFieldControl = {
  root: HTMLLabelElement;
  input: HTMLInputElement;
  setValue(value: number): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

export function createNumberField(options: NumberFieldOptions): NumberFieldControl {
  const root = document.createElement("label");
  root.className = "mircmd-number";

  if (options.label) {
    const text = document.createElement("span");
    text.textContent = options.label;
    root.appendChild(text);
  }

  const input = document.createElement("input");
  input.type = "number";
  input.value = String(options.value);
  if (options.min !== undefined) input.min = String(options.min);
  if (options.max !== undefined) input.max = String(options.max);
  if (options.step !== undefined) input.step = String(options.step);
  input.disabled = options.disabled ?? false;
  if (options.disabled) {
    root.classList.add("disabled");
  }

  root.appendChild(input);

  const onChange = () => {
    const raw = Number(input.value);
    options.onChange?.(Number.isFinite(raw) ? raw : options.value);
  };
  if (options.onChange) {
    input.addEventListener("change", onChange);
  }

  return {
    root,
    input,
    setValue(value: number) {
      input.value = String(value);
    },
    setDisabled(disabled: boolean) {
      input.disabled = disabled;
      root.classList.toggle("disabled", disabled);
    },
    destroy() {
      if (options.onChange) {
        input.removeEventListener("change", onChange);
      }
      root.remove();
    },
  };
}
