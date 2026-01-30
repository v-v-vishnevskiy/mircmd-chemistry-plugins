export interface ProgramPluginContext {
  host: HTMLElement;
  root: ShadowRoot;
  addStyles: (cssText: string) => void;
}
