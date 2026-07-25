// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

/**
 * ProgramSession for molecular visualizer.
 */

import type {
  ControlPanelContribution,
  ProgramCommand,
  ProgramCommandContext,
  ProgramSession,
} from "./program_context";
import type { MolecularVisualizerController } from "./controller";
import { createControlPanelContribution } from "./control_panel";

export class MolecularVisualizerSession implements ProgramSession {
  private disposed = false;
  readonly controlPanel: ControlPanelContribution;

  constructor(
    private readonly controller: MolecularVisualizerController,
    private readonly cleanup: () => void | Promise<void>,
  ) {
    this.controlPanel = createControlPanelContribution(controller);
  }

  async execute(command: ProgramCommand, _context: ProgramCommandContext): Promise<void> {
    if (this.disposed) return;
    await this.controller.execute(command);
  }

  async dispose(): Promise<void> {
    if (this.disposed) return;
    this.disposed = true;
    try {
      await this.cleanup();
    } finally {
      await this.controller.dispose();
    }
  }
}
