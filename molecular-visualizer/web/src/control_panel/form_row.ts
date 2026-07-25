// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Labeled form row for a shared .cp-form grid (display: contents).
 */

export type LabeledRowOptions = {
  label: string;
  /** Primary control element (column 2), or a wrapper for multiple controls. */
  control: HTMLElement;
  /** Optional trailing element (column 3). */
  trailing?: HTMLElement;
  /** If true, control spans columns 2–3. */
  spanControls?: boolean;
};

export type LabeledRowControl = {
  root: HTMLElement;
  destroy(): void;
};

export function createLabeledRow(options: LabeledRowOptions): LabeledRowControl {
  const root = document.createElement("div");
  root.className = "cp-form-row";

  const label = document.createElement("span");
  label.className = "cp-label";
  label.textContent = options.label;

  root.appendChild(label);

  if (options.spanControls) {
    const span = document.createElement("div");
    span.className = "cp-span-controls";
    span.appendChild(options.control);
    if (options.trailing) {
      span.appendChild(options.trailing);
    }
    root.appendChild(span);
  } else {
    root.appendChild(options.control);
    if (options.trailing) {
      root.appendChild(options.trailing);
    }
  }

  return {
    root,
    destroy() {
      root.remove();
    },
  };
}

export function createForm(): HTMLElement {
  const form = document.createElement("div");
  form.className = "cp-form";
  return form;
}

export function wrapFullWidth(element: HTMLElement): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "cp-span-full";
  wrap.appendChild(element);
  return wrap;
}
