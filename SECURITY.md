# Segurança

## Como reportar uma falha
Escreva para **[e-mail de segurança a definir]** com o que você viu, os passos para reproduzir e a versão do app. Não abra uma issue pública. Respondemos em até 5 dias úteis e avisamos quando houver correção.

## O que o app protege
- **Seu save:** antes de qualquer gravação o app guarda uma cópia completa; nunca grava com o jogo aberto; mostra a mudança antes de aplicar e relê o arquivo depois para conferir.
- **Seus dados:** o app roda no seu computador e não envia save, caminhos, nome de usuário nem estatísticas. A única conversa com a internet é a compra do Premium (ver `docs/legal/PRIVACIDADE.md`).
- **Licença:** assinatura Ed25519 conferida sem internet. A chave privada de emissão existe só no servidor de licenças (variável de ambiente) e nunca entra neste repositório.
- **Janela do app:** política de conteúdo (CSP) restrita a recursos locais, permissões mínimas do Tauri (comandos do app e diálogos de pasta/confirmação) e links de pagamento aceitos só em `https://`.
- **Servidor de licenças:** confere a assinatura do webhook do Mercado Pago, consulta o pagamento na API antes de emitir, limita o tamanho do corpo e as tentativas por IP.

## Práticas do projeto
- Dependências vigiadas pelo Dependabot e por `cargo audit` e `npm audit` no CI.
- Segredos só como variáveis de ambiente do servidor ou segredos do GitHub; nunca no código, nos testes nem nos logs.
- Nenhum arquivo do jogo é distribuído com o app.
