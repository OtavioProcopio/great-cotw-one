// Servidor de licenças do Great COTW One (roda na VPS, só com Node 20+ e sem dependências).
//
//   POST /v1/checkout            { plan, token }  -> { url }   cria o pagamento no Mercado Pago
//   POST /v1/webhook/mercadopago                  -> 200       aviso do Mercado Pago (conferido na API)
//   GET  /v1/claim/<token>                        -> { status: 'pending' | 'ready', key?, plan? }
//   GET  /v1/obrigado                             -> página "volte ao app"
//   GET  /health
//
// O `token` é um segredo aleatório que o app gera antes de abrir o checkout; só quem o tem
// consegue buscar a chave. A chave privada de assinatura nunca sai desta máquina.

import { createHmac, timingSafeEqual } from 'node:crypto';
import { existsSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';

import { PLAN_DAYS, issueLicense } from '../license-issuer/issuer.mjs';

const MP_API = 'https://api.mercadopago.com';
const TOKEN_PATTERN = /^[A-Za-z0-9_-]{22,64}$/;

/** Preços em reais; o valor pago tem de ser pelo menos este. */
export const DEFAULT_PRICES = { monthly: 15, yearly: 75 };

const TITLES = {
  monthly: 'Great COTW One Premium (mensal)',
  yearly: 'Great COTW One Premium (anual)',
};

const THANKS = `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Obrigado</title>
<body style="font-family:system-ui;max-width:32rem;margin:4rem auto;padding:0 1rem"><h1>Obrigado!</h1>
<p>Volte ao <strong>Great COTW One</strong>: o Premium é ativado sozinho assim que o pagamento for confirmado (o Pix é na hora).</p></body>`;

/** Guarda as vendas num arquivo JSON (troca atômica). Serve bem para centenas de vendas. */
export class FileStore {
  constructor(path) {
    this.path = path;
    this.orders = path && existsSync(path) ? JSON.parse(readFileSync(path, 'utf8')) : {};
  }

  byPayment(paymentId) {
    return Object.values(this.orders).find((o) => o.paymentId === paymentId);
  }

  byToken(token) {
    return this.orders[token];
  }

  save(order) {
    this.orders[order.token] = order;
    if (!this.path) return;
    const temp = `${this.path}.tmp`;
    writeFileSync(temp, JSON.stringify(this.orders, null, 2));
    renameSync(temp, this.path);
  }
}

/** Confere a assinatura `x-signature` que o Mercado Pago manda nos webhooks. */
export function validSignature({ secret, signature, requestId, dataId }) {
  const parts = Object.fromEntries(
    String(signature ?? '')
      .split(',')
      .map((p) => p.trim().split('=')),
  );
  if (!parts.ts || !parts.v1) return false;
  const id = /^[A-Za-z0-9]+$/.test(dataId) ? dataId.toLowerCase() : dataId;
  const manifest = `id:${id};request-id:${requestId ?? ''};ts:${parts.ts};`;
  const expected = createHmac('sha256', secret).update(manifest).digest('hex');
  const given = Buffer.from(String(parts.v1));
  const want = Buffer.from(expected);
  return given.length === want.length && timingSafeEqual(given, want);
}

function json(response, status, body) {
  response.writeHead(status, { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store' });
  response.end(JSON.stringify(body));
}

async function readBody(request, limit = 8_192) {
  let size = 0;
  const chunks = [];
  for await (const chunk of request) {
    size += chunk.length;
    if (size > limit) throw Object.assign(new Error('corpo grande demais'), { status: 413 });
    chunks.push(chunk);
  }
  const text = Buffer.concat(chunks).toString('utf8');
  return text === '' ? {} : JSON.parse(text);
}

/**
 * Monta o tratador de requisições. Tudo que tem efeito externo (Mercado Pago, relógio, arquivo)
 * entra por parâmetro, para os testes não tocarem a rede.
 */
export function createHandler({
  env,
  store,
  fetchImpl = fetch,
  now = () => Math.floor(Date.now() / 1000),
  prices = DEFAULT_PRICES,
  log = () => undefined,
}) {
  const publicUrl = env.PUBLIC_URL.replace(/\/$/, '');
  const mp = (path, init = {}) =>
    fetchImpl(`${MP_API}${path}`, {
      ...init,
      headers: { authorization: `Bearer ${env.MP_ACCESS_TOKEN}`, 'content-type': 'application/json', ...init.headers },
    });

  const hits = new Map(); // limite simples por IP e minuto
  const limited = (ip) => {
    const minute = Math.floor(now() / 60);
    const entry = hits.get(ip);
    if (entry?.minute === minute) entry.count += 1;
    else hits.set(ip, { minute, count: 1 });
    return (hits.get(ip)?.count ?? 0) > 60;
  };

  async function checkout(request, response) {
    const { plan, token } = await readBody(request);
    if (!(plan in PLAN_DAYS) || !TOKEN_PATTERN.test(String(token))) {
      return json(response, 400, { error: 'pedido inválido' });
    }
    const res = await mp('/checkout/preferences', {
      method: 'POST',
      headers: { 'x-idempotency-key': `${plan}-${token}` },
      body: JSON.stringify({
        items: [{ title: TITLES[plan], quantity: 1, currency_id: 'BRL', unit_price: prices[plan] }],
        external_reference: `${plan}:${token}`,
        notification_url: `${publicUrl}/v1/webhook/mercadopago`,
        back_urls: {
          success: `${publicUrl}/v1/obrigado`,
          pending: `${publicUrl}/v1/obrigado`,
          failure: `${publicUrl}/v1/obrigado`,
        },
        auto_return: 'approved',
      }),
    });
    if (!res.ok) {
      log('checkout recusado pelo Mercado Pago', res.status);
      return json(response, 502, { error: 'não foi possível abrir o pagamento agora' });
    }
    const preference = await res.json();
    return json(response, 200, { url: preference.init_point });
  }

  async function webhook(request, response, url) {
    const dataId = url.searchParams.get('data.id') ?? url.searchParams.get('id') ?? '';
    const type = url.searchParams.get('type') ?? url.searchParams.get('topic') ?? '';
    const valid = validSignature({
      secret: env.MP_WEBHOOK_SECRET,
      signature: request.headers['x-signature'],
      requestId: request.headers['x-request-id'],
      dataId,
    });
    if (!valid) return json(response, 401, { error: 'assinatura inválida' });
    if (type !== 'payment' || dataId === '') return json(response, 200, { ignored: true });

    // Nunca confiar no corpo do aviso: o pagamento é consultado na API do Mercado Pago.
    const res = await mp(`/v1/payments/${encodeURIComponent(dataId)}`);
    if (!res.ok) return json(response, 502, { error: 'tente de novo' }); // o Mercado Pago repete
    const payment = await res.json();
    if (payment.status !== 'approved') return json(response, 200, { status: payment.status });

    const [plan, token] = String(payment.external_reference ?? '').split(':');
    const paid = Number(payment.transaction_amount);
    if (!(plan in PLAN_DAYS) || !TOKEN_PATTERN.test(token ?? '') || !(paid >= prices[plan])) {
      log('pagamento aprovado mas fora do esperado', dataId);
      return json(response, 200, { ignored: true });
    }
    const paymentId = String(payment.id);
    if (store.byPayment(paymentId)) return json(response, 200, { duplicated: true });

    const email = String(payment.payer?.email ?? '').trim();
    const key = await issueLicense({ privateKey: env.LICENSE_PRIVATE_KEY, email, plan, now: now() });
    store.save({ token, paymentId, plan, email, key, createdAt: now() });
    log('licença emitida', plan, paymentId);
    return json(response, 200, { issued: true });
  }

  function claim(response, token, ip) {
    if (limited(ip) || !TOKEN_PATTERN.test(token)) return json(response, 404, { status: 'unknown' });
    const order = store.byToken(token);
    return json(response, 200, order ? { status: 'ready', key: order.key, plan: order.plan } : { status: 'pending' });
  }

  return async (request, response) => {
    try {
      const url = new URL(request.url ?? '/', publicUrl);
      const forwarded = String(request.headers['x-forwarded-for'] ?? request.socket.remoteAddress ?? '');
      const ip = forwarded.split(',')[0]?.trim() ?? '';
      const { method } = request;
      if (method === 'GET' && url.pathname === '/health') return json(response, 200, { ok: true });
      if (method === 'POST' && url.pathname === '/v1/checkout') return await checkout(request, response);
      if (method === 'POST' && url.pathname === '/v1/webhook/mercadopago') {
        return await webhook(request, response, url);
      }
      if (method === 'GET' && url.pathname.startsWith('/v1/claim/')) {
        return claim(response, url.pathname.slice('/v1/claim/'.length), ip);
      }
      if (method === 'GET' && url.pathname === '/v1/obrigado') {
        response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
        return response.end(THANKS);
      }
      return json(response, 404, { error: 'não encontrado' });
    } catch (error) {
      log('erro', error instanceof Error ? error.message : error);
      return json(response, error?.status ?? 500, { error: 'erro no servidor' });
    }
  };
}

/** Sobe o servidor com as variáveis de ambiente (ver docs/LICENCAS-SITE.md). */
export function start(env = process.env) {
  for (const name of ['MP_ACCESS_TOKEN', 'MP_WEBHOOK_SECRET', 'LICENSE_PRIVATE_KEY', 'PUBLIC_URL']) {
    if (!env[name]) throw new Error(`falta a variável de ambiente ${name}`);
  }
  const store = new FileStore(env.DATA_FILE ?? './orders.json');
  const handler = createHandler({ env, store, log: (...args) => console.log(new Date().toISOString(), ...args) });
  const server = createServer(handler);
  server.listen(Number(env.PORT ?? 8787), '127.0.0.1', () => console.log('servidor de licenças no ar'));
  return server;
}
