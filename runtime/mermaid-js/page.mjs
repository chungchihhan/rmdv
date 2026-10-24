import mermaid from '/node_modules/mermaid/dist/mermaid.esm.min.mjs';
import { loadAwsIcons } from '/icons.mjs';
const aws = await loadAwsIcons();
mermaid.registerIconPacks([{ name: 'aws', icons: aws.pack }]);
let serial = 0;
window.renderDiagram = async ({ source, raster }) => {
  if (new TextEncoder().encode(source).length > 68 * 1024) throw Error('diagram too large');
  aws.validate(source);
  mermaid.initialize({ startOnLoad: false, securityLevel: 'strict', suppressErrorRendering: true,
    maxTextSize: 68 * 1024, maxEdges: 500, layout: 'dagre', theme: 'base',
    fontFamily: 'Helvetica, PingFang TC, sans-serif' });
  const host = document.getElementById('diagram');
  host.replaceChildren();
  await mermaid.parse(source);
  const { svg } = await mermaid.render(`rmdv-${++serial}`, source);
  if (new TextEncoder().encode(svg).length > 4 * 1024 * 1024) throw Error('rendered diagram too large');
  host.innerHTML = svg;
  await document.fonts.ready;
  const element = host.firstElementChild;
  const box = element.viewBox.baseVal;
  if (!(box.width > 0 && box.height > 0)) throw Error('svg has zero size');
  const scale = Math.min(2, 1600 / box.width, 8192 / box.height, Math.sqrt(8_000_000 / (box.width * box.height)));
  if (raster) {
    element.style.maxWidth = 'none';
    element.style.width = `${Math.max(1, Math.floor(box.width * scale))}px`;
    element.style.height = `${Math.max(1, Math.floor(box.height * scale))}px`;
  }
  return { svg };
};
