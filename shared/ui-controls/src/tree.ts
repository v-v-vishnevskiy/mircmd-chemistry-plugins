export type TreeNode = {
  id: string;
  label: string;
  children?: TreeNode[];
  expanded?: boolean;
};

export type TreeOptions = {
  nodes?: TreeNode[];
  indentSize?: number;
  selectable?: boolean;
  showExpanders?: boolean;
  onSelect?: (node: TreeNode) => void;
  onToggle?: (node: TreeNode, expanded: boolean) => void;
  /** Trailing widgets per row (color / eye / delete). */
  renderTrailing?: (node: TreeNode) => HTMLElement | null | undefined;
};

export type TreeControl = {
  root: HTMLElement;
  setNodes(nodes: TreeNode[]): void;
  setDisabled(disabled: boolean): void;
  destroy(): void;
};

function hasChildren(node: TreeNode): boolean {
  return (node.children?.length ?? 0) > 0;
}

function isExpanded(node: TreeNode): boolean {
  if (node.expanded !== undefined) return node.expanded;
  return hasChildren(node);
}

export function createTree(options: TreeOptions = {}): TreeControl {
  const indentSize = options.indentSize ?? 20;
  const selectable = options.selectable !== false;
  const showExpanders = options.showExpanders !== false;

  const root = document.createElement("div");
  root.className = "mircmd-tree";
  root.setAttribute("role", "tree");

  let nodes: TreeNode[] = options.nodes ? structuredClone(options.nodes) : [];
  let selectedId: string | null = null;
  let disabled = false;

  const findNode = (id: string, list: TreeNode[]): TreeNode | null => {
    for (const node of list) {
      if (node.id === id) return node;
      if (node.children) {
        const found = findNode(id, node.children);
        if (found) return found;
      }
    }
    return null;
  };

  const renderNode = (node: TreeNode, level: number, container: HTMLElement) => {
    const item = document.createElement("div");
    item.className = "mircmd-tree-node";

    const row = document.createElement("div");
    row.className = "mircmd-tree-row";
    row.dataset.nodeId = node.id;
    row.style.paddingLeft = `${level * indentSize}px`;
    row.setAttribute("role", "treeitem");
    row.tabIndex = 0;
    if (selectable && selectedId === node.id) {
      row.classList.add("selected");
    }
    if (hasChildren(node)) {
      row.setAttribute("aria-expanded", String(isExpanded(node)));
    }

    const expander = document.createElement("button");
    expander.type = "button";
    expander.className = "mircmd-tree-expander";
    expander.tabIndex = -1;
    if (showExpanders && hasChildren(node)) {
      expander.textContent = isExpanded(node) ? "▾" : "▸";
      expander.addEventListener("click", (event) => {
        event.stopPropagation();
        if (disabled) return;
        node.expanded = !isExpanded(node);
        options.onToggle?.(node, isExpanded(node));
        render();
      });
    } else {
      expander.classList.add("empty");
      expander.disabled = true;
    }

    const label = document.createElement("span");
    label.className = "mircmd-tree-label";
    label.textContent = node.label;

    const trailing = document.createElement("div");
    trailing.className = "mircmd-tree-trailing";
    const trailingEl = options.renderTrailing?.(node);
    if (trailingEl) {
      trailing.appendChild(trailingEl);
    }

    row.append(expander, label, trailing);

    if (selectable) {
      row.addEventListener("click", () => {
        if (disabled) return;
        selectedId = node.id;
        options.onSelect?.(node);
        render();
      });
    }

    row.addEventListener("dblclick", (event) => {
      if (disabled || !hasChildren(node)) return;
      event.preventDefault();
      node.expanded = !isExpanded(node);
      options.onToggle?.(node, isExpanded(node));
      render();
    });

    item.appendChild(row);

    if (hasChildren(node) && isExpanded(node) && node.children) {
      const children = document.createElement("div");
      children.className = "mircmd-tree-children";
      for (const child of node.children) {
        renderNode(child, level + 1, children);
      }
      item.appendChild(children);
    }

    container.appendChild(item);
  };

  const render = () => {
    root.replaceChildren();
    for (const node of nodes) {
      renderNode(node, 0, root);
    }
  };

  render();

  return {
    root,
    setNodes(next: TreeNode[]) {
      nodes = structuredClone(next);
      if (selectedId && !findNode(selectedId, nodes)) {
        selectedId = null;
      }
      render();
    },
    setDisabled(value: boolean) {
      disabled = value;
      root.classList.toggle("disabled", value);
    },
    destroy() {
      root.replaceChildren();
      root.remove();
    },
  };
}
