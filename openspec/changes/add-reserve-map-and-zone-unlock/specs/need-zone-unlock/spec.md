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

### Requirement: Liberar por animal ou por necessidade
O app SHALL permitir escolher o que liberar: tudo, só algumas espécies ou só algumas necessidades (comer, beber, descansar), e SHALL mostrar, antes de gravar, quantas zonas cada espécie e cada necessidade ganha nas reservas escolhidas.

#### Scenario: Só beber
- **WHEN** o usuário escolhe "Por necessidade" e marca só "Beber"
- **THEN** o app mostra quantas zonas de beber serão liberadas e só essas são acrescentadas

#### Scenario: Só alguns animais
- **WHEN** o usuário escolhe "Por animal" e marca só o alce
- **THEN** só as zonas usadas pelos grupos de alce são acrescentadas

#### Scenario: Nada escolhido
- **WHEN** o modo é por animal ou por necessidade e nada está marcado
- **THEN** o app pede para marcar uma opção e não oferece gravação

### Requirement: Liberar é Premium
O app SHALL mostrar a todos os usuários as zonas que ainda faltam descobrir e quantas seriam liberadas, e SHALL gravar a liberação somente com a licença Premium ativa.

#### Scenario: Usuário gratuito
- **WHEN** um usuário sem licença escolhe o que liberar
- **THEN** o app mostra a prévia e os números, mas o botão leva ao aviso do Premium e nada é gravado

#### Scenario: Pedido sem licença
- **WHEN** um pedido de liberação chega ao app sem licença ativa
- **THEN** o app recusa com `premiumRequired` e o save fica intacto
