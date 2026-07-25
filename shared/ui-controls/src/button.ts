export type ButtonOptions = {
  label: string;
  disabled?: boolean;
  onClick?: () => void;
};

export type ButtonControl = {
  root: HTMLButtonElement;
  setLabel(label: string): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

export function createButton(options: ButtonOptions): ButtonControl {
  const root = document.createElement("button");
  root.type = "button";
  root.className = "mircmd-button";
  root.textContent = options.label;
  root.disabled = options.disabled ?? false;
  if (options.disabled) {
    root.classList.add("disabled");
  }

  const onClick = () => {
    options.onClick?.();
  };
  if (options.onClick) {
    root.addEventListener("click", onClick);
  }

  return {
    root,
    setLabel(label: string) {
      root.textContent = label;
    },
    setDisabled(disabled: boolean) {
      root.disabled = disabled;
      root.classList.toggle("disabled", disabled);
    },
    destroy() {
      if (options.onClick) {
        root.removeEventListener("click", onClick);
      }
      root.remove();
    },
  };
}
