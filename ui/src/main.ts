import './styles.css';

import { tauriApi } from './api';
import { Controller } from './controller';
import { detectLocale, savedLocale, setLocale } from './i18n';

const root = document.getElementById('app');
if (!root) throw new Error('#app ausente');

// `?mock` mostra a interface no navegador comum, com dados de exemplo (só em desenvolvimento).
if (import.meta.env.DEV && new URLSearchParams(location.search).has('mock')) {
  const { installMock } = await import('./mock');
  installMock();
}

setLocale(detectLocale(savedLocale(), navigator.languages));

await new Controller(root, tauriApi).start();
