## ADDED Requirements

### Requirement: Listar as reservas do jogo
O app SHALL listar todas as reservas que o jogo instalado tem mapa de spawn, jogadas ou não, com a foto de capa do próprio jogo e a quantidade de espécies, e SHALL marcar as reservas onde o save tem animais.

#### Scenario: Reserva nunca jogada
- **WHEN** o save não tem animais numa reserva que o jogo traz
- **THEN** a reserva aparece na lista sem a marca de jogada e suas espécies continuam consultáveis

#### Scenario: Reserva de expansão nova
- **WHEN** o jogo instalado traz uma reserva que o app nunca viu
- **THEN** ela aparece na lista com as espécies lidas dos mapas de spawn, sem mudança no app

### Requirement: Ficha da espécie
O app SHALL mostrar, para uma espécie numa reserva, as reservas onde ela vive (com atalho para cada uma), os animais vivos do save (quantidade, grupos, machos, fêmeas, maior peso, maior pontuação e Great Ones) e os cinco melhores abates da conta com pontuação, medalha e data.

#### Scenario: Espécie com animais no save
- **WHEN** o usuário abre "Alce" em Medved-Taiga
- **THEN** o app mostra os animais vivos dessa espécie nessa reserva e os melhores abates de alce da conta

#### Scenario: Espécie sem dados
- **WHEN** a espécie não tem animais no save nem abates
- **THEN** o app diz que não há animais no progresso e que a espécie ainda não foi abatida

### Requirement: Buscar espécies
O app SHALL filtrar as espécies da reserva escolhida pelo nome, sem diferenciar maiúsculas de minúsculas.

#### Scenario: Busca parcial
- **WHEN** o usuário digita "alc"
- **THEN** a lista mostra só as espécies cujo nome contém "alc"
