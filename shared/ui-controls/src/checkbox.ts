export type CheckboxOptions = {
  label: string;
  checked?: boolean;
  disabled?: boolean;
  onChange?: (checked: boolean) => void;
};

export type CheckboxControl = {
  root: HTMLLabelElement;
  input: HTMLInputElement;
  setChecked(checked: boolean): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

export function createCheckbox(options: CheckboxOptions): CheckboxControl {
  const root = document.createElement("label");
  root.className = "mircmd-checkbox";

  const input = document.createElement("input");
  input.type = "checkbox";
  input.checked = options.checked ?? false;
  input.disabled = options.disabled ?? false;
  if (options.disabled) {
    root.classList.add("disabled");
  }

  root.append(input, document.createTextNode(options.label));

  const onChange = () => {
    options.onChange?.(input.checked);
  };
  if (options.onChange) {
    input.addEventListener("change", onChange);
  }

  return {
    root,
    input,
    setChecked(checked: boolean) {
      input.checked = checked;
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
