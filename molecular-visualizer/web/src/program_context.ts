// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * Local mirror of host program plugin contract (protocol v1).
 * Keep structurally aligned with mircmd-app/ui/core/program_plugin_api.ts.
 */

export type MaybePromise<T> = T | Promise<T>;
export type Cleanup = () => MaybePromise<void>;

export interface MenuItem {
  label: string;
  icon?: string;
  disabled?: boolean;
  shortcut?: string;
  action?: (data: unknown) => void;
  children?: MenuItem[];
  checkable?: boolean;
  checked?: boolean;
  separator?: boolean;
}

export interface ContextMenuParams {
  event: MouseEvent;
  items: MenuItem[];
  data?: unknown;
}

export interface PluginSurface {
  host: HTMLElement;
  root: ShadowRoot;
  addStyles(cssText: string): void;
}

export interface ProgramNodeIdentity {
  readonly id: string;
  readonly name: string;
  readonly type: string;
}

export interface ProgramPluginContext extends PluginSurface {
  signal: AbortSignal;
  node: ProgramNodeIdentity;
  contextMenu: {
    open(params: ContextMenuParams): void;
    close(): void;
  };
}

export interface ProgramCommand {
  type: string;
  payload?: unknown;
}

export interface ProgramCommandContext {
  instanceIndex: number;
  instanceCount: number;
}

export interface ControlPanelBlockMountContext {
  signal: AbortSignal;
  dispatch(command: ProgramCommand): Promise<void>;
}

export interface ControlPanelBlock {
  id: string;
  title: string;
  initiallyExpanded?: boolean;
  mount(
    surface: PluginSurface,
    context: ControlPanelBlockMountContext,
  ): MaybePromise<void | Cleanup>;
}

export interface ControlPanelContribution {
  title?: string;
  allowApplyToAll?: boolean;
  broadcastKey?: string;
  blocks: readonly ControlPanelBlock[];
}

export interface ProgramSession {
  controlPanel?: ControlPanelContribution;
  execute(command: ProgramCommand, context: ProgramCommandContext): MaybePromise<void>;
  dispose(): MaybePromise<void>;
}
