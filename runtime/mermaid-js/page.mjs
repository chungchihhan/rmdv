import mermaid from '/node_modules/mermaid/dist/mermaid.esm.min.mjs';
import { loadAwsIcons } from '/icons.mjs';
const aws = await loadAwsIcons();
const logosResponse = await fetch('/node_modules/@iconify-json/logos/icons.json');
if (!logosResponse.ok) throw Error('Logos 圖示包尚未安裝，請在 runtime/mermaid-js 執行 npm ci。');
const logos = await logosResponse.json();
mermaid.registerIconPacks([{ name: 'aws', icons: aws.pack }, { name: 'logos', icons: logos }]);
let serial = 0;
window.renderDiagram = async ({ source, raster, config = {} }) => {
  if (new TextEncoder().encode(source).length > 68 * 1024) throw Error('diagram too large');
  aws.validate(source);
  for (const match of source.matchAll(/\blogos:([a-z0-9-]+)/g)) {
    if (!Object.hasOwn(logos.icons, match[1]) && !Object.hasOwn(logos.aliases || {}, match[1])) {
      throw Error(`找不到 Logos 圖示：${match[0]}。`);
    }
  }
  // Configure defaults from the current app palette on every request. Mermaid
  // still applies explicit source init/frontmatter/style overrides afterwards.
  const variables = config.themeVariables || {};
  document.documentElement.style.backgroundColor = 'transparent';
  document.body.style.backgroundColor = 'transparent';
  document.documentElement.style.color = variables.textColor || '';
  document.documentElement.style.colorScheme = variables.darkMode ? 'dark' : 'light';
  mermaid.initialize({ ...config, startOnLoad: false, securityLevel: 'strict', suppressErrorRendering: true,
    maxTextSize: 68 * 1024, maxEdges: 500, layout: 'dagre', theme: config.theme || 'base',
    fontFamily: config.fontFamily || 'Helvetica, PingFang TC, sans-serif' });
  const host = document.getElementById('diagram');
  host.replaceChildren();
  await mermaid.parse(source);
  let { svg } = await mermaid.render(`rmdv-${++serial}`, source);
  if (new TextEncoder().encode(svg).length > 4 * 1024 * 1024) throw Error('rendered diagram too large');
  host.innerHTML = svg;
  await document.fonts.ready;
  const element = host.firstElementChild;
  // The app/modal supplies the backdrop. Keep only diagram elements opaque,
  // and retain the transparent root in exported SVG as well as the PNG.
  element.style.setProperty('background-color', 'transparent', 'important');
  svg = element.outerHTML;
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
