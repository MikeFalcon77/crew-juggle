export function cropLayout(width: number, height: number, zoom: number, x: number, y: number) {
  const scale = Math.max(256 / width, 256 / height) * Math.max(1, zoom);
  const w = width * scale,
    h = height * scale;
  return {
    width: w,
    height: h,
    x: Math.max(-(w - 256) / 2, Math.min((w - 256) / 2, x)) || 0,
    y: Math.max(-(h - 256) / 2, Math.min((h - 256) / 2, y)) || 0,
  };
}
