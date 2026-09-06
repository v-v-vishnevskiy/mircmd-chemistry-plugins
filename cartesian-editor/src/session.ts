// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

import type {
  ControlPanelContribution,
  ProgramCommand,
  ProgramCommandContext,
  ProgramSession,
} from "./program_context";
import {
  createCartesianControlPanel,
  DecimalsCommand,
  type CartesianEditorState,
} from "./control_panel";

export type CartesianRenderHook = (decimals: number) => void;

export class CartesianEditorSession implements ProgramSession {
  private disposed = false;
  private state: CartesianEditorState = { decimals: 6 };
  readonly controlPanel: ControlPanelContribution;

  constructor(
    private readonly onDecimalsChanged: CartesianRenderHook,
    private readonly cleanup: () => void | Promise<void>,
  ) {
    this.controlPanel = createCartesianControlPanel(() => this.state);
  }

  getDecimals(): number {
    return this.state.decimals;
  }

  async execute(command: ProgramCommand, _context: ProgramCommandContext): Promise<void> {
    if (this.disposed) return;
    if (command.type === DecimalsCommand.Set) {
      const value = Math.min(15, Math.max(1, Math.round(Number(command.payload) || 6)));
      this.state = { decimals: value };
      this.onDecimalsChanged(value);
    }
  }

  async dispose(): Promise<void> {
    if (this.disposed) return;
    this.disposed = true;
    await this.cleanup();
  }
}
