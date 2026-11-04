# Licenças no site: como ligar pagamento, chave e e-mail

O app confere a licença **sem internet** (assinatura Ed25519, chave pública embutida). Então o
site só precisa fazer uma coisa: **depois do pagamento aprovado, assinar uma chave e mandar ao
comprador**. Tudo o mais (ativar, vencer, avisar 30 dias antes) já está no app.

## O fluxo

```
Comprador ──paga──▶ Plataforma de pagamento ──webhook──▶ Servidor do site (função)
                                                              │ 1. confere a assinatura do webhook
                                                              │ 2. descobre e-mail e plano (R$ 15 mês / R$ 75 ano)
                                                              │ 3. issueLicense(...)  ← chave privada (segredo)
                                                              ▼
                                            E-mail com a chave GCO1.… ──▶ Comprador cola em Premium → Ativar
```

## O que já existe

- `tools/license-issuer/issuer.mjs`: `issueLicense({ privateKey, email, plan })` devolve a chave.
  Só usa WebCrypto, então roda em Node 20+, Cloudflare Workers, Deno e Bun.
- `tools/license-issuer/cli.mjs`: o mesmo, pela linha de comando (para emitir à mão).
- O teste `issuer.test.mjs` e o teste `accepts_the_key_issued_by_the_website_issuer` (em
  `license.rs`) travam o formato nos dois lados com o mesmo vetor. Se alguém mexer no formato,
  os dois testes quebram.

## O que falta (precisa de decisões e contas suas)

1. **Plataforma de pagamento** (com Pix e cartão). Candidatas: Kiwify, Hotmart ou Mercado Pago.
   O que importa: ter **webhook** de "pagamento aprovado" e "assinatura renovada" com o e-mail
   do comprador, e a taxa por venda.
2. **Onde roda a função**: Cloudflare Workers (grátis para esse volume), Vercel ou Netlify.
   A chave privada entra como **variável secreta** (`LICENSE_PRIVATE_KEY` = conteúdo do
   `private.key`). Nunca no repositório, nunca em página estática.
3. **Envio do e-mail**: Resend, Brevo ou o próprio e-mail da plataforma.
4. **Renovação do plano mensal.** A licença é offline e vence em 31 dias; a cada renovação paga,
   o servidor emite **uma chave nova** e manda por e-mail, e o comprador cola de novo. Para o
   mensal isso é um atrito real. Alternativa (mais trabalho): o app pedir a chave nova ao
   servidor com o e-mail e um código, quando houver internet. Decidir depois das primeiras vendas.
5. **Reembolso e fraude**: a chave já emitida continua valendo até vencer. Se for um problema,
   usar o plano mensal como padrão e o anual só com pagamento confirmado.
6. **Dados**: guarde só e-mail, plano e data (para reenviar a chave). Isso é dado pessoal:
   política de privacidade no site.

## Esqueleto da função (Cloudflare Workers)

```js
import { issueLicense } from './issuer.mjs';

export default {
  async fetch(request, env) {
    // 1. CONFIRA a assinatura/segredo do webhook da plataforma antes de qualquer coisa.
    const order = await request.json();            // formato depende da plataforma
    const plan = order.valor >= 75 ? 'yearly' : 'monthly';
    const key = await issueLicense({ privateKey: env.LICENSE_PRIVATE_KEY, email: order.email, plan });
    // 2. Mande `key` por e-mail (Resend/Brevo) e responda 200 à plataforma.
    return new Response('ok');
  },
};
```

## Antes de vender

- Faça **uma cópia segura** de `private.key` (gerenciador de senhas). Perdê-la = não emitir mais.
- Trocar o par de chaves invalida todas as licenças já vendidas.
- Teste ponta a ponta com uma compra de R$ 1 (ou modo de teste da plataforma): pagar → receber o
  e-mail → colar no app → Premium ativo.
