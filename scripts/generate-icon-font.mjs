#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SVGIcons2SVGFontStream } from 'svgicons2svgfont';
import svg2ttf from 'svg2ttf';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const sourceSvg = path.join(root, 'shelly.svg');
const outDir = path.join(root, 'fonts');
const outSvg = path.join(outDir, 'ShellyIcons.svg');
const outTtf = path.join(outDir, 'ShellyIcons.ttf');

fs.mkdirSync(outDir, { recursive: true });

const fontStream = new SVGIcons2SVGFontStream({
  fontName: 'ShellyIcons',
  fontHeight: 1000,
  normalize: true,
  centerHorizontally: true,
  log: () => {},
});

const chunks = [];
fontStream.on('data', (chunk) => chunks.push(Buffer.from(chunk)));
fontStream.on('finish', () => {
  const svgFont = Buffer.concat(chunks).toString('utf8');
  fs.writeFileSync(outSvg, svgFont);
  const ttf = svg2ttf(svgFont, { copyright: 'Shelly icon font' });
  fs.writeFileSync(outTtf, Buffer.from(ttf.buffer));
  console.log(`wrote ${path.relative(root, outSvg)}`);
  console.log(`wrote ${path.relative(root, outTtf)}`);
});
fontStream.on('error', (error) => {
  console.error(error);
  process.exitCode = 1;
});

const glyph = fs.createReadStream(sourceSvg);
glyph.metadata = {
  name: 'shelly',
  unicode: ['\u{100000}'],
};
fontStream.write(glyph);
fontStream.end();
