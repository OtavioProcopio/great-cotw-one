## ADDED Requirements

### Requirement: Missões por cadeia
O app SHALL listar as missões de cada reserva agrupadas pela cadeia (com o nome de quem dá a missão), na ordem do jogo, com estado (feita, em andamento, bloqueada) e cada objetivo com marcação de feito.

#### Scenario: Cadeia em andamento
- **WHEN** o usuário abre a cadeia da Dra. Alena Khasavovna em Medved-Taiga
- **THEN** o app mostra todas as missões da cadeia em ordem, a etapa atual e os objetivos que faltam

### Requirement: Conquistas e progresso
O app SHALL mostrar todas as conquistas da Steam com status de desbloqueio e, para as que faltam, o progresso lido do save: contagem atual e meta, objetivos cumpridos de cada conquista com vários objetivos, e missões feitas nas conquistas de cadeia.

#### Scenario: Conquista de abates
- **WHEN** a conquista pede 50 coiotes e o save tem 36
- **THEN** o app mostra 36/50 e "faltam 14"

#### Scenario: Status da Steam indisponível
- **WHEN** o cache de estatísticas da Steam não é encontrado
- **THEN** o progresso do save continua visível e o status de desbloqueio aparece como desconhecido

### Requirement: O que falta em cada conquista
Quando o save ou a definição do jogo permitir, o app SHALL dizer qual item específico falta (espécie a fotografar, sub-região sem abate, cor de pelagem) e, quando não permitir, SHALL dizer que o jogo só guarda a contagem.

#### Scenario: Fotografar animais
- **WHEN** a conquista de fotos está em 10/14
- **THEN** o app lista as espécies já fotografadas e as que ainda contam

### Requirement: Vitórias rápidas
O app SHALL destacar as conquistas mais próximas de terminar (poucas missões ou poucos itens restantes).

#### Scenario: Cadeia a uma missão do fim
- **WHEN** falta uma missão para concluir uma cadeia que dá conquista
- **THEN** ela aparece na lista de vitórias rápidas com atalho para a cadeia
