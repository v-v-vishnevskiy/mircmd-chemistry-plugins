// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

import { get_element_by_number } from './periodic_table';
import type { ProgramPluginContext } from './program_context';
import styles from './style.css';
import type { AtomicCoordinates } from './types';

const ROW_HEIGHT = 28;
const CELL_PADDING = 4;
const COL_TAG_WIDTH = 55;
const COL_SYMBOL_WIDTH = 60;
const COL_COORD_WIDTH = 90;
const COL_COORD_MIN_WIDTH = 40;
const SCROLL_BUFFER = 20;

interface VirtualTableConfig {
    data: {
        symbols: string[];
        x: number[];
        y: number[];
        z: number[];
    };
    row_height: number;
    scroll_buffer: number;
}

function supportedTypes(): string[] {
    return ['mircmd:chemistry:atomic_coordinates'];
}

function run(ctx: ProgramPluginContext, data: Uint8Array): void {
    const parsed = parse_coords(data);
    clear_root(ctx.root);
    if (!parsed.ok) {
        render_error(ctx.root, parsed.error);
        return;
    }

    const symbols = get_symbols(parsed.value);
    const css_vars = build_css_vars();
    ctx.addStyles(`${css_vars}\n${styles}`);

    const config: VirtualTableConfig = {
        data: { symbols, x: parsed.value.x, y: parsed.value.y, z: parsed.value.z },
        row_height: ROW_HEIGHT,
        scroll_buffer: SCROLL_BUFFER,
    };

    const container = create_table_container();
    ctx.root.appendChild(container);
    init_virtual_table(container, config);
}

function parse_coords(data: Uint8Array):
    | { ok: true; value: AtomicCoordinates }
    | { ok: false; error: string } {
    try {
        const json = new TextDecoder().decode(data);
        return { ok: true, value: JSON.parse(json) as AtomicCoordinates };
    } catch (e) {
        return { ok: false, error: String(e) };
    }
}

function clear_root(root: ShadowRoot): void {
    root.textContent = '';
}

function render_error(root: ShadowRoot, message: string): void {
    const div = document.createElement('div');
    div.style.color = 'red';
    div.textContent = `Error: ${message}`;
    root.appendChild(div);
}

function get_symbols(coords: AtomicCoordinates): string[] {
    return coords.atomic_num.map((n) => {
        const element = get_element_by_number(n);
        return element ? element.symbol : `?(${n})`;
    });
}

function build_css_vars(): string {
    const min_width = COL_TAG_WIDTH + COL_SYMBOL_WIDTH + 3 * COL_COORD_MIN_WIDTH;
    return `.vt {
    --row-height: ${ROW_HEIGHT}px;
    --cell-padding: ${CELL_PADDING}px;
    --col-tag: ${COL_TAG_WIDTH}px;
    --col-symbol: ${COL_SYMBOL_WIDTH}px;
    --col-coord: ${COL_COORD_WIDTH}px;
    --col-coord-min: ${COL_COORD_MIN_WIDTH}px;
    --min-width: ${min_width}px;
}`;
}

function create_table_container(): HTMLDivElement {
    const container = document.createElement('div');
    container.className = 'vt';
    const header = create_header_row();
    const header_wrapper = document.createElement('div');
    header_wrapper.className = 'vt-header-wrapper';
    header_wrapper.appendChild(header);
    const body = create_body();
    container.append(header_wrapper, body);
    return container;
}

function create_header_row(): HTMLDivElement {
    const header = document.createElement('div');
    header.className = 'vt-header';
    header.append(
        create_header_cell('Tag', 'col-tag'),
        create_header_cell('Symbol', 'col-symbol'),
        create_header_cell('X', 'col-coord'),
        create_header_cell('Y', 'col-coord'),
        create_header_cell('Z', 'col-coord'),
    );
    return header;
}

function create_header_cell(text: string, class_name: string): HTMLDivElement {
    const cell = document.createElement('div');
    cell.className = class_name;
    cell.textContent = text;
    return cell;
}

function create_body(): HTMLDivElement {
    const body = document.createElement('div');
    body.className = 'vt-body';
    const viewport = document.createElement('div');
    viewport.className = 'vt-viewport';
    body.appendChild(viewport);
    return body;
}

function init_virtual_table(container: HTMLElement, config: VirtualTableConfig): void {
    if (container.dataset.vtInitialized) return;
    container.dataset.vtInitialized = 'true';
    const body = require_element(container, '.vt-body');
    const viewport = require_element(container, '.vt-viewport');
    const header = require_element(container, '.vt-header');
    prepare_viewport(body, header, viewport, config);
    const state = create_render_state();
    const render = () => render_visible_rows(body, viewport, config, state);
    attach_scroll_handler(body, header, render);
    render();
    attach_edit_handler(viewport);
}

function require_element<T extends HTMLElement>(container: Element, selector: string): T {
    const element = container.querySelector<T>(selector);
    if (!element) {
        throw new Error(`Missing element ${selector}`);
    }
    return element;
}

