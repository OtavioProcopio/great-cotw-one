## ADDED Requirements

### Requirement: Calcular as zonas a liberar
O app SHALL calcular, por reserva, as zonas de necessidade usadas pelas rotas dos grupos atuais que ainda não existem no save, e SHALL mostrar a lista (espécie, tipo, posição, horário calculado) antes de gravar.

#### Scenario: Nada a liberar
- **WHEN** todas as zonas usadas pelos grupos já estão no save
- **THEN** o app informa que não há zonas novas e não oferece gravação

### Requirement: Posição, altura e horário corretos
Cada zona liberada SHALL ter posição no centro da área do mapa de spawn do jogo, altura no nível do terreno (nunca um valor fixo como 1,0), identificador único e horário escolhido entre os horários reais da espécie que cabem entre as paradas conhecidas do grupo.

#### Scenario: Zona entre duas paradas conhecidas
- **WHEN** a zona nova fica entre uma parada que termina às 6h e outra que começa às 10h
- **THEN** o horário escolhido é um horário real da espécie para aquele tipo que cabe entre 6h e 10h

#### Scenario: Zona sem horário conhecido da espécie
- **WHEN** a espécie não tem nenhuma zona daquele tipo descoberta pelo jogo
- **THEN** a zona não é liberada e aparece na lista como "sem horário conhecido"

### Requirement: Gravação segura e verificada
A liberação SHALL usar a escrita segura (jogo fechado, backup, prévia, validação) e SHALL preservar todas as zonas já existentes, seus horários e grupos descobertos.

#### Scenario: Releitura depois de gravar
- **WHEN** a gravação termina
- **THEN** o arquivo relido tem as zonas antigas intactas mais as novas, sem zona com altura 1,0 e sem quebra na cadeia de ponteiros

### Requirement: Escopo por reserva
O app SHALL permitir liberar uma reserva por vez ou todas, e SHALL mostrar quantas zonas cada reserva vai ganhar.

#### Scenario: Liberar só Layton Lake
- **WHEN** o usuário escolhe apenas Layton Lake
- **THEN** só as zonas da reserva 1 são acrescentadas
