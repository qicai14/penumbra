/** Whatever form a CSS colour takes (hex, rgb(), named), paint it on a 1x1 canvas and read the pixel back. sRGB bytes. */
export function rgb(color: string): [number, number, number] {
  const px = document.createElement("canvas"); px.width = px.height = 1;
  const g = px.getContext("2d", { willReadFrequently: true })!;
  g.fillStyle = color; g.fillRect(0, 0, 1, 1);
  const d = g.getImageData(0, 0, 1, 1).data;
  return [d[0], d[1], d[2]];
}
