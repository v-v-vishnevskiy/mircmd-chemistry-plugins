// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

//! Cartesian Coordinates Editor Plugin
//!
//! A virtual table component for displaying and editing atomic Cartesian coordinates.
//!
//! Features:
//! - Virtual scrolling for efficient rendering of large datasets (hundreds of thousands of rows)
//! - Five columns: Tag (row number), Symbol (element), X, Y, Z coordinates
//! - Fixed width for Tag and Symbol columns, flexible width for coordinate columns
//! - Inline cell editing on double-click
//!
//! TODO:
//! - Notify host application about cell value changes
//! - Add numeric validation for coordinate cells

#[allow(warnings)]
mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "plugin",
    });

    use super::CartesianEditor;

    export!(CartesianEditor);
}

use bindings::Guest;
use shared_lib::periodic_table::get_element_by_number;
use shared_lib::types::AtomicCoordinates;

const ROW_HEIGHT: u32 = 28;
const CELL_PADDING: u32 = 4;
const COL_TAG_WIDTH: u32 = 55;
const COL_SYMBOL_WIDTH: u32 = 60;
const COL_COORD_WIDTH: u32 = 90;
const COL_COORD_MIN_WIDTH: u32 = 40;
const SCROLL_BUFFER: u32 = 20;

struct CartesianEditor;

