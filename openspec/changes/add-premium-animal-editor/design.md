## Context

Ver proposal.md. Protótipo funcional: aba "Editor de animais" da Central do Caçador (`research/reference/core.js`). Registro do animal e regras na seção 3.1 da skill. Lição registrada: inserir animais sem reconstruir a cadeia de relocação travou o jogo; o protótipo já corrige e verifica 0 quebras.

## Goals / Non-Goals

**Goals:** edição no mesmo tamanho, só de animais que já existem; pelagens escolhidas pela lista do jogo.
**Non-Goals:** criar ou remover animais, grupos ou populações; mudar a posição de animais (não existe no save); editar animais no multiplayer.

## Decisions

1. **Faixas**: diamante e Great One a partir de tabela de faixas por espécie (fonte comunitária APC, versionada no app como dado próprio de faixas numéricas) + fallback do Great One real do save.
2. **Sementes**: só sementes conhecidas por pelagem; a semente atual desconhecida aparece como "sorteada pelo jogo".
3. **Equilíbrio**: as faixas aceitas de peso e pontuação vêm dos animais da espécie no próprio save, por sexo (menor e maior valor, mais 10% de folga), então a edição nunca cria um animal diferente do que o jogo já gera; ao trocar de sexo o painel sugere os valores típicos (mediana) do sexo novo. A regra vale no serviço do app, não só na tela.
3b. **Pelagem**: a escolha troca só a semente do animal; as pelagens vêm da mesma fonte do guia de pelagens (`global_animal_types.blo`), então o editor e o guia nunca divergem.
4. **Bloqueio duplo**: a interface mostra o que é premium, mas quem decide é o serviço do app (`premiumRequired`), então contornar a tela não grava nada.
5. **Licença offline por plano**: a chave é um texto assinado (Ed25519) com plano (`monthly` ou `yearly`), e-mail do comprador e datas de emissão e vencimento; a chave pública fica no app e a privada só com o vendedor, fora do repositório. O app verifica a assinatura e a data sem internet e guarda a chave no computador. Quem emite a chave é o canal de venda (webhook de pagamento → função que assina e envia por e-mail); no começo pode ser manual, com a ferramenta `gco-license` (`issue <private.key> <e-mail> monthly|yearly`). Preços: R$ 15,00 por mês ou R$ 75,00 por ano; renovação = nova chave.
6. **Tela de vendas**: o Premium tem uma página própria (menu, bloqueios e avisos levam a ela) que mostra vantagens, comparação função por função, preços e a economia do anual. Os números vêm de uma configuração única (`ui/src/config.ts`) e a comparação marca "em breve" o que ainda não existe.

## Risks / Trade-offs

- [Pelagem escolhida não aparecer no jogo] → conferir no jogo com vários saves antes de liberar; edição no mesmo tamanho limita o risco.
- [Pirataria da licença] → aceitar risco moderado; o foco é não atrapalhar quem pagou.
- [Faixas desatualizadas após patch] → faixas editáveis por atualização de dados sem nova versão do app.

## Open Questions

- Canal de venda e meio de entrega da chave (não muda as specs: o app só precisa de uma chave assinada válida).

## Decisão: nada de lote

Ajustar o grupo todo de uma vez (ex.: todos os animais no maior valor) foi construído e depois removido por decisão do dono: vários diamantes num mesmo grupo desequilibram a caça e viram algo apelativo. A edição fica animal por animal, sempre dentro das faixas do jogo.