function prepare_viewport(
    body: HTMLElement,
    header: HTMLElement,
    viewport: HTMLElement,
    config: VirtualTableConfig,
): void {
    const total = config.data.symbols.length;
    viewport.style.height = `${total * config.row_height}px`;
    const scrollbar_width = body.offsetWidth - body.clientWidth;
    header.style.paddingRight = `${scrollbar_width}px`;
}

interface RenderState {
    row_cache: Map<number, HTMLDivElement>;
    visible_start: number;
    visible_end: number;
}

function create_render_state(): RenderState {
    return { row_cache: new Map(), visible_start: -1, visible_end: -1 };
}

function attach_scroll_handler(
    body: HTMLElement,
    header: HTMLElement,
    render: () => void,
): void {
    body.addEventListener(
        'scroll',
        () => {
            header.style.transform = `translateX(-${body.scrollLeft}px)`;
            render();
        },
        { passive: true },
    );
}

function render_visible_rows(
    body: HTMLElement,
    viewport: HTMLElement,
    config: VirtualTableConfig,
    state: RenderState,
): void {
    const total = config.data.symbols.length;
    const range = get_visible_range(body, config.row_height, config.scroll_buffer, total);
    if (range.start === state.visible_start && range.end === state.visible_end) return;
    prune_rows(state, range);
    append_rows(viewport, config, range, state);
    state.visible_start = range.start;
    state.visible_end = range.end;
}

function get_visible_range(
    body: HTMLElement,
    row_height: number,
    buffer: number,
    total: number,
): { start: number; end: number } {
    const scroll_top = body.scrollTop;
    const view_height = body.clientHeight;
    const start = Math.max(0, Math.floor(scroll_top / row_height) - buffer);
    const end = Math.min(total, Math.ceil((scroll_top + view_height) / row_height) + buffer);
    return { start, end };
}

function prune_rows(state: RenderState, range: { start: number; end: number }): void {
    for (const [idx, row] of state.row_cache) {
        if (idx < range.start || idx >= range.end) {
            row.remove();
            state.row_cache.delete(idx);
        }
    }
}

function append_rows(
    viewport: HTMLElement,
    config: VirtualTableConfig,
    range: { start: number; end: number },
    state: RenderState,
): void {
    const fragment = document.createDocumentFragment();
    for (let i = range.start; i < range.end; i++) {
        if (state.row_cache.has(i)) continue;
        const row = build_row(i, config);
        fragment.appendChild(row);
        state.row_cache.set(i, row);
    }
    viewport.appendChild(fragment);
}

function build_row(index: number, config: VirtualTableConfig): HTMLDivElement {
    const row = document.createElement('div');
    row.className = 'vt-row';
    row.style.top = `${index * config.row_height}px`;
    row.innerHTML =
        `<div class="vt-cell col-tag">${index + 1}</div>` +
        `<div class="vt-cell col-symbol">${config.data.symbols[index]}</div>` +
        `<div class="vt-cell col-coord">${config.data.x[index].toFixed(6)}</div>` +
        `<div class="vt-cell col-coord">${config.data.y[index].toFixed(6)}</div>` +
        `<div class="vt-cell col-coord">${config.data.z[index].toFixed(6)}</div>`;
    return row;
}

function attach_edit_handler(viewport: HTMLElement): void {
    viewport.addEventListener('dblclick', (event) => {
        const target = event.target;
        if (!(target instanceof Element)) return;
        const cell = target.closest('.vt-cell');
        if (!cell || cell.classList.contains('editing')) return;
        start_cell_editing(cell);
    });
}

function start_cell_editing(cell: Element): void {
    const original_value = cell.textContent ?? '';
    cell.classList.add('editing');
    const input = document.createElement('input');
    input.type = 'text';
    input.value = original_value;
    cell.textContent = '';
    cell.appendChild(input);
    input.focus();
    input.select();
    const finish = (save: boolean) => finish_editing(cell, input, original_value, save);
    input.addEventListener('blur', () => finish(true));
    input.addEventListener('keydown', (event) => handle_edit_key(event, input, finish));
}

function finish_editing(
    cell: Element,
    input: HTMLInputElement,
    original_value: string,
    save: boolean,
): void {
    if (!cell.classList.contains('editing')) return;
    cell.classList.remove('editing');
    cell.textContent = save ? input.value : original_value;
}

function handle_edit_key(
    event: KeyboardEvent,
    input: HTMLInputElement,
    finish: (save: boolean) => void,
): void {
    if (event.key === 'Enter') {
        event.preventDefault();
        input.blur();
        return;
    }
    if (event.key === 'Escape') {
        finish(false);
    }
}

// Export instantiate function compatible with current plugin loader
export function instantiate(): {
    run: (ctx: ProgramPluginContext, data: Uint8Array) => void;
    supportedTypes: () => string[];
} {
    return { run, supportedTypes };
}
