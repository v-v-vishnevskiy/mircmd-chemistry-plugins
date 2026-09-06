// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

import type { Rgba } from "../wasm_types";

function toHexByte(value: number): string {
  return Math.min(255, Math.max(0, Math.round(value * 255)))
    .toString(16)
    .padStart(2, "0");
}

export function rgbaToHex(color: Rgba, withAlpha = false): string {
  const rgb = `#${toHexByte(color.r)}${toHexByte(color.g)}${toHexByte(color.b)}`;
  return withAlpha ? `${rgb}${toHexByte(color.a)}` : rgb;
}

export function hexToRgba(hex: string, defaultAlpha = 1): [number, number, number, number] {
  const raw = hex.replace("#", "");
  const r = Number.parseInt(raw.slice(0, 2), 16) / 255;
  const g = Number.parseInt(raw.slice(2, 4), 16) / 255;
  const b = Number.parseInt(raw.slice(4, 6), 16) / 255;
  const a = raw.length >= 8 ? Number.parseInt(raw.slice(6, 8), 16) / 255 : defaultAlpha;
  return [r, g, b, a];
}

export function readRgba(value: unknown, fallbackAlpha = 1): [number, number, number, number] {
  if (!Array.isArray(value)) {
    return [0, 0, 0, fallbackAlpha];
  }
  return [
    Number(value[0]) || 0,
    Number(value[1]) || 0,
    Number(value[2]) || 0,
    typeof value[3] === "number" ? value[3] : fallbackAlpha,
  ];
}
