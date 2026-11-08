## ADDED Requirements

### Requirement: Arsenal por categoria
O app SHALL mostrar os equipamentos do jogador por categoria (armas, munição, miras, iscas e chamados, equipamentos de campo, veículos), com ícone, nome, posse e quantidade, e filtro "tenho / não tenho / bloqueado".

#### Scenario: Filtrar armas que o jogador tem
- **WHEN** o jogador abre a categoria de armas e escolhe "tenho"
- **THEN** a lista mostra só as armas da posse dele, cada uma com ícone e nome

#### Scenario: Ícone indisponível
- **WHEN** a instalação do jogo não é encontrada
- **THEN** a página mostra um ícone genérico da categoria e continua funcionando

### Requirement: Ficha da arma
O app SHALL mostrar, para cada arma com histórico, tiros, acertos, precisão, abates, abates de um tiro, recorde de pontos com a espécie, tiro mais longe em órgão vital e a data da compra.

#### Scenario: Arma usada
- **WHEN** o jogador abre a ficha de uma arma com 120 tiros e 90 acertos
- **THEN** a precisão aparece como 75 % junto das demais estatísticas

#### Scenario: Arma sem uso
- **WHEN** a arma foi comprada mas nunca disparada
- **THEN** a ficha mostra os campos vazios sem erro

### Requirement: Ranking pessoal
O app SHALL ordenar as armas do jogador por abates, precisão ou recorde, à escolha dele.

#### Scenario: Ordenar por precisão
- **WHEN** o jogador escolhe ordenar por precisão
- **THEN** as armas aparecem da mais precisa para a menos precisa, ignorando as que nunca dispararam

### Requirement: Loadouts e progresso de coleção
O app SHALL listar os loadouts salvos, marcar o ativo, mostrar os itens que faltam em cada um e mostrar quantos itens de cada categoria o jogador já tem.

#### Scenario: Loadout com item faltando
- **WHEN** um loadout tem um item que o jogador não possui
- **THEN** o item aparece na lista de faltantes daquele loadout

### Requirement: Montador de equipamento por animal
O app SHALL sugerir, para uma reserva e um animal escolhidos, combinações de arma, munição e mira, usando a classe de caça do animal e as classes recomendadas de cada munição que o próprio jogo informa, separando o que o jogador já tem do que vale comprar.

#### Scenario: Animal grande
- **WHEN** o jogador escolhe o alce (classe 8)
- **THEN** só aparecem munições cuja faixa de classes inclui 8, cada uma com o motivo (faixa, precisão, alcance, penetração)

#### Scenario: O que falta
- **WHEN** a combinação pede uma arma ou munição que o jogador não tem
- **THEN** o app mostra quanto falta gastar e o nível que libera o item, e marca como bloqueado se o jogador ainda não chegou nele

#### Scenario: Premium entrega o conjunto
- **WHEN** o usuário Premium aperta "Entregar este conjunto" numa sugestão a comprar
- **THEN** o app pede confirmação e entrega a arma e a munição pelo editor de equipamentos, com backup

#### Scenario: Sem classe de caça
- **WHEN** o jogo não informa a classe do animal
- **THEN** o app avisa que não há sugestão

### Requirement: Kit para a reserva inteira
O app SHALL montar, para uma reserva escolhida, o menor conjunto de armas do arsenal que cobre as classes de caça de todos os animais dela, indicar as iscas e chamados que atraem esses animais, listar extras úteis (telêmetro, primeiros socorros, eliminador de cheiro) e somar o peso do kit. O app SHALL deixar o jogador digitar o limite de peso que o jogo mostra e comparar com o total, sem inventar um limite.

#### Scenario: Uma arma cobre toda a reserva
- **WHEN** uma única arma e munição do jogador atende todas as classes dos animais da reserva
- **THEN** o kit leva só essa arma e diz quais animais ela atende

#### Scenario: Animal sem cobertura
- **WHEN** nenhuma arma do arsenal atende um animal da reserva
- **THEN** o app sugere a arma a comprar que o cobre, com custo e nível que libera, ou avisa que nem a loja o cobre

#### Scenario: Iscas e chamados
- **WHEN** o jogador tem um chamado que atrai espécies da reserva
- **THEN** o kit o lista com as espécies que atrai, alcance e chance, e as iscas a comprar só trazem o que ainda falta

#### Scenario: Limite de peso
- **WHEN** o jogador digita o limite de peso
- **THEN** o app mostra quanto sobra ou quanto passa, guarda o valor só neste computador e, sem limite digitado, só mostra o peso total
