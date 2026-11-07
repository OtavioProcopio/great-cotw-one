/** Ícones de traço simples (24 x 24), no mesmo estilo em todas as telas. */
const PATHS = {
  home: 'M3 11.5 12 4l9 7.5M5.5 10v9.5h5v-5h3v5h5V10',
  map: 'M9 4 3.5 6v14L9 18l6 2 5.5-2V4L15 6zM9 4v14M15 6v14',
  unlock: 'M7 11V8a5 5 0 0 1 9.6-2M5.5 11h13v9h-13zM12 14.5v2.5',
  backup: 'M4 12a8 8 0 1 0 2.6-5.9M4 4v4.5h4.5M12 8v4.5l3 1.8',
  settings:
    'M12 8.8a3.2 3.2 0 1 0 0 6.4 3.2 3.2 0 0 0 0-6.4zM19.4 13.5l1.6 1.2-1.7 3-1.9-.7a7 7 0 0 1-1.6.9l-.3 2h-3.4l-.3-2a7 7 0 0 1-1.6-.9l-1.9.7-1.7-3 1.6-1.2a7 7 0 0 1 0-1.9L4.6 10.4l1.7-3 1.9.7a7 7 0 0 1 1.6-.9l.3-2h3.4l.3 2a7 7 0 0 1 1.6.9l1.9-.7 1.7 3-1.6 1.2a7 7 0 0 1 0 1.9z',
  leaf: 'M20 4C10 4 4 10 4 17c0 1 .2 2 .5 3 1 .3 2 .5 3 .5C14.5 20.5 20 14 20 4zM4 20c3-5 7-8 11-10',
  drop: 'M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z',
  moon: 'M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z',
  check: 'm5 12.5 4.5 4.5L19 7.5',
  alert: 'M12 8v5M12 16.5v.01M10.3 4.5 3 18a2 2 0 0 0 1.7 3h14.6a2 2 0 0 0 1.7-3L13.7 4.5a2 2 0 0 0-3.4 0z',
  folder: 'M3.5 7.5a2 2 0 0 1 2-2H10l2 2.5h6.5a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z',
  game: 'M7 8h10a4.5 4.5 0 0 1 4.4 5.4l-.7 3.3a2.5 2.5 0 0 1-4.2 1.2L15 16.5H9L7.5 17.9a2.5 2.5 0 0 1-4.2-1.2l-.7-3.3A4.5 4.5 0 0 1 7 8zM8 10.5v3M6.5 12h3M16 11.5v.01M18 13.5v.01',
  save: 'M5 4h11l3 3v13H5zM8 4v5h7V4M8 20v-6h8v6',
  refresh: 'M20 12a8 8 0 1 1-2.5-5.8M20 4v4.5h-4.5',
  back: 'M14.5 5.5 8 12l6.5 6.5',
  star: 'm12 3.5 2.6 5.4 5.9.8-4.3 4.1 1 5.8L12 16.8l-5.2 2.8 1-5.8-4.3-4.1 5.9-.8z',
  plus: 'M12 5v14M5 12h14',
  minus: 'M5 12h14',
  fit: 'M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5',
  clock: 'M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16zM12 8v4.5l3 1.5',
  paw: 'M8.5 11.5c-1.2 0-2-1-2-2.3s.8-2.2 2-2.2 2 1 2 2.3-.8 2.2-2 2.2zM15.5 11.5c-1.2 0-2-1-2-2.3s.8-2.2 2-2.2 2 1 2 2.3-.8 2.2-2 2.2zM4.5 15c-1 0-1.7-.8-1.7-1.9s.7-1.8 1.7-1.8 1.7.8 1.7 1.9S5.5 15 4.5 15zM19.5 15c-1 0-1.7-.8-1.7-1.9s.7-1.8 1.7-1.8 1.7.8 1.7 1.9-.7 1.8-1.7 1.8zM12 12.5c-3 0-5.5 3-5.5 5.2 0 1.4 1.2 2 2.6 2 1 0 1.9-.5 2.9-.5s1.9.5 2.9.5c1.4 0 2.6-.6 2.6-2 0-2.2-2.5-5.2-5.5-5.2z',
  shield: 'M12 3.5 5 6v5.5c0 4.2 3 7.5 7 9 4-1.5 7-4.8 7-9V6zM9 12l2.2 2.2L15.5 10',
  sparkle: 'M12 4v4M12 16v4M4 12h4M16 12h4M7 7l2 2M15 15l2 2M17 7l-2 2M9 15l-2 2',
  close: 'M6 6l12 12M18 6 6 18',
  edit: 'M4 20h4L19 9l-4-4L4 16zM13.5 6.5l4 4',
  user: 'M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4.5 20a7.5 7.5 0 0 1 15 0',
  flag: 'M5 21V4M5 5h11l-1.8 3.5L16 12H5',
  trophy: 'M8 4h8v5a4 4 0 0 1-8 0zM8 6H4.5a3 3 0 0 0 3.5 4M16 6h3.5a3 3 0 0 1-3.5 4M12 13v4M8.5 20h7M10 17h4',
  target: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 11.5v1',
  lock: 'M7 11V8a5 5 0 0 1 10 0v3M5.500 11h13v9h-13z',
  list: 'M9 7h11M9 12h11M9 17h11M4.5 7h.01M4.5 12h.01M4.5 17h.01',
  deer: 'M12 20.5c-2 0-3-1.500-3-3.500 0-2 1-3.500 3-3.500s3 1.500 3 3.500c0 2-1 3.500-3 3.500zM9.500 13.500 7.500 8.500 5.500 4M7.500 8.500 4 7.500M14.500 13.500l2-5 2-4.500M16.500 8.500 20 7.500M11 17v.01M13 17v.01',
  bird: 'M4 15.500c3 0 4-3.500 7-3.500 1.500-3 4-4.500 7-4l2 1.500-2 1.200c0 4.500-3 8.500-8.500 8.500-3 0-4.500-1-5.500-3.700zM16.500 9.800v.01M9 19.500l-1.500 2M13 19.500l.5 2',
  canid: 'M5 4l3 4h8l3-4v9.500c0 3.700-3.300 6.500-7 6.500s-7-2.800-7-6.500zM9.500 12v.01M14.500 12v.01M12 15l-1.200 1.500h2.400z',
  cat: 'M5 5l3.500 3h7L19 5v9c0 3.500-3.500 6-7 6s-7-2.500-7-6zM9.500 12v.01M14.500 12v.01M12 14.500v1.200M6.500 15.500l-3 .5M17.500 15.500l3 .5',
  bear: 'M7.500 7.500a2.500 2.500 0 1 0-3-2.500M16.500 7.500a2.500 2.500 0 1 1 3-2.500M12 20.500c-4 0-7-2.500-7-6.500S8 7.500 12 7.500s7 2.500 7 6.500-3 6.500-7 6.500zM9.500 13v.01M14.500 13v.01M11 16.500h2',
  pig: 'M5 8l2-3 3 2h4l3-2 2 3v6c0 3-3 6-7 6s-7-3-7-6zM9 11.500v.01M15 11.500v.01M9.500 15.500h5M8 17.500l-1 2.500M16 17.500l1 2.500',
  bull: 'M3 7c0 3.500 2 4.500 4 4.500M21 7c0 3.500-2 4.500-4 4.500M7 11.500c0-2 1.500-3.500 5-3.500s5 1.500 5 3.500v4.500c0 2.500-2 4-5 4s-5-1.500-5-4zM10 13.500v.01M14 13.500v.01M10.500 17.500h3',
  horn: 'M8 9.500C5 9.500 3.500 6.500 5.500 4.500M16 9.500c3 0 4.500-3 2.500-5M8 9.500h8v7c0 2.500-2 4-4 4s-4-1.500-4-4zM10 13v.01M14 13v.01M12 16.500v2',
  hare: 'M9 3.500c-1.500 0-2 2-2 4.500v3M15 3.500c1.500 0 2 2 2 4.500v3M7 11c-2 1.200-3 3-3 5.500 0 2.500 3.500 3.500 8 3.500s8-1 8-3.500c0-2.500-1-4.300-3-5.500zM10 15v.01M14 15v.01',
  croc: 'M3 15h12l2-2.500 4 1.500-3 1.500V18H3zM6.500 15v-2M9.500 15v-2M12.500 15v-2M16.500 13.200v.01',
  backpack: 'M9 7V5.5a3 3 0 0 1 6 0V7M6.5 7h11a1.5 1.5 0 0 1 1.5 1.5V19a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 5 19V8.5A1.5 1.5 0 0 1 6.5 7zM9 12h6M9 15.5h6',
} as const;

