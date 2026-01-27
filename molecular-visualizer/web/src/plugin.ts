import type { ProgramPluginContext } from './program_context';

interface WasmModule {
    default: (wasm_url: URL) => Promise<void>;
    MolecularVisualizer: new (canvas: HTMLCanvasElement) => {
        set_data(data: Uint8Array): void;
        render(): void;
    };
}

let wasm_module: WasmModule | null = null;

function supportedTypes(): string[] {
    return ['mircmd:chemistry:atomic_coordinates'];
}

async function run(ctx: ProgramPluginContext, data: Uint8Array): Promise<void> {
    clear_root(ctx.root);

    if (!wasm_module) {
        const module_url = new URL('./molecular_visualizer.js', import.meta.url);
        wasm_module = (await import(module_url.href)) as WasmModule;
        const wasm_url = new URL('./molecular_visualizer_bg.wasm', import.meta.url);
        await wasm_module.default(wasm_url);
    }

    const canvas = create_canvas(ctx.root);
    const visualizer = new wasm_module.MolecularVisualizer(canvas);
    visualizer.set_data(data);
    visualizer.render();
}

function clear_root(root: ShadowRoot): void {
    root.textContent = '';
}

function create_canvas(root: ShadowRoot): HTMLCanvasElement {
    const container = document.createElement('div');
    container.style.width = '100%';
    container.style.height = '100%';
    container.style.display = 'flex';
    container.style.justifyContent = 'center';
    container.style.alignItems = 'center';
    container.style.backgroundColor = '#1e1e1e';

    const canvas = document.createElement('canvas');
    canvas.style.maxWidth = '100%';
    canvas.style.maxHeight = '100%';

    container.appendChild(canvas);
    root.appendChild(container);

    return canvas;
}

// Export instantiate function compatible with current plugin loader
export function instantiate(): {
    run: (ctx: ProgramPluginContext, data: Uint8Array) => Promise<void>;
    supportedTypes: () => string[];
} {
    return { run, supportedTypes };
}
