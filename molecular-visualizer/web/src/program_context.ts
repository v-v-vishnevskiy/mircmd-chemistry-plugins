export interface MenuItem {
  label: string;
  icon?: string;
  action?: (data: any) => void;
  children?: MenuItem[];
  separator?: boolean;
}

export interface ProgramPluginContext {
  host: HTMLElement;
  root: ShadowRoot;
  addStyles: (cssText: string) => void;
  contextMenu: {
    open: (params: { event: MouseEvent; items: MenuItem[]; data?: unknown }) => void;
    close: () => void;
  };
}
