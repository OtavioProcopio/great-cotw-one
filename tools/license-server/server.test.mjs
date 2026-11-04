import assert from 'node:assert/strict';
import { createHmac } from 'node:crypto';
import { once } from 'node:events';
import { createServer } from 'node:http';
import { test } from 'node:test';

import { FileStore, createHandler, validSignature } from './server.mjs';

const SECRET = 'segredo-do-webhook';
const SEED = Buffer.alloc(32, 7).toString('base64');
const TOKEN = 'abcdefghijklmnopqrstuv0123456789';
const NOW = 1_700_000_000;

function sign(dataId, requestId = 'req-1', ts = '1700000000') {
  const v1 = createHmac('sha256', SECRET).update(`id:${dataId};request-id:${requestId};ts:${ts};`).digest('hex');
  return { 'x-signature': `ts=${ts},v1=${v1}`, 'x-request-id': requestId };
}

/** Sobe o servidor com um "Mercado Pago" falso e devolve um cliente. */
async function setup({ payments = {}, preferenceOk = true } = {}) {
  const calls = [];
  const fetchImpl = (url, init = {}) => {
    calls.push({ url: String(url), init });
    const payment = /\/v1\/payments\/(.+)$/.exec(String(url));
    if (String(url).endsWith('/checkout/preferences')) {
      return Promise.resolve(
        preferenceOk
          ? Response.json({ init_point: 'https://mp.example/checkout/123' })
          : new Response('{}', { status: 500 }),
      );
    }
    const found = payment ? payments[payment[1]] : undefined;
    return Promise.resolve(found ? Response.json(found) : new Response('{}', { status: 404 }));
  };
  const store = new FileStore(null);
  const handler = createHandler({
    env: {
      PUBLIC_URL: 'https://licencas.exemplo.com',
      MP_ACCESS_TOKEN: 'TOKEN',
      MP_WEBHOOK_SECRET: SECRET,
      LICENSE_PRIVATE_KEY: SEED,
    },
    store,
    fetchImpl,
    now: () => NOW,
  });
  const server = createServer(handler);
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const base = `http://127.0.0.1:${server.address().port}`;
  return { base, calls, store, close: () => server.close() };
}

const approved = (over = {}) => ({
  id: 777,
  status: 'approved',
  external_reference: `yearly:${TOKEN}`,
  transaction_amount: 75,
  payer: { email: 'comprador@exemplo.com' },
  ...over,
});

const hook = (base, id, headers) =>
  fetch(`${base}/v1/webhook/mercadopago?type=payment&data.id=${id}`, { method: 'POST', headers, body: '{}' });

test('o checkout cria a preferência com a referência, o valor e o aviso', async () => {
  const s = await setup();
  try {
    const res = await fetch(`${s.base}/v1/checkout`, {
      method: 'POST',
      body: JSON.stringify({ plan: 'yearly', token: TOKEN }),
    });
    assert.deepEqual(await res.json(), { url: 'https://mp.example/checkout/123' });
    const sent = JSON.parse(s.calls[0].init.body);
    assert.equal(sent.external_reference, `yearly:${TOKEN}`);
    assert.equal(sent.items[0].unit_price, 75);
    assert.equal(sent.notification_url, 'https://licencas.exemplo.com/v1/webhook/mercadopago');
    assert.equal(s.calls[0].init.headers.authorization, 'Bearer TOKEN');
  } finally {
    s.close();
  }
});

test('o checkout recusa plano ou código inválidos e avisa quando o Mercado Pago falha', async () => {
  const s = await setup();
  const bad = await setup({ preferenceOk: false });
  try {
    for (const body of [{ plan: 'weekly', token: TOKEN }, { plan: 'yearly', token: 'curto' }, {}]) {
      const res = await fetch(`${s.base}/v1/checkout`, { method: 'POST', body: JSON.stringify(body) });
      assert.equal(res.status, 400);
    }
    const res = await fetch(`${bad.base}/v1/checkout`, { method: 'POST', body: JSON.stringify({ plan: 'monthly', token: TOKEN }) });
    assert.equal(res.status, 502);
  } finally {
    s.close();
    bad.close();
  }
});

