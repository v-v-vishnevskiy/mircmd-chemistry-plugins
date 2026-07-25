export type ColorFieldOptions = {
  label?: string;
  value?: string;
  disabled?: boolean;
  onChange?: (value: string) => void;
};

export type ColorFieldControl = {
  root: HTMLElement;
  input: HTMLInputElement;
  setValue(value: string): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

function normalizeHex(value: string): string {
  const trimmed = value.trim();
  if (/^#[0-9a-fA-F]{6}$/.test(trimmed)) {
    return trimmed.toLowerCase();
  }
  if (/^[0-9a-fA-F]{6}$/.test(trimmed)) {
    return `#${trimmed.toLowerCase()}`;
  }
  return "#000000";
}

export function createColorField(options: ColorFieldOptions): ColorFieldControl {
  const root = document.createElement(options.label ? "label" : "div");
  root.className = "mircmd-color";

  if (options.label) {
    const text = document.createElement("span");
    text.className = "mircmd-color-label";
    text.textContent = options.label;
    root.appendChild(text);
  }

  const input = document.createElement("input");
  input.type = "color";
  input.value = normalizeHex(options.value ?? "#ffffff");
  input.disabled = options.disabled ?? false;
  if (options.disabled) {
    root.classList.add("disabled");
  }
  root.appendChild(input);

  const onChange = () => {
    options.onChange?.(input.value);
  };
  if (options.onChange) {
    input.addEventListener("change", onChange);
  }

  return {
    root,
    input,
    setValue(value: string) {
      input.value = normalizeHex(value);
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
