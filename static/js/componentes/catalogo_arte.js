// catalogo_arte.js: las imágenes de producto de las tarjetas del catálogo.
// Mientras la tienda no suba fotos reales, cada producto se muestra con una imagen de estudio dibujada en SVG
// (laptop, SSD, memoria RAM, pantalla, batería, mouse, teclado, audífonos, cable o caja). Todo es SVG fijo: no entra
// ningún dato del servidor dentro del dibujo, solo se elige cuál dibujar según el tipo de producto.

let contador = 0; // cada dibujo necesita ids de degradado propios para que no se pisen entre tarjetas

const DIBUJOS = {
  laptop(k, v, defs, sombra) {
    let c = '';
    const w=v===1?['#fb923c','#be185d','#4c1d95']:['#38bdf8','#6366f1','#7c3aed'];
    const metal=v===1?['#9aa3b2','#6b7384','#4b5262']:['#f1f4f9','#cfd5df','#aab2c0'];
    c=defs(`<linearGradient id="${k}w" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="${w[0]}"/><stop offset=".55" stop-color="${w[1]}"/><stop offset="1" stop-color="${w[2]}"/></linearGradient>
    <linearGradient id="${k}m" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${metal[0]}"/><stop offset=".6" stop-color="${metal[1]}"/><stop offset="1" stop-color="${metal[2]}"/></linearGradient>
    <linearGradient id="${k}t" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2a2f3a"/><stop offset="1" stop-color="#0d0f14"/></linearGradient>
    <linearGradient id="${k}r" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".35"/><stop offset=".5" stop-color="#fff" stop-opacity="0"/></linearGradient>`)+sombra+
    `<rect x="92" y="40" width="216" height="150" rx="11" fill="url(#${k}t)"/>
     <rect x="100" y="48" width="200" height="134" rx="5" fill="url(#${k}w)"/>
     <path d="M100 150 C150 120 200 170 300 120 L300 182 L100 182Z" fill="#fff" opacity=".13"/>
     <circle cx="236" cy="86" r="30" fill="#fff" opacity=".12"/><circle cx="148" cy="70" r="12" fill="#fff" opacity=".1"/>
     <rect x="100" y="48" width="200" height="134" rx="5" fill="url(#${k}r)"/>
     <circle cx="200" cy="44" r="1.6" fill="#3b4252"/>
     <path d="M70 196 H330 L366 230 H34Z" fill="url(#${k}m)"/>
     <path d="M70 196 H330 L334 200 H66Z" fill="#fff" opacity=".5"/>
     <rect x="172" y="198" width="56" height="5" rx="2.5" fill="#000" opacity=".12"/>
     ${[0,1,2,3].map(r=>`<path d="M${96-r*7} ${206+r*5.5} H${304+r*7}" stroke="#7b8394" stroke-width="3.2" stroke-dasharray="14 3" opacity=".55" stroke-linecap="round"/>`).join('')}
     <path d="M34 230 H366 Q370 238 360 240 H40 Q30 238 34 230Z" fill="${metal[2]}"/>
     <path d="M34 230 H366" stroke="#fff" stroke-opacity=".55"/>`;
    return c;
  },
  ssd(k, v, defs, sombra) {
    let c = '';
    c=defs(`<linearGradient id="${k}p" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2b3140"/><stop offset="1" stop-color="#141821"/></linearGradient>
    <linearGradient id="${k}e" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#e9ecf2"/><stop offset="1" stop-color="#9aa3b2"/></linearGradient>
    <linearGradient id="${k}o" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fde68a"/><stop offset="1" stop-color="#b8860b"/></linearGradient>
    <linearGradient id="${k}c" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#3a4152"/><stop offset="1" stop-color="#0b0d12"/></linearGradient>`)+sombra+
    `<g transform="rotate(-8 200 150)">
     <rect x="52" y="104" width="296" height="84" rx="7" fill="url(#${k}p)"/>
     <rect x="52" y="104" width="296" height="84" rx="7" fill="none" stroke="#fff" stroke-opacity=".18"/>
     ${Array.from({length:12},(_,i)=>`<rect x="40" y="${110+i*6}" width="16" height="3.6" rx="1" fill="url(#${k}o)"/>`).join('')}
     <circle cx="338" cy="146" r="9" fill="#e5e7eb"/><circle cx="338" cy="146" r="9" fill="none" stroke="#9aa3b2"/>
     <rect x="150" y="114" width="58" height="42" rx="4" fill="url(#${k}c)"/><rect x="216" y="114" width="58" height="42" rx="4" fill="url(#${k}c)"/>
     <rect x="150" y="114" width="58" height="9" rx="3" fill="#fff" opacity=".1"/><rect x="216" y="114" width="58" height="9" rx="3" fill="#fff" opacity=".1"/>
     <rect x="72" y="114" width="64" height="44" rx="4" fill="url(#${k}e)"/>
     <rect x="80" y="122" width="34" height="5" rx="2" fill="#1d4ed8"/><rect x="80" y="132" width="46" height="3" rx="1.5" fill="#64748b"/><rect x="80" y="139" width="40" height="3" rx="1.5" fill="#94a3b8"/><rect x="80" y="146" width="22" height="3" rx="1.5" fill="#94a3b8"/>
     </g>`;
    return c;
  },
  ram(k, v, defs, sombra) {
    let c = '';
    c=defs(`<linearGradient id="${k}p" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2f8f5b"/><stop offset="1" stop-color="#1b5e3b"/></linearGradient>
    <linearGradient id="${k}o" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fde68a"/><stop offset="1" stop-color="#b8860b"/></linearGradient>
    <linearGradient id="${k}c" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#3a4152"/><stop offset="1" stop-color="#0b0d12"/></linearGradient>`)+sombra+
    `<g transform="rotate(-5 200 150)">
     <rect x="46" y="100" width="308" height="88" rx="6" fill="url(#${k}p)"/>
     ${Array.from({length:30},(_,i)=>i>=13&&i<=14?'':`<rect x="${52+i*9.6}" y="172" width="6" height="17" fill="url(#${k}o)"/>`).join('')}
     ${[0,1,2,3].map(i=>`<rect x="${70+i*58}" y="110" width="48" height="52" rx="3" fill="url(#${k}c)"/><rect x="${70+i*58}" y="110" width="48" height="8" rx="2" fill="#fff" opacity=".1"/><rect x="${78+i*58}" y="130" width="22" height="3" rx="1.5" fill="#cbd5e1" opacity=".5"/>`).join('')}
     <rect x="300" y="108" width="46" height="22" rx="3" fill="#f1f5f9"/><rect x="305" y="113" width="26" height="4" rx="2" fill="#1d4ed8"/><rect x="305" y="120" width="34" height="3" rx="1.5" fill="#94a3b8"/>
     <rect x="46" y="100" width="308" height="88" rx="6" fill="none" stroke="#fff" stroke-opacity=".25"/>
     </g>`;
    return c;
  },
  pantalla(k, v, defs, sombra) {
    let c = '';
    c=defs(`<linearGradient id="${k}d" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#7dd3fc"/><stop offset=".5" stop-color="#3b82f6"/><stop offset="1" stop-color="#4338ca"/></linearGradient>
    <linearGradient id="${k}r" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".4"/><stop offset=".45" stop-color="#fff" stop-opacity="0"/></linearGradient>
    <linearGradient id="${k}o" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fde68a"/><stop offset="1" stop-color="#b8860b"/></linearGradient>`)+sombra+
    `<rect x="62" y="44" width="276" height="170" rx="7" fill="#12151c"/>
     <rect x="68" y="50" width="264" height="158" rx="3" fill="url(#${k}d)"/>
     <path d="M68 170 C130 140 190 190 332 140 L332 208 L68 208Z" fill="#fff" opacity=".14"/>
     <rect x="68" y="50" width="264" height="158" rx="3" fill="url(#${k}r)"/>
     <rect x="150" y="214" width="100" height="18" rx="2" fill="url(#${k}o)"/>
     ${Array.from({length:14},(_,i)=>`<rect x="${153+i*7}" y="217" width="2.6" height="12" fill="#7a5a00" opacity=".35"/>`).join('')}`;
    return c;
  },
  bateria(k, v, defs, sombra) {
    let c = '';
    c=defs(`<linearGradient id="${k}b2" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3a4152"/><stop offset=".5" stop-color="#1c2029"/><stop offset="1" stop-color="#0b0d12"/></linearGradient>
    <linearGradient id="${k}l" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#e2e8f0"/></linearGradient>`)+sombra+
    `<g transform="rotate(-4 200 150)">
     <rect x="58" y="92" width="284" height="104" rx="14" fill="url(#${k}b2)"/>
     <path d="M70 96 H330" stroke="#fff" stroke-opacity=".28" stroke-width="2" stroke-linecap="round"/>
     <rect x="90" y="112" width="200" height="64" rx="6" fill="url(#${k}l)"/>
     <path d="M90 112 H290 V124 H90Z" fill="#1d4ed8"/>
     <rect x="100" y="134" width="96" height="6" rx="3" fill="#334155"/><rect x="100" y="146" width="120" height="4" rx="2" fill="#94a3b8"/><rect x="100" y="156" width="80" height="4" rx="2" fill="#94a3b8"/>
     <path d="M246 134 l14 -8 v8 l14 -8 v24 l-14 8 v-8 l-14 8Z" fill="#0f172a" opacity=".85" transform="translate(-4 4) scale(.9)"/>
     <rect x="300" y="190" width="40" height="14" rx="3" fill="#e5e7eb"/><rect x="300" y="190" width="40" height="14" rx="3" fill="none" stroke="#9aa3b2"/>
     ${[0,1,2,3,4].map(i=>`<rect x="${305+i*7}" y="194" width="3.5" height="7" fill="#b8860b"/>`).join('')}
     </g>`;
    return c;
  },
  mouse(k, v, defs, sombra) {
    let c = '';
    c=defs(`<linearGradient id="${k}m" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#4b5563"/><stop offset=".5" stop-color="#1f2530"/><stop offset="1" stop-color="#0b0d12"/></linearGradient>
    <radialGradient id="${k}h" cx=".35" cy=".25" r=".7"><stop offset="0" stop-color="#fff" stop-opacity=".4"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></radialGradient>`)+sombra+
    `<g transform="rotate(-10 200 140)">
     <path d="M200 30 C262 30 290 84 290 140 C290 200 258 238 200 238 C142 238 110 200 110 140 C110 84 138 30 200 30Z" fill="url(#${k}m)"/>
     <path d="M200 30 C262 30 290 84 290 140 C290 200 258 238 200 238 C142 238 110 200 110 140 C110 84 138 30 200 30Z" fill="url(#${k}h)"/>
     <path d="M200 32 V108" stroke="#05070a" stroke-width="3"/>
     <path d="M130 112 Q200 126 270 112" stroke="#05070a" stroke-width="3" fill="none"/>
     <rect x="190" y="62" width="20" height="38" rx="9" fill="#e5e7eb"/><rect x="190" y="62" width="20" height="38" rx="9" fill="none" stroke="#6b7280"/>
     <path d="M196 70 V92 M204 70 V92" stroke="#9ca3af" stroke-width="1.5"/>
     <text x="200" y="196" text-anchor="middle" font-family="Inter,sans-serif" font-weight="800" font-size="16" fill="#9ca3af" opacity=".7">M170</text>
     </g>`;
    return c;
  },
  teclado(k, v, defs, sombra) {
    let c = '';
    c=defs(`<linearGradient id="${k}k" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3a4152"/><stop offset="1" stop-color="#171b23"/></linearGradient>
    <linearGradient id="${k}y" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f8fafc"/><stop offset="1" stop-color="#cbd5e1"/></linearGradient>`)+sombra+
    `<path d="M40 100 H360 L378 206 H22Z" fill="url(#${k}k)"/>
     <path d="M40 100 H360 L362 104 H38Z" fill="#fff" opacity=".25"/>
     ${[0,1,2,3,4].map(r=>{const n=r===4?1:r===3?11:13;const sx=r===4?120:0;const out=[];const xl=40-r*3.6, xr=360+r*3.6,y=108+r*18;
       if(r===4){return `<rect x="104" y="${y}" width="190" height="13" rx="3" fill="url(#${k}y)"/><rect x="62" y="${y}" width="36" height="13" rx="3" fill="url(#${k}y)"/><rect x="300" y="${y}" width="36" height="13" rx="3" fill="url(#${k}y)"/>`}
       const wk=(xr-xl-16)/n; for(let i=0;i<n;i++)out.push(`<rect x="${xl+8+i*wk+1.5}" y="${y}" width="${wk-3}" height="13" rx="3" fill="url(#${k}y)"/>`);return out.join('')}).join('')}`;
    return c;
  },

  audio(k, v, defs, sombra) {
    return defs(`<linearGradient id="${k}a" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3a4152"/><stop offset="1" stop-color="#0b0d12"/></linearGradient>
    <linearGradient id="${k}c" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#4b5563"/><stop offset="1" stop-color="#171b23"/></linearGradient>`) + sombra +
    `<path d="M92 160 V140 A108 108 0 0 1 308 140 V160" fill="none" stroke="url(#${k}a)" stroke-width="16" stroke-linecap="round"/>
     <rect x="70" y="128" width="56" height="94" rx="24" fill="url(#${k}c)"/><rect x="274" y="128" width="56" height="94" rx="24" fill="url(#${k}c)"/>
     <rect x="80" y="142" width="26" height="66" rx="13" fill="#0b0d12" opacity=".55"/><rect x="294" y="142" width="26" height="66" rx="13" fill="#0b0d12" opacity=".55"/>
     <path d="M78 140 q10 -12 30 -10" stroke="#fff" stroke-opacity=".3" stroke-width="3" fill="none" stroke-linecap="round"/>`;
  },
  cable(k, v, defs, sombra) {
    return defs(`<linearGradient id="${k}m" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#e5e7eb"/><stop offset="1" stop-color="#9aa3b2"/></linearGradient>`) + sombra +
    `<path d="M104 150 C104 40 296 40 296 130 C296 210 190 220 150 190" fill="none" stroke="#1f2530" stroke-width="12" stroke-linecap="round"/>
     <path d="M104 150 C104 40 296 40 296 130" fill="none" stroke="#fff" stroke-opacity=".18" stroke-width="3" stroke-linecap="round" transform="translate(0 -3)"/>
     <rect x="88" y="150" width="32" height="46" rx="6" fill="url(#${k}m)"/><rect x="96" y="196" width="16" height="14" rx="2" fill="#6b7280"/>
     <rect x="134" y="178" width="40" height="26" rx="5" fill="url(#${k}m)" transform="rotate(32 154 191)"/>`;
  },
  caja(k, v, defs, sombra) {
    return defs(`<linearGradient id="${k}l" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#e7c79a"/><stop offset="1" stop-color="#d3a870"/></linearGradient>
    <linearGradient id="${k}r" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#c9a066"/><stop offset="1" stop-color="#a97f48"/></linearGradient>`) + sombra +
    `<path d="M200 60 L320 100 V200 L200 240 L80 200 V100Z" fill="url(#${k}l)"/>
     <path d="M200 60 L320 100 L200 140 L80 100Z" fill="#f0d9b5"/>
     <path d="M200 140 V240 L320 200 V100Z" fill="url(#${k}r)"/>
     <path d="M178 66 L238 86 V128 L178 108Z" fill="#fff" opacity=".35"/>`;
  },
};

