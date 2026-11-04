// Emissor de licenças do Great COTW One para rodar no servidor do site (Node 20+, Cloudflare
// Workers, Deno ou Bun: só usa WebCrypto). Gera o mesmo formato que `gco-license issue`:
//   GCO1.<json em base64url>.<assinatura Ed25519 em base64url>
// A chave privada (os 32 bytes do `private.key`, em base64) fica numa variável secreta do
// servidor e NUNCA no navegador, no repositório ou em um site estático.

const PRODUCT = 'great-cotw-one';
const FORMAT_VERSION = 1;
const DAY = 86_400;

/** Dias de validade de cada plano (iguais aos do app, em `license.rs`). */
export const PLAN_DAYS = { monthly: 31, yearly: 366 };

// Prefixo PKCS#8 de uma chave privada Ed25519; depois vêm os 32 bytes da semente.
const PKCS8_PREFIX = Uint8Array.from([
  0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
]);

function base64ToBytes(text) {
  return Uint8Array.from(atob(text.trim()), (c) => c.charCodeAt(0));
}

function base64url(bytes) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '');
}

/**
 * Emite a chave de licença.
 * @param {{ privateKey: string, email: string, plan: 'monthly' | 'yearly', now?: number }} order
 *   `privateKey`: conteúdo do private.key (base64 da semente de 32 bytes).
 *   `now`: segundos desde 1970 (padrão: agora); serve para testes.
 * @returns {Promise<string>}
 */
export async function issueLicense({ privateKey, email, plan, now = Math.floor(Date.now() / 1000) }) {
  const days = PLAN_DAYS[plan];
  if (days === undefined) throw new Error(`plano inválido: ${String(plan)}`);
  if (typeof email !== 'string' || !email.includes('@')) throw new Error('e-mail inválido');
  const seed = base64ToBytes(privateKey);
  if (seed.length !== 32) throw new Error('chave privada com tamanho errado');

  const der = new Uint8Array(PKCS8_PREFIX.length + 32);
  der.set(PKCS8_PREFIX);
  der.set(seed, PKCS8_PREFIX.length);
  const key = await crypto.subtle.importKey('pkcs8', der, 'Ed25519', false, ['sign']);

  const payload = new TextEncoder().encode(
    JSON.stringify({ v: FORMAT_VERSION, p: PRODUCT, l: plan, e: email.trim(), i: now, x: now + days * DAY }),
  );
  const signature = new Uint8Array(await crypto.subtle.sign('Ed25519', key, payload));
  return `GCO1.${base64url(payload)}.${base64url(signature)}`;
}
