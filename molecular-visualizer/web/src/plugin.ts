import type { ProgramPluginContext } from './program_context';

interface MolecularVisualizerInstance {
    resize(width: number, height: number): void;
    render(): void;
}

interface WasmModule {
    default: (wasm_url: URL) => Promise<void>;
    MolecularVisualizer: {
        create(canvas: HTMLCanvasElement): Promise<MolecularVisualizerInstance>;
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
    const visualizer = await wasm_module.MolecularVisualizer.create(canvas);
    visualizer.render();

    // Handle resize
    const resize_observer = new ResizeObserver(() => {
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        const width = Math.floor(rect.width * dpr);
        const height = Math.floor(rect.height * dpr);

        if (canvas.width !== width || canvas.height !== height) {
            canvas.width = width;
            canvas.height = height;
            visualizer.resize(width, height);
            visualizer.render();
        }
    });
    resize_observer.observe(canvas);
}

function clear_root(root: ShadowRoot): void {
    root.textContent = '';
}

function create_canvas(root: ShadowRoot): HTMLCanvasElement {
    const container = document.createElement('div');
    container.style.width = '100%';
    container.style.height = '100%';
    container.style.overflow = 'hidden';

    const canvas = document.createElement('canvas');
    canvas.style.display = 'block';
    canvas.style.width = '100%';
    canvas.style.height = '100%';

    container.appendChild(canvas);
    root.appendChild(container);

    // Set canvas buffer size to match display size
    const rect = container.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;

    return canvas;
}

// Export instantiate function compatible with current plugin loader
export function instantiate(): {
    run: (ctx: ProgramPluginContext, data: Uint8Array) => Promise<void>;
    supportedTypes: () => string[];
} {
    return { run, supportedTypes };
}