test('pagamento aprovado emite a chave, que o app busca com o código', async () => {
  const s = await setup({ payments: { 777: approved() } });
  try {
    assert.deepEqual(await (await fetch(`${s.base}/v1/claim/${TOKEN}`)).json(), { status: 'pending' });
    assert.equal((await hook(s.base, 777, sign('777'))).status, 200);
    const claim = await (await fetch(`${s.base}/v1/claim/${TOKEN}`)).json();
    assert.equal(claim.status, 'ready');
    assert.equal(claim.plan, 'yearly');
    const body = JSON.parse(Buffer.from(claim.key.split('.')[1], 'base64url').toString());
    assert.equal(body.e, 'comprador@exemplo.com');
    assert.equal(body.x - body.i, 366 * 86_400);
  } finally {
    s.close();
  }
});

test('aviso repetido não emite outra chave', async () => {
  const s = await setup({ payments: { 777: approved() } });
  try {
    await hook(s.base, 777, sign('777'));
    const first = (await (await fetch(`${s.base}/v1/claim/${TOKEN}`)).json()).key;
    const again = await (await hook(s.base, 777, sign('777', 'req-2'))).json();
    assert.deepEqual(again, { duplicated: true });
    assert.equal((await (await fetch(`${s.base}/v1/claim/${TOKEN}`)).json()).key, first);
  } finally {
    s.close();
  }
});

test('aviso com assinatura errada ou sem assinatura é recusado e nada é emitido', async () => {
  const s = await setup({ payments: { 777: approved() } });
  try {
    assert.equal((await hook(s.base, 777, {})).status, 401);
    const forged = { 'x-signature': 'ts=1700000000,v1=00', 'x-request-id': 'req-1' };
    assert.equal((await hook(s.base, 777, forged)).status, 401);
    assert.equal(s.calls.length, 0, 'nem consulta o Mercado Pago');
    assert.equal(s.store.byToken(TOKEN), undefined);
  } finally {
    s.close();
  }
});

test('só pagamento aprovado, com o valor do plano, vira licença', async () => {
  const s = await setup({
    payments: {
      1: approved({ id: 1, status: 'pending' }),
      2: approved({ id: 2, transaction_amount: 1 }), // pagou R$ 1 pelo plano anual
      3: approved({ id: 3, external_reference: 'qualquer-coisa' }),
    },
  });
  try {
    for (const id of ['1', '2', '3']) assert.equal((await hook(s.base, id, sign(id))).status, 200);
    assert.equal(s.store.byToken(TOKEN), undefined);
  } finally {
    s.close();
  }
});

test('se o Mercado Pago não responde, o aviso falha para ele tentar de novo', async () => {
  const s = await setup({ payments: {} });
  try {
    assert.equal((await hook(s.base, 999, sign('999'))).status, 502);
  } finally {
    s.close();
  }
});

test('buscar com código inválido ou desconhecido não revela nada', async () => {
  const s = await setup();
  try {
    assert.equal((await fetch(`${s.base}/v1/claim/curto`)).status, 404);
    assert.equal((await (await fetch(`${s.base}/v1/claim/${'x'.repeat(30)}`)).json()).status, 'pending');
    assert.equal((await fetch(`${s.base}/health`)).status, 200);
    assert.equal((await fetch(`${s.base}/nada`)).status, 404);
  } finally {
    s.close();
  }
});

test('a assinatura confere o manifesto id;request-id;ts do Mercado Pago', () => {
  // O Mercado Pago assina o id em minúsculas, mesmo que a URL traga maiúsculas.
  const headers = sign('abc123');
  const check = (secret, dataId) =>
    validSignature({ secret, signature: headers['x-signature'], requestId: 'req-1', dataId });
  assert.equal(check(SECRET, 'ABC123'), true);
  assert.equal(check(SECRET, 'abc123'), true);
  assert.equal(check('outro', 'abc123'), false);
  assert.equal(check(SECRET, 'abc124'), false);
  assert.equal(validSignature({ secret: SECRET, signature: undefined, requestId: 'req-1', dataId: 'x' }), false);
});
