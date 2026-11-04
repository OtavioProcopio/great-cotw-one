## Context

Ver proposal.md. Os textos que o app escreve ficam em dicionários; o que o jogo e a Steam já traduzem vem deles, no idioma pedido.

## Decisions

1. **Dicionários tipados**: `pt-BR` é a referência; `en` e `es` são `Record<MessageKey, string>`, então uma chave faltando ou sobrando não compila. Um teste confere que os `{marcadores}` e as quebras de linha são iguais nos três.
2. **Idioma padrão**: o que o jogador escolheu antes (guardado no computador); senão o do sistema (`pt*`, `en*`, `es*`); senão inglês.
3. **Conteúdo vindo do jogo**: o idioma da missão é a posição na tabela de textos do jogo (0 inglês, 3 espanhol, 7 português do Brasil); o das conquistas é a chave do esquema da Steam (`english`, `spanish`, `brazilian`).
4. **Espécies**: uma tabela com o nome em português, inglês e espanhol de cada espécie conhecida; espécie nova vira o identificador legível.
5. **Histórico**: os nomes das funções gravadas no histórico (liberar mapa, editar animais, teste) são traduzidos na hora de mostrar; o resumo guardado fica no idioma em que foi criado.
6. **Preços**: continuam em reais em todos os idiomas, formatados conforme o idioma.

## Risks / Trade-offs

- [Traduções feitas sem revisão de nativos] → tratar como boas, não finais; aceitar correções da comunidade.
- [Resumos do histórico antigos ficam no idioma de antes] → aceito; só o nome da função é traduzido.
- [Nomes de espécies de reservas novas sem tabela] → identificador legível até a tabela crescer.
