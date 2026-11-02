## ADDED Requirements

### Requirement: Mostrar zonas de necessidade
O app SHALL mostrar no mapa da reserva as zonas de necessidade do save e, opcionalmente, as zonas não descobertas que algum grupo atual usa, diferenciando visualmente as duas.

#### Scenario: Filtrar por espécie e tipo
- **WHEN** o usuário escolhe "Coiote" e o tipo "Beber"
- **THEN** o mapa mostra apenas as zonas de beber de coiote

#### Scenario: Filtrar por horário
- **WHEN** o usuário ativa "só ativas às 06:00"
- **THEN** o mapa mostra apenas as zonas cujo horário inclui 06:00, considerando horários que passam da meia-noite

### Requirement: Coordenadas iguais às do jogo
O app SHALL mostrar as coordenadas X/Z do cursor no mesmo sistema do mapa do jogo, com norte para cima.

#### Scenario: Conferir uma zona
- **WHEN** o usuário passa o cursor sobre uma zona
- **THEN** o app mostra espécie, tipo, horário e as coordenadas, que batem com a posição da zona no mapa do jogo

### Requirement: Rastrear um grupo
O app SHALL permitir escolher espécie e grupo no mesmo painel do filtro de espécies e SHALL mostrar a rota diária do grupo em ordem, com o horário de cada parada e destaque da parada correspondente ao horário escolhido.

#### Scenario: Horário vindo do save
- **WHEN** a parada é uma zona já descoberta
- **THEN** o horário mostrado é o que o jogo gravou

#### Scenario: Horário estimado
- **WHEN** a parada é uma zona não descoberta
- **THEN** o horário é estimado pela tabela de horários da espécie na reserva e aparece marcado como estimado

### Requirement: Mostrar os animais do grupo
O app SHALL listar os animais de cada grupo com sexo, peso, pontuação e indicação de diamante e Great One, e SHALL ordenar a lista de grupos destacando os que têm Great One ou diamante.

#### Scenario: Grupo com Great One
- **WHEN** um grupo tem um animal com a marca de Great One
- **THEN** o grupo aparece no topo da lista com a marca ⭐
