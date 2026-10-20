export interface TreemapNode {
  id: number;
  name: string;
  value: number; // bytes
  color: string;
  isHidden: boolean;
}

export interface TreemapRect {
  node: TreemapNode;
  x: number;
  y: number;
  width: number;
  height: number;
}

function squarify(
  nodes: TreemapNode[],
  x: number,
  y: number,
  width: number,
  height: number
): TreemapRect[] {
  if (nodes.length === 0 || width <= 0 || height <= 0) return [];

  const total = nodes.reduce((sum, n) => sum + n.value, 0);
  if (total === 0) return [];

  const rects: TreemapRect[] = [];
  let remaining = [...nodes];
  let rx = x,
    ry = y,
    rw = width,
    rh = height;

  while (remaining.length > 0) {
    const short = Math.min(rw, rh);
    if (short <= 0) break;

    const remainingTotal = remaining.reduce((s, n) => s + n.value, 0);
    if (remainingTotal === 0) break;

    let rowNodes: TreemapNode[] = [];
    let rowTotal = 0;
    let bestRatio = Infinity;

    for (let i = 0; i < remaining.length; i++) {
      const candidate = [...rowNodes, remaining[i]];
      const candidateTotal = rowTotal + remaining[i].value;
      const rowArea = (candidateTotal / remainingTotal) * rw * rh;
      const rowLen = rowArea / short;

      if (rowLen <= 0) continue;

      let worst = 0;
      for (const n of candidate) {
        const cellArea = (n.value / candidateTotal) * rowLen * short;
        const cellW = cellArea / short;
        if (cellW <= 0) continue;
        const r = Math.max(short / cellW, cellW / short);
        worst = Math.max(worst, r);
      }

      if (worst <= bestRatio || rowNodes.length === 0) {
        rowNodes = candidate;
        rowTotal = candidateTotal;
        bestRatio = worst;
      } else {
        break;
      }
    }

    if (rowNodes.length === 0) break;

    const rowFraction = rowTotal / remainingTotal;
    const isHorizontal = rw <= rh;
    const rowLen = (isHorizontal ? rw : rh) * rowFraction;
    let cursor = isHorizontal ? ry : rx;

    for (const n of rowNodes) {
      const fraction = n.value / rowTotal;
      const cellLen = (isHorizontal ? rh : rw) * fraction;
      if (isHorizontal) {
        rects.push({
          node: n,
          x: rx,
          y: cursor,
          width: Math.max(0, rowLen),
          height: Math.max(0, cellLen),
        });
        cursor += cellLen;
      } else {
        rects.push({
          node: n,
          x: cursor,
          y: ry,
          width: Math.max(0, cellLen),
          height: Math.max(0, rowLen),
        });
        cursor += cellLen;
      }
    }

    remaining = remaining.slice(rowNodes.length);
    if (isHorizontal) {
      rx += rowLen;
      rw -= rowLen;
    } else {
      ry += rowLen;
      rh -= rowLen;
    }
  }

  return rects;
}

export function computeTreemap(
  nodes: TreemapNode[],
  width: number,
  height: number
): TreemapRect[] {
  const sorted = [...nodes].filter((n) => n.value > 0).sort((a, b) => b.value - a.value);
  return squarify(sorted, 0, 0, width, height);
}
