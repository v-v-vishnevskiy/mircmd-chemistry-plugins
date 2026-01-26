// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

export interface ProgramPluginContext {
  host: HTMLElement;
  root: ShadowRoot;
  addStyles: (cssText: string) => void;
}
