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

### Requirement: Recomendação de arma e munição
O app SHALL sugerir, para uma espécie escolhida, armas e munições adequadas com o motivo de cada sugestão, e SHALL marcar quais o jogador já possui.

#### Scenario: Sugestão para um cervo
- **WHEN** o jogador escolhe uma espécie de porte médio
- **THEN** o app lista armas e munições adequadas à classe de animal, com o motivo e a indicação de posse
