// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

import { get_element_by_number } from './periodic_table';
import styles from './style.css';
import type { AtomicCoordinates } from './types';

const ROW_HEIGHT = 28;
const CELL_PADDING = 4;
const COL_TAG_WIDTH = 55;
const COL_SYMBOL_WIDTH = 60;
const COL_COORD_WIDTH = 90;
const COL_COORD_MIN_WIDTH = 40;
const SCROLL_BUFFER = 20;

function render(data: Uint8Array): string {
  let coords: AtomicCoordinates;
  try {
    const json = new TextDecoder().decode(data);
    coords = JSON.parse(json) as AtomicCoordinates;
  } catch (e) {
    return `<div style="color:red">Error: ${e}</div>`;
  }

  const symbols = coords.atomic_num.map((n) => {
    const element = get_element_by_number(n);
    return element ? element.symbol : `?(${n})`;
  });

  const data_json = JSON.stringify({
    symbols,
    x: coords.x,
    y: coords.y,
    z: coords.z,
  });

  const min_width = COL_TAG_WIDTH + COL_SYMBOL_WIDTH + 3 * COL_COORD_MIN_WIDTH;

  const css_vars = `.vt {
    --row-height: ${ROW_HEIGHT}px;
    --cell-padding: ${CELL_PADDING}px;
    --col-tag: ${COL_TAG_WIDTH}px;
    --col-symbol: ${COL_SYMBOL_WIDTH}px;
    --col-coord: ${COL_COORD_WIDTH}px;
    --col-coord-min: ${COL_COORD_MIN_WIDTH}px;
    --min-width: ${min_width}px;
}`;

  return `<style>${css_vars}\n${styles}</style>
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
(function() {
    const DATA = ${data_json};
    const ROW_HEIGHT = ${ROW_HEIGHT};
    const BUFFER = ${SCROLL_BUFFER};
    const TOTAL = DATA.symbols.length;

    // Get the container element relative to the current script
    const container = document.currentScript.parentElement;
    const body = container.querySelector('.vt-body');
    const viewport = container.querySelector('.vt-viewport');
    const header = container.querySelector('.vt-header');

    viewport.style.height = TOTAL * ROW_HEIGHT + 'px';

    const scrollbarWidth = body.offsetWidth - body.clientWidth;
    header.style.paddingRight = scrollbarWidth + 'px';

    const rowCache = new Map();
    let visibleStart = -1;
    let visibleEnd = -1;

    function renderVisibleRows() {
        const scrollTop = body.scrollTop;
        const viewHeight = body.clientHeight;

        const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER);
        const end = Math.min(TOTAL, Math.ceil((scrollTop + viewHeight) / ROW_HEIGHT) + BUFFER);

        if (start === visibleStart && end === visibleEnd) return;

        for (const [idx, row] of rowCache) {
            if (idx < start || idx >= end) {
                row.remove();
                rowCache.delete(idx);
            }
        }

        const fragment = document.createDocumentFragment();
        for (let i = start; i < end; i++) {
            if (rowCache.has(i)) continue;

            const row = document.createElement('div');
            row.className = 'vt-row';
            row.style.top = i * ROW_HEIGHT + 'px';
            row.innerHTML =
                \`<div class="vt-cell col-tag">\${i + 1}</div>\` +
                \`<div class="vt-cell col-symbol">\${DATA.symbols[i]}</div>\` +
                \`<div class="vt-cell col-coord">\${DATA.x[i].toFixed(6)}</div>\` +
                \`<div class="vt-cell col-coord">\${DATA.y[i].toFixed(6)}</div>\` +
                \`<div class="vt-cell col-coord">\${DATA.z[i].toFixed(6)}</div>\`;
            fragment.appendChild(row);
            rowCache.set(i, row);
        }
        viewport.appendChild(fragment);

        visibleStart = start;
        visibleEnd = end;
    }

    body.addEventListener('scroll', () => {
        header.style.transform = \`translateX(-\${body.scrollLeft}px)\`;
        renderVisibleRows();
    }, { passive: true });

    renderVisibleRows();

    viewport.addEventListener('dblclick', (e) => {
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

        function finishEditing(save) {
            if (!cell.classList.contains('editing')) return;
            cell.classList.remove('editing');
            cell.textContent = save ? input.value : originalValue;
            // TODO: notify host about value change
        }

        input.addEventListener('blur', () => finishEditing(true));
        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                input.blur();
            } else if (e.key === 'Escape') {
                finishEditing(false);
            }
        });
    });
})();
</script>`;
}

// Export instantiate function compatible with current plugin loader
export function instantiate(): { render: (data: Uint8Array) => string } {
  return { render };
}
