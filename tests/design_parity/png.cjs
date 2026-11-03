// GitHub #414: decode Chrome's 8-bit RGBA PNG screenshots and count differing pixels (no dependency).
const zlib = require('zlib');
function decode(buf) {
  let pos = 8, width = 0, height = 0, bpp = 4; const idat = [];
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos), type = buf.toString('ascii', pos + 4, pos + 8), body = buf.subarray(pos + 8, pos + 8 + len);
    if (type === 'IHDR') { width = body.readUInt32BE(0); height = body.readUInt32BE(4); if (body[8] !== 8 || ![2, 6].includes(body[9]) || body[12] !== 0) throw new Error('unsupported PNG'); bpp = body[9] === 6 ? 4 : 3; }
    if (type === 'IDAT') idat.push(body);
    pos += 12 + len;
  }
  const raw = zlib.inflateSync(Buffer.concat(idat)), stride = width * bpp, out = Buffer.alloc(height * stride);
  for (let y = 0; y < height; y++) {
    const f = raw[y * (stride + 1)], line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let x = 0; x < stride; x++) {
      const a = x >= bpp ? out[y * stride + x - bpp] : 0, b = y ? out[(y - 1) * stride + x] : 0, c = x >= bpp && y ? out[(y - 1) * stride + x - bpp] : 0;
      const p = f === 0 ? 0 : f === 1 ? a : f === 2 ? b : f === 3 ? (a + b) >> 1 : (() => { const pa = Math.abs(b - c), pb = Math.abs(a - c), pc = Math.abs(a + b - 2 * c), pr = pa <= pb && pa <= pc ? a : pb <= pc ? b : c; return pr; })();
      out[y * stride + x] = (line[x] + p) & 255;
    }
  }
  if (bpp === 4) return { width, height, data: out };
  const rgba = Buffer.alloc(width * height * 4, 255);
  for (let i = 0; i < width * height; i++) out.copy(rgba, i * 4, i * 3, i * 3 + 3);
  return { width, height, data: rgba };
}
// Share of pixels whose channels differ by more than `tol` (0-255); sizes must match.
function diffRatio(a, b, tol = 24) {
  if (a.width !== b.width || a.height !== b.height) return { ratio: 1, sizeMismatch: [a.width, a.height, b.width, b.height] };
  let bad = 0;
  for (let i = 0; i < a.data.length; i += 4) if (Math.max(...[0, 1, 2, 3].map(k => Math.abs(a.data[i + k] - b.data[i + k]))) > tol) bad++;
  return { ratio: bad / (a.width * a.height) };
}
module.exports = { decode, diffRatio };
