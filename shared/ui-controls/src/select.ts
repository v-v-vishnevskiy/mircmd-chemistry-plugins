export type SelectOption = {
  value: string;
  label: string;
};

export type SelectOptions = {
  label?: string;
  options?: SelectOption[];
  value?: string;
  disabled?: boolean;
  onChange?: (value: string) => void;
};

export type SelectControl = {
  root: HTMLElement;
  select: HTMLSelectElement;
  setValue(value: string): void;
  setOptions(options: SelectOption[]): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

function fillOptions(select: HTMLSelectElement, items: SelectOption[]): void {
  select.replaceChildren();
  for (const item of items) {
    const option = document.createElement("option");
    option.value = item.value;
    option.textContent = item.label;
    select.appendChild(option);
  }
}

export function createSelect(options: SelectOptions): SelectControl {
  const root = document.createElement(options.label ? "label" : "div");
  root.className = "mircmd-select";

  if (options.label) {
    const text = document.createElement("span");
    text.className = "mircmd-select-label";
    text.textContent = options.label;
    root.appendChild(text);
  }

  const select = document.createElement("select");
  const items = options.options ?? [];
  fillOptions(select, items);
  if (options.value !== undefined) {
    select.value = options.value;
  }
  select.disabled = options.disabled ?? false;
  if (options.disabled) {
    root.classList.add("disabled");
  }
  root.appendChild(select);

  const onChange = () => {
    options.onChange?.(select.value);
  };
  if (options.onChange) {
    select.addEventListener("change", onChange);
  }

  return {
    root,
    select,
    setValue(value: string) {
      select.value = value;
    },
    setOptions(next: SelectOption[]) {
      const previous = select.value;
      fillOptions(select, next);
      const stillExists = next.some((item) => item.value === previous);
      if (stillExists) {
        select.value = previous;
      }
    },
    setDisabled(disabled: boolean) {
      select.disabled = disabled;
      root.classList.toggle("disabled", disabled);
    },
    destroy() {
      if (options.onChange) {
        select.removeEventListener("change", onChange);
      }
      root.remove();
    },
  };
}
