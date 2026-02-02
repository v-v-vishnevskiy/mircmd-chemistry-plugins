import type { ProgramPluginContext } from './program_context';

interface AtomInfo {
    symbol: string;
    tag: number;
}

interface MolecularVisualizerInstance {
    resize(width: number, height: number): void;
    scale_scene(factor: number): void;
    rotate_scene(pitch: number, yaw: number, roll: number): void;
    new_cursor_position(x: number, y: number): Promise<AtomInfo | null>;
    toggle_atom_selection(x: number, y: number): Promise<void>;
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
    const container = canvas.parentElement as HTMLElement;
    const overlay = create_overlay(container);
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
        }
    });
    resize_observer.observe(canvas);

    // Handle mouse rotation
    let is_dragging = false;
    let has_dragged = false;
    let is_async_busy = false;
    let last_mouse_x = 0;
    let last_mouse_y = 0;
    const rotation_sensitivity = 0.5;

    canvas.addEventListener('mousedown', (event: MouseEvent) => {
        if (event.button === 0) {
            is_dragging = true;
            has_dragged = false;
            last_mouse_x = event.clientX;
            last_mouse_y = event.clientY;
        }
    });

    canvas.addEventListener('click', async (event: MouseEvent) => {
        if (event.button === 0 && !has_dragged && !is_async_busy) {
            is_async_busy = true;
            try {
                const rect = canvas.getBoundingClientRect();
                const dpr = window.devicePixelRatio || 1;
                const canvas_x = Math.floor((event.clientX - rect.left) * dpr);
                const canvas_y = Math.floor((event.clientY - rect.top) * dpr);
                await visualizer.toggle_atom_selection(canvas_x, canvas_y);
            } finally {
                is_async_busy = false;
            }
        }
    });

    canvas.addEventListener('mousemove', async (event: MouseEvent) => {
        if (is_dragging) {
            has_dragged = true;
            const delta_x = event.clientX - last_mouse_x;
            const delta_y = event.clientY - last_mouse_y;

            last_mouse_x = event.clientX;
            last_mouse_y = event.clientY;

            const yaw = delta_x * rotation_sensitivity;
            const pitch = delta_y * rotation_sensitivity;

            visualizer.rotate_scene(pitch, yaw, 0);
            overlay.style.display = 'none';
        } else if (!is_async_busy) {
            is_async_busy = true;
            try {
                const rect = canvas.getBoundingClientRect();
                const dpr = window.devicePixelRatio || 1;
                const canvas_x = Math.floor((event.clientX - rect.left) * dpr);
                const canvas_y = Math.floor((event.clientY - rect.top) * dpr);
                const atom = await visualizer.new_cursor_position(canvas_x, canvas_y);
                const overlay_x = event.clientX - rect.left;
                const overlay_y = event.clientY - rect.top;
                update_overlay(overlay, atom, overlay_x, overlay_y, container);
            } finally {
                is_async_busy = false;
            }
        }
    });

    canvas.addEventListener('mouseup', (event: MouseEvent) => {
        if (event.button === 0) {
            is_dragging = false;
        }
    });

    canvas.addEventListener('mouseleave', () => {
        is_dragging = false;
        overlay.style.display = 'none';
    });

    // Handle mouse wheel zoom
    const zoom_sensitivity = 0.001;

    canvas.addEventListener('wheel', (event: WheelEvent) => {
        event.preventDefault();

        const factor = 1.0 - event.deltaY * zoom_sensitivity;
        visualizer.scale_scene(factor);
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
    container.style.position = 'relative';

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

function create_overlay(container: HTMLElement): HTMLDivElement {
    const overlay = document.createElement('div');
    overlay.style.position = 'absolute';
    overlay.style.backgroundColor = '#44444499';
    overlay.style.color = '#D8D8D8';
    overlay.style.padding = '6px 10px';
    overlay.style.borderRadius = '6px';
    overlay.style.fontSize = '13px';
    overlay.style.fontFamily = 'system-ui, -apple-system, sans-serif';
    overlay.style.pointerEvents = 'none';
    overlay.style.display = 'none';
    overlay.style.whiteSpace = 'nowrap';
    overlay.style.zIndex = '1000';

    container.appendChild(overlay);
    return overlay;
}

function update_overlay(
    overlay: HTMLDivElement,
    atom: AtomInfo | null,
    x: number,
    y: number,
    container: HTMLElement
): void {
    if (!atom) {
        overlay.style.display = 'none';
        return;
    }

    overlay.textContent = `Atom: ${atom.symbol}${atom.tag}`;
    overlay.style.display = 'block';

    const offset_x = 6;
    const offset_y = -6;
    const container_rect = container.getBoundingClientRect();

    let left = x + offset_x;
    let top = y + offset_y - overlay.offsetHeight;

    if (left + overlay.offsetWidth > container_rect.width) {
        left = x - offset_x - overlay.offsetWidth;
    }

    if (top < 0) {
        top = y + offset_x;
    }

    overlay.style.left = `${left}px`;
    overlay.style.top = `${top}px`;
}

// Export instantiate function compatible with current plugin loader
export function instantiate(): {
    run: (ctx: ProgramPluginContext, data: Uint8Array) => Promise<void>;
    supportedTypes: () => string[];
} {
    return { run, supportedTypes };
}
