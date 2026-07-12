export interface MenuItem {
  label: string;
  icon?: string;
  disabled?: boolean;
  shortcut?: string;
  action?: (data: any) => void;
  children?: MenuItem[];
  checkable?: boolean;
  checked?: boolean;
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
