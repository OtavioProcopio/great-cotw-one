import assert from 'node:assert/strict';
import { test } from 'node:test';

import { issueLicense } from './issuer.mjs';

// Vetor compartilhado com o teste em Rust (`license.rs`): a assinatura Ed25519 é determinística,
// então os dois lados têm de produzir e aceitar exatamente este texto.
const SEED = Buffer.alloc(32, 7).toString('base64');
const VECTOR = 'GCO1.eyJ2IjoxLCJwIjoiZ3JlYXQtY290dy1vbmUiLCJsIjoibW9udGhseSIsImUiOiJ2ZXRvckBleGVtcGxvLmNvbSIsImkiOjE3MDAwMDAwMDAsIngiOjE3MDI2Nzg0MDB9.cJs0lhVWpgw6VByJ6iRmjwVSrVA2JFzMYoRuRtr27HTxhBS-Hp6jDGCT92EkdQc3tH11LrtwBF69NEZ0W9xtDg';

test('gera exatamente a chave do vetor', async () => {
  const key = await issueLicense({ privateKey: SEED, email: 'vetor@exemplo.com', plan: 'monthly', now: 1_700_000_000 });
  assert.equal(key, VECTOR);
});

test('recusa plano, e-mail e chave inválidos', async () => {
  await assert.rejects(issueLicense({ privateKey: SEED, email: 'a@b.c', plan: 'weekly' }), /plano/);
  await assert.rejects(issueLicense({ privateKey: SEED, email: 'sem-arroba', plan: 'yearly' }), /e-mail/);
  await assert.rejects(issueLicense({ privateKey: 'AAAA', email: 'a@b.c', plan: 'yearly' }), /tamanho/);
});

test('o plano anual vence 366 dias depois e o mensal 31', async () => {
  const at = async (plan) => {
    const key = await issueLicense({ privateKey: SEED, email: 'a@b.c', plan, now: 1000 });
    const body = JSON.parse(Buffer.from(key.split('.')[1], 'base64url').toString());
    return body.x - body.i;
  };
  assert.equal(await at('yearly'), 366 * 86_400);
  assert.equal(await at('monthly'), 31 * 86_400);
});
