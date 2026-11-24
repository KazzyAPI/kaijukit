import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Tree, type TreeItem } from './Tree';

const all: Record<string, { label: string; children?: string[] }> = {
  a: { label: 'A', children: ['a1', 'a2'] },
  a1: { label: 'A1' },
  a2: { label: 'A2' },
  b: { label: 'B', children: ['b1'] },
  b1: { label: 'B1' },
};

function Harness({ onSelect = () => undefined }: { onSelect?: (id: string) => void }) {
  const [open, setOpen] = useState<Set<string>>(new Set());
  const items: TreeItem[] = [];
  const walk = (ids: string[], level: number) =>
    ids.forEach((id) => {
      const n = all[id]!;
      items.push({ id, label: n.label, level, expanded: n.children ? open.has(id) : undefined });
      if (n.children && open.has(id)) walk(n.children, level + 1);
    });
  walk(['a', 'b'], 1);
  return (
    <Tree
      label="Pages"
      items={items}
      onSelect={onSelect}
      onToggle={(id, x) => setOpen((s) => new Set(x ? [...s, id] : [...s].filter((i) => i !== id)))}
    />
  );
}

const names = () => screen.getAllByRole('treeitem').map((e) => e.textContent);

afterEach(cleanup);

describe('Tree keyboard', () => {
  it('exposes tree roles, levels and a single tab stop', () => {
    render(<Harness />);
    expect(screen.getByRole('tree', { name: 'Pages' })).toBeTruthy();
    const items = screen.getAllByRole('treeitem');
    expect(items.map((i) => i.getAttribute('tabindex'))).toEqual(['0', '-1']);
    expect(items[0]!.getAttribute('aria-expanded')).toBe('false');
    expect(items[0]!.getAttribute('aria-level')).toBe('1');
  });

  it('right expands, right again enters first child, left goes to parent then collapses', async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.tab();
    expect(document.activeElement?.textContent).toBe('A');
    await user.keyboard('{ArrowRight}');
    expect(names()).toEqual(['A', 'A1', 'A2', 'B']);
    await user.keyboard('{ArrowRight}');
    expect(document.activeElement?.textContent).toBe('A1');
    expect(document.activeElement?.getAttribute('aria-level')).toBe('2');
    await user.keyboard('{ArrowLeft}');
    expect(document.activeElement?.textContent).toBe('A');
    await user.keyboard('{ArrowLeft}');
    expect(names()).toEqual(['A', 'B']);
  });

  it('down/up/end/home move focus and enter selects', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<Harness onSelect={onSelect} />);
    await user.tab();
    await user.keyboard('{ArrowDown}');
    expect(document.activeElement?.textContent).toBe('B');
    await user.keyboard('{Home}');
    expect(document.activeElement?.textContent).toBe('A');
    await user.keyboard('{End}{Enter}');
    expect(onSelect).toHaveBeenCalledWith('b');
  });
});
