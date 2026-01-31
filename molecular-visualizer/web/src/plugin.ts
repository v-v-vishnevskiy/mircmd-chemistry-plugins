import type { ProgramPluginContext } from './program_context';

interface MolecularVisualizerInstance {
    resize(width: number, height: number): void;
    scale_scene(factor: number): void;
    rotate_scene(pitch: number, yaw: number, roll: number): void;
    new_cursor_position(x: number, y: number): Promise<void>;
    render(): void;
}

interface WasmModule {
    default: (wasm_url: URL) => Promise<void>;
    MolecularVisualizer: {
        create(canvas: HTMLCanvasElement, data: Uint8Array): Promise<MolecularVisualizerInstance>;
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
    const visualizer = await wasm_module.MolecularVisualizer.create(canvas, data);
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

    // Handle mouse rotation
    let is_dragging = false;
    let last_mouse_x = 0;
    let last_mouse_y = 0;
    const rotation_sensitivity = 0.5;

    canvas.addEventListener('mousedown', (event: MouseEvent) => {
        if (event.button === 0) {
            is_dragging = true;
            last_mouse_x = event.clientX;
            last_mouse_y = event.clientY;
        }
    });

    canvas.addEventListener('mousemove', async (event: MouseEvent) => {
        if (is_dragging) {
            const delta_x = event.clientX - last_mouse_x;
            const delta_y = event.clientY - last_mouse_y;

            last_mouse_x = event.clientX;
            last_mouse_y = event.clientY;

            const yaw = delta_x * rotation_sensitivity;
            const pitch = delta_y * rotation_sensitivity;

            visualizer.rotate_scene(pitch, yaw, 0);
            visualizer.render();
        } else {
            const rect = canvas.getBoundingClientRect();
            const dpr = window.devicePixelRatio || 1;
            const x = Math.floor((event.clientX - rect.left) * dpr);
            const y = Math.floor((event.clientY - rect.top) * dpr);
            await visualizer.new_cursor_position(x, y);
        }
    });

    canvas.addEventListener('mouseup', (event: MouseEvent) => {
        if (event.button === 0) {
            is_dragging = false;
        }
    });

    canvas.addEventListener('mouseleave', () => {
        is_dragging = false;
    });

    // Handle mouse wheel zoom
    const zoom_sensitivity = 0.001;

    canvas.addEventListener('wheel', (event: WheelEvent) => {
        event.preventDefault();

        const factor = 1.0 - event.deltaY * zoom_sensitivity;
        visualizer.scale_scene(factor);
        visualizer.render();
    }, { passive: false });
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
