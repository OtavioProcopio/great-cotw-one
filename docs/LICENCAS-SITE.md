# Venda e licenças: Mercado Pago + servidor na VPS

O app confere a licença **sem internet** (assinatura Ed25519, chave pública embutida). Para vender
é preciso um servidor seu que, **depois do pagamento aprovado, assine uma chave e entregue ao app**.
Ele fica em `tools/license-server` (Node 20+, sem dependências) e usa o emissor de `tools/license-issuer`.

## Como funciona

```
App (Premium → Assinar)                     Servidor (VPS)                    Mercado Pago
  gera um CÓDIGO secreto aleatório
  POST /v1/checkout {plano, código} ───────▶ cria a preferência ─────────────▶ checkout
  ◀──────────── link do checkout ◀───────────
  abre o link no navegador ───────────────────────────────────────────────────▶ jogador paga (Pix/cartão)
                                             ◀──── webhook "pagamento" ────────
                                             1. confere a assinatura do webhook
                                             2. CONSULTA o pagamento na API (não confia no aviso)
                                             3. confere status, valor do plano e o código
                                             4. assina a chave com a chave privada
  GET /v1/claim/<código> (a cada poucos s) ─▶ devolve a chave quando existe
  ativa o Premium sozinho
```

Pontos de segurança:
- A **chave privada** só existe na VPS (variável de ambiente). Nunca vai para o app nem para o GitHub.
- Só quem tem o **código** gerado pelo app busca a chave; saber o e-mail do comprador não basta.
- O servidor **não confia no aviso** do Mercado Pago: confere a assinatura do webhook, consulta o
  pagamento na API e exige `approved` e o valor do plano (um pagamento de R$ 1 não vira anual).
- Aviso repetido não emite outra chave (idempotente pelo id do pagamento).

## Instalar na VPS

1. **Mercado Pago** → Seu negócio → Configurações → Credenciais (ou developers): crie uma aplicação
   de *Checkout Pro* e pegue o **Access Token de produção**. Em *Webhooks*, cadastre a URL
   `https://SEU-DOMINIO/v1/webhook/mercadopago` com o evento **Pagamentos** e copie a **chave
   secreta** da assinatura. Para testar antes de vender, use as credenciais de teste.
2. **Domínio com HTTPS** apontando para a VPS (o Mercado Pago exige HTTPS). O mais simples é o
   Caddy, que gera o certificado sozinho:
   ```
   licencas.seudominio.com {
       reverse_proxy 127.0.0.1:8787
   }
   ```
3. **Código na VPS** (precisa de Node 20 ou mais novo): copie as pastas `tools/license-issuer` e
   `tools/license-server` (ou clone o repositório).
4. **Variáveis** em `/etc/great-cotw-license.env` (permissão `600`, dono do serviço):
   ```
   PUBLIC_URL=https://licencas.seudominio.com
   MP_ACCESS_TOKEN=<access token do Mercado Pago>
   MP_WEBHOOK_SECRET=<chave secreta do webhook>
   LICENSE_PRIVATE_KEY=<conteúdo do private.key, em uma linha>
   DATA_FILE=/var/lib/great-cotw/orders.json
   ```
5. **Serviço** (`/etc/systemd/system/great-cotw-license.service`):
   ```
   [Unit]
   Description=Great COTW One license server
   After=network.target

   [Service]
   EnvironmentFile=/etc/great-cotw-license.env
   ExecStart=/usr/bin/node /opt/great-cotw/tools/license-server/main.mjs
   Restart=always
   User=great-cotw
   StateDirectory=great-cotw

   [Install]
   WantedBy=multi-user.target
   ```
   `systemctl enable --now great-cotw-license` e confira `https://licencas.seudominio.com/health`.
6. **Faça backup de `orders.json`** (é o registro das vendas) e do `private.key`.
7. No app, preencha `LICENSE_SERVER` em `crates/cotw-core/src/purchase.rs` com a URL do servidor e gere o instalador de novo. Para testar sem recompilar, numa compilação de depuração use a variável de ambiente `GREAT_COTW_LICENSE_SERVER`. Enquanto estiver vazio, o botão do plano avisa que a loja ainda não abriu.

## Testar ponta a ponta antes de vender

Com as credenciais de **teste** do Mercado Pago: Premium → Assinar → pagar com um usuário e cartão
de teste → o app deve ativar sozinho. Depois troque para as credenciais de produção e faça uma
compra real pequena (pode reembolsar).

## Decisões que ficaram

- **Plano mensal:** a licença vence em 31 dias. A cada renovação paga, o servidor emite outra
  chave; o app precisa de um novo código para buscá-la (renovação automática exige o app
  consultar o servidor ao abrir; ficou para depois das primeiras vendas).
- **Reembolso/chargeback:** a chave já emitida vale até vencer.
- **Privacidade:** o servidor guarda e-mail do pagamento, plano e data. Publique uma política de
  privacidade e fale de reembolso na página do Premium.
- **Troca do par de chaves** invalida todas as licenças já vendidas.

## Emitir à mão (suporte)

`node tools/license-issuer/cli.mjs <private.key> <e-mail> monthly|yearly` (ou `gco-license issue`).
