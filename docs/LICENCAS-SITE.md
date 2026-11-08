# Venda e licenças

O servidor de licenças, o deploy na VPS, a operação (suporte, reembolso, contabilidade, incidentes) e o
checklist de lançamento ficam no repositório privado **`great-cotw-one-platform`**:

- `license-server/`: Node 22, sem dependências. Cria o pagamento no Mercado Pago, confere o webhook e o
  pagamento na API, assina a chave Ed25519 e entrega ao app.
- `deploy/`: Caddy (HTTPS), serviço systemd isolado, script de instalação e backup diário.
- `docs/ARQUITETURA.md`, `docs/LANCAMENTO.md`, `docs/OPERACAO.md`.

## O que o app espera do servidor
- `POST /v1/checkout {plan, token}` → `{ url }` (só aceita `https://`).
- `GET /v1/claim/<token>` → `{ status: 'pending' }`, `{ status: 'ready', key, plan }` ou outro status (tratado como recusa).
- O endereço entra **na compilação** pela variável `GREAT_COTW_LICENSE_SERVER` (no CI, variável do repositório).
  Em compilação de depuração ela também vale em tempo de execução, para testar com um servidor local.
- Formato da chave: `GCO1.<json base64url>.<assinatura Ed25519>`; o vetor de teste é o mesmo nos dois lados
  (`crates/cotw-core/src/license.rs` e `license-server/test/issuer.test.mjs`).

## Emitir à mão
`gco-license issue <private.key> <e-mail> monthly|yearly` (este repositório) ou, na VPS,
`bin/admin.mjs issue <e-mail> <plano>` (registra a venda manual).