// Tipos que ya tienen dibujo; cualquier otro sale como «caja».
export function tieneDibujo(tipo) {
  return Object.prototype.hasOwnProperty.call(DIBUJOS, tipo);
}

// Devuelve el <svg> del producto. "variante" (0 o 1) cambia colores en los productos que lo usan (las laptops).
export function dibujoDeProducto(tipo, variante = 0) {
  const clave = `arte${++contador}`;
  const nombre = tieneDibujo(tipo) ? tipo : 'caja';
  const sombraCy = nombre === 'laptop' ? 246 : nombre === 'pantalla' ? 238 : 236;
  const sombraRx = nombre === 'laptop' ? 150 : 120;
  const defs = (extra) => `<defs><filter id="${clave}b" x="-20%" y="-200%" width="140%" height="500%"><feGaussianBlur stdDeviation="7"/></filter>${extra || ''}</defs>`;
  const sombra = `<ellipse cx="200" cy="${sombraCy}" rx="${sombraRx}" ry="11" fill="#0f172a" opacity=".16" filter="url(#${clave}b)"/>`;
  const interior = DIBUJOS[nombre](clave, variante, defs, sombra);
  const texto = `<svg xmlns="http://www.w3.org/2000/svg" class="lz-prod-arte" viewBox="0 0 400 270" preserveAspectRatio="xMidYMid meet" aria-hidden="true">${interior}</svg>`;
  return document.importNode(new DOMParser().parseFromString(texto, 'image/svg+xml').documentElement, true);
}

