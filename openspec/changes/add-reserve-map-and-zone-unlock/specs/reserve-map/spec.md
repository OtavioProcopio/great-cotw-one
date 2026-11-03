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

### Requirement: Reconhecer as reservas do jogador
O app SHALL montar a lista de reservas a partir do que existe no computador do jogador (reservas com dados no save e mapas no jogo instalado), com a foto de capa do próprio jogo, sem depender de uma lista fixa de reservas.

#### Scenario: Reserva de qualquer expansão
- **WHEN** o save e o jogo têm dados de uma reserva
- **THEN** ela aparece na faixa de reservas do mapa e na tela inicial, com a capa e o nome

#### Scenario: Escolher pela tela inicial
- **WHEN** o usuário toca numa reserva da tela inicial
- **THEN** o app abre o mapa dessa reserva

### Requirement: Imagem de fundo do mapa
O app SHALL desenhar as zonas sobre a imagem do mapa do próprio jogo instalado quando ela existir, e SHALL avisar quando o alinhamento da imagem tiver sido calculado pelas zonas em vez de vir de um retângulo conhecido.

#### Scenario: Reserva com retângulo conhecido
- **WHEN** a reserva tem imagem e retângulo conhecidos
- **THEN** o mapa mostra a imagem alinhada com as coordenadas do jogo, sem aviso

#### Scenario: Reserva nova com alinhamento calculado
- **WHEN** a reserva tem imagem mas não tem retângulo conhecido
- **THEN** o app alinha o centro da parte iluminada da imagem com o centro das zonas e avisa que a imagem pode estar deslocada algumas centenas de metros

#### Scenario: Reserva sem imagem
- **WHEN** o jogo não tem a imagem do mapa da reserva
- **THEN** as zonas aparecem sobre fundo liso com a grade de 1 km e um aviso
