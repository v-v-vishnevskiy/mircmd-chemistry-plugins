export type TextFieldOptions = {
  label?: string;
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  onInput?: (value: string) => void;
  onChange?: (value: string) => void;
};

export type TextFieldControl = {
  root: HTMLElement;
  input: HTMLInputElement;
  setValue(value: string): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

export function createTextField(options: TextFieldOptions): TextFieldControl {
  const root = document.createElement(options.label ? "label" : "div");
  root.className = "mircmd-text";

  if (options.label) {
    const text = document.createElement("span");
    text.className = "mircmd-text-label";
    text.textContent = options.label;
    root.appendChild(text);
  }

  const input = document.createElement("input");
  input.type = "text";
  input.value = options.value ?? "";
  if (options.placeholder !== undefined) {
    input.placeholder = options.placeholder;
  }
  input.disabled = options.disabled ?? false;
  if (options.disabled) {
    root.classList.add("disabled");
  }
  root.appendChild(input);

  const onInput = () => {
    options.onInput?.(input.value);
  };
  const onChange = () => {
    options.onChange?.(input.value);
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
    setValue(value: string) {
      input.value = value;
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