// ---------- Las vistas de la galería de la ficha ----------
// Mientras no haya fotos reales, cada vista es un dibujo: la «frontal» es el dibujo de siempre, la «lateral» y la «trasera» son
// otros ángulos (dibujados aparte en laptops, mouses y discos; en el resto, el mismo dibujo girado) y el «empaque» es una caja.
// Cuando un producto tenga fotos, su vista trae una dirección en lugar de «dibujo:...» y la ficha muestra esa foto.
const SOMBRA_ALT = '<ellipse cx="200" cy="232" rx="125" ry="11" fill="#0f172a" opacity=".14"/>';
const GIROS = {
  lateral: 'perspective(700px) rotateY(-38deg)',
  trasera: 'scaleX(-1) perspective(700px) rotateY(-14deg)',
};
const OTROS_ANGULOS = {
  laptop: {
    lateral:
      SOMBRA_ALT +
      '<path d="M52 196 L348 196 L356 210 Q356 216 348 216 L52 216 Q44 216 44 210 Z" fill="#64748b"/><path d="M52 196 L348 196 L350 200 L50 200Z" fill="#94a3b8"/><path d="M62 196 L124 58 Q128 52 136 54 L146 56 Q150 58 148 64 L92 196Z" fill="#334155"/><rect x="236" y="203" width="26" height="6" rx="3" fill="#0f172a"/><rect x="270" y="203" width="14" height="6" rx="3" fill="#0f172a"/><rect x="292" y="203" width="14" height="6" rx="3" fill="#0f172a"/><circle cx="318" cy="206" r="3.5" fill="#0f172a"/>',
    trasera:
      SOMBRA_ALT +
      '<rect x="66" y="42" width="268" height="176" rx="16" fill="#64748b"/><rect x="66" y="42" width="268" height="176" rx="16" fill="#94a3b8" opacity=".25"/><rect x="66" y="205" width="268" height="13" rx="6" fill="#334155"/><circle cx="200" cy="124" r="19" fill="none" stroke="#e2e8f0" stroke-width="3" opacity=".7"/><path d="M190 124h20M200 114v20" stroke="#e2e8f0" stroke-width="3" opacity=".7" stroke-linecap="round"/>',
  },
  mouse: {
    lateral:
      SOMBRA_ALT +
      '<path d="M86 196 C92 112 168 82 252 96 C322 108 338 152 334 196 Z" fill="#1e293b"/><path d="M96 190 C104 122 170 98 246 108 C300 116 322 146 324 178" fill="none" stroke="#475569" stroke-width="5" stroke-linecap="round" opacity=".7"/><rect x="82" y="196" width="256" height="12" rx="6" fill="#0f172a"/><path d="M150 100 C176 118 236 126 316 128" fill="none" stroke="#0f172a" stroke-width="3" opacity=".6"/>',
    trasera:
      SOMBRA_ALT +
      '<ellipse cx="200" cy="130" rx="112" ry="84" fill="#1e293b"/><ellipse cx="200" cy="130" rx="96" ry="70" fill="none" stroke="#334155" stroke-width="3"/><circle cx="200" cy="128" r="16" fill="#ef4444"/><circle cx="200" cy="128" r="30" fill="#ef4444" opacity=".18"/><rect x="122" y="92" width="30" height="8" rx="4" fill="#475569" transform="rotate(-35 137 96)"/><rect x="248" y="92" width="30" height="8" rx="4" fill="#475569" transform="rotate(35 263 96)"/><rect x="186" y="186" width="28" height="10" rx="5" fill="#475569"/>',
  },
  ssd: {
    lateral:
      SOMBRA_ALT +
      '<rect x="60" y="120" width="280" height="30" rx="6" fill="#1e293b"/><rect x="60" y="120" width="20" height="30" rx="4" fill="#e2b93b"/><path d="M62 124h16M62 130h16M62 136h16M62 142h16" stroke="#92741c" stroke-width="2"/><rect x="322" y="126" width="18" height="18" rx="9" fill="#cbd5e1"/>',
    trasera:
      SOMBRA_ALT +
      '<rect x="60" y="96" width="280" height="78" rx="10" fill="#334155"/><rect x="60" y="96" width="30" height="78" rx="6" fill="#e2b93b"/><path d="M64 104h22M64 112h22M64 120h22M64 128h22M64 136h22M64 144h22M64 152h22M64 160h22" stroke="#92741c" stroke-width="2"/><rect x="112" y="112" width="190" height="46" rx="6" fill="#e2e8f0"/><rect x="124" y="122" width="80" height="8" rx="4" fill="#1d4ed8"/><rect x="124" y="138" width="120" height="6" rx="3" fill="#94a3b8"/>',
  },
};

// El dibujo de una vista: "vista" es «frontal», «lateral», «trasera» o «empaque» (lo que trae la base de datos tras «dibujo:»).
export function vistaDeProducto(tipo, variante, vista) {
  if (vista === 'empaque') return dibujoDeProducto('caja');
  const otro = OTROS_ANGULOS[tipo] && OTROS_ANGULOS[tipo][vista];
  if (otro) {
    const texto = `<svg xmlns="http://www.w3.org/2000/svg" class="lz-prod-arte" viewBox="0 0 400 270" preserveAspectRatio="xMidYMid meet" aria-hidden="true">${otro}</svg>`;
    return document.importNode(new DOMParser().parseFromString(texto, 'image/svg+xml').documentElement, true);
  }
  const dibujo = dibujoDeProducto(tipo, variante);
  if (GIROS[vista]) dibujo.style.transform = GIROS[vista];
  return dibujo;
}