export type IconName = keyof typeof PATHS;

/** Caminho do ícone, no mesmo formato que o canvas do mapa desenha com `Path2D`. */
export function iconPath(name: IconName): string {
  return PATHS[name];
}

/** SVG do ícone, com a cor do texto ao redor. Decorativo: leitores de tela o ignoram. */
export function icon(name: IconName, size = 20): string {
  return `<svg class="icon" width="${String(size)}" height="${String(size)}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false"><path d="${PATHS[name]}"/></svg>`;
}

/** Ícone de uma espécie, pela classe que o jogo dá a ela (`deer`, `cat`, `duck`…). */
export function animalIcon(group: string): IconName {
  switch (group) {
    case 'deer':
    case 'antelope':
      return 'deer';
    case 'duck':
    case 'goose':
    case 'grouse':
    case 'landfowl':
    case 'quail':
    case 'turkey':
      return 'bird';
    case 'canid':
    case 'wolf':
    case 'fox':
      return 'canid';
    case 'cat':
    case 'lynx':
      return 'cat';
    case 'bear':
      return 'bear';
    case 'pig':
      return 'pig';
    case 'bovine':
    case 'bison':
    case 'buffalo':
    case 'cattle':
      return 'bull';
    case 'goat':
    case 'goat antelope':
    case 'sheep':
      return 'horn';
    case 'hare':
    case 'rabbit':
    case 'rodent':
    case 'mustelid':
      return 'hare';
    case 'crocodilian':
      return 'croc';
    default:
      return 'paw';
  }
}
