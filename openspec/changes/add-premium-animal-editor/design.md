## Context

Ver proposal.md. Protótipo funcional: aba "Editor de animais" da Central do Caçador (`research/reference/core.js`). Registro do animal e regras na seção 3.1 da skill. Lição registrada: inserir animais sem reconstruir a cadeia de relocação travou o jogo; o protótipo já corrige e verifica 0 quebras.

## Goals / Non-Goals

**Goals:** edição no mesmo tamanho como caminho principal; criação de animais só com verificação estrutural completa.
**Non-Goals:** mudar posição de animais (não existe no save), criar grupos ou populações novas, editar animais no multiplayer.

## Decisions

1. **Faixas**: diamante e Great One a partir de tabela de faixas por espécie (fonte comunitária APC, versionada no app como dado próprio de faixas numéricas) + fallback do Great One real do save.
2. **Pelagem**: só sementes conhecidas por pelagem; a semente atual desconhecida aparece como "sorteada pelo jogo".
3. **Criação**: clonar 32 bytes de um animal do mesmo grupo; inserir ao fim do array; recalcular todos os ponteiros posteriores, contagens, `next` de toda a cadeia, tamanho da instância e offsets do cabeçalho ADF; limite = maior grupo da espécie em todos os saves lidos.
4. **Licença**: serviço externo de licenças (Lemon Squeezy, Paddle ou Keygen) com chave assinada guardada no Windows Credential Manager; verificação da assinatura offline.

## Risks / Trade-offs

- [Animal criado ainda travar o jogo por outra regra interna] → função experimental, desligada por padrão, liga por usuário com aviso; telemetria zero, então coletar relatos manualmente.
- [Pirataria da licença] → aceitar risco moderado; o foco é não atrapalhar quem pagou.
- [Faixas desatualizadas após patch] → faixas editáveis por atualização de dados sem nova versão do app.

## Open Questions

- Fornecedor de pagamento/licença e preço (não muda specs).