impl Guest for CartesianEditor {
    fn render(data: Vec<u8>) -> String {
        let coords: AtomicCoordinates = match serde_json::from_slice(&data) {
            Ok(c) => c,
            Err(e) => return format!("<div style=\"color:red\">Error: {e}</div>"),
        };

        let symbols: Vec<String> = coords
            .atomic_num
            .iter()
            .map(|&n| {
                get_element_by_number(n)
                    .map(|e| e.symbol.to_string())
                    .unwrap_or_else(|| format!("?({n})"))
            })
            .collect();

        let data_json = serde_json::json!({
            "symbols": symbols,
            "x": coords.x,
            "y": coords.y,
            "z": coords.z,
        });

        let min_width = COL_TAG_WIDTH + COL_SYMBOL_WIDTH + 3 * COL_COORD_MIN_WIDTH;

        format!(
            r##"<style>
.vt {{
    --row-height: {row_height}px;
    --cell-padding: {cell_padding}px;
    --col-tag: {col_tag}px;
    --col-symbol: {col_symbol}px;
    --col-coord: {col_coord}px;
    --col-coord-min: {col_coord_min}px;
    --min-width: {min_width}px;
    --border-color: #e0e0e0;
    --header-bg: #f5f5f5;
    --hover-bg: #f9f9f9;
    --focus-color: #007acc;

    height: 100%;
    display: flex;
    flex-direction: column;
}}
.vt-header-wrapper {{
    flex-shrink: 0;
    overflow: hidden;
    border-bottom: 1px solid var(--border-color);
    background: var(--header-bg);
}}
.vt-header {{
    display: flex;
    min-width: var(--min-width);
}}
.vt-header > div {{
    padding: var(--cell-padding);
    text-align: center;
    font-weight: bold;
    box-sizing: border-box;
    border-right: 1px solid var(--border-color);
}}
.vt-header > div:last-child {{
    border-right: none;
}}
.vt-body {{
    flex: 1;
    overflow: auto;
}}
.vt-viewport {{
    position: relative;
    min-width: var(--min-width);
}}
.vt-row {{
    display: flex;
    position: absolute;
    left: 0;
    right: 0;
    height: var(--row-height);
}}
.vt-row:hover {{
    background: var(--hover-bg);
}}
.vt-cell {{
    padding: var(--cell-padding);
    border-right: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
    box-sizing: border-box;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    height: var(--row-height);
    line-height: calc(var(--row-height) - var(--cell-padding) * 2);
}}
.vt-cell:last-child {{
    border-right: none;
}}
.vt-cell.editing {{
    padding: 0;
    outline: 1px solid var(--focus-color);
    outline-offset: -1px;
}}
.col-tag {{
    width: var(--col-tag);
    flex-shrink: 0;
    text-align: center;
}}
.col-symbol {{
    width: var(--col-symbol);
    flex-shrink: 0;
    text-align: center;
}}
.col-coord {{
    flex: 1 0 var(--col-coord);
    min-width: var(--col-coord-min);
    text-align: right;
}}
.vt-cell input {{
    width: 100%;
    height: 100%;
    padding: var(--cell-padding);
    margin: 0;
    border: none;
    background: transparent;
    font: inherit;
    line-height: inherit;
    text-align: inherit;
    outline: none;
    box-sizing: border-box;
}}
</style>
<div class="vt">
    <div class="vt-header-wrapper">
        <div class="vt-header">
            <div class="col-tag">Tag</div>
            <div class="col-symbol">Symbol</div>
            <div class="col-coord">X</div>
            <div class="col-coord">Y</div>
            <div class="col-coord">Z</div>
        </div>
    </div>
    <div class="vt-body">
        <div class="vt-viewport"></div>
    </div>
</div>
<script>
(function() {{
    const DATA = {data_json};
    const ROW_HEIGHT = {row_height};
    const BUFFER = {buffer};
    const TOTAL = DATA.symbols.length;

    const body = document.querySelector('.vt-body');
    const viewport = document.querySelector('.vt-viewport');
    const header = document.querySelector('.vt-header');

    viewport.style.height = TOTAL * ROW_HEIGHT + 'px';

    const scrollbarWidth = body.offsetWidth - body.clientWidth;
    if (scrollbarWidth > 0) {{
        header.style.paddingRight = scrollbarWidth + 'px';
    }}

    const rowCache = new Map();
    let visibleStart = -1;
    let visibleEnd = -1;

    function renderVisibleRows() {{
        const scrollTop = body.scrollTop;
        const viewHeight = body.clientHeight;

        const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER);
        const end = Math.min(TOTAL, Math.ceil((scrollTop + viewHeight) / ROW_HEIGHT) + BUFFER);

        if (start === visibleStart && end === visibleEnd) return;

        for (const [idx, row] of rowCache) {{
            if (idx < start || idx >= end) {{
                row.remove();
                rowCache.delete(idx);
            }}
        }}

        const fragment = document.createDocumentFragment();
        for (let i = start; i < end; i++) {{
            if (rowCache.has(i)) continue;

            const row = document.createElement('div');
            row.className = 'vt-row';
            row.style.top = i * ROW_HEIGHT + 'px';
            row.innerHTML =
                `<div class="vt-cell col-tag">${{i + 1}}</div>` +
                `<div class="vt-cell col-symbol">${{DATA.symbols[i]}}</div>` +
                `<div class="vt-cell col-coord">${{DATA.x[i].toFixed(6)}}</div>` +
                `<div class="vt-cell col-coord">${{DATA.y[i].toFixed(6)}}</div>` +
                `<div class="vt-cell col-coord">${{DATA.z[i].toFixed(6)}}</div>`;
            fragment.appendChild(row);
            rowCache.set(i, row);
        }}
        viewport.appendChild(fragment);

        visibleStart = start;
        visibleEnd = end;
    }}

    body.addEventListener('scroll', () => {{
        header.style.transform = `translateX(-${{body.scrollLeft}}px)`;
        renderVisibleRows();
    }}, {{ passive: true }});

    renderVisibleRows();

    viewport.addEventListener('dblclick', (e) => {{
        const cell = e.target.closest('.vt-cell');
        if (!cell || cell.classList.contains('editing')) return;

        const originalValue = cell.textContent;
        cell.classList.add('editing');

        const input = document.createElement('input');
        input.type = 'text';
        input.value = originalValue;
        cell.textContent = '';
        cell.appendChild(input);
        input.focus();
        input.select();

        function finishEditing(save) {{
            if (!cell.classList.contains('editing')) return;
            cell.classList.remove('editing');
            cell.textContent = save ? input.value : originalValue;
            // TODO: notify host about value change
        }}

        input.addEventListener('blur', () => finishEditing(true));
        input.addEventListener('keydown', (e) => {{
            if (e.key === 'Enter') {{
                e.preventDefault();
                input.blur();
            }} else if (e.key === 'Escape') {{
                finishEditing(false);
            }}
        }});
    }});
}})();
</script>"##,
            data_json = data_json,
            row_height = ROW_HEIGHT,
            cell_padding = CELL_PADDING,
            col_tag = COL_TAG_WIDTH,
            col_symbol = COL_SYMBOL_WIDTH,
            col_coord = COL_COORD_WIDTH,
            col_coord_min = COL_COORD_MIN_WIDTH,
            min_width = min_width,
            buffer = SCROLL_BUFFER,
        )
    }
}
