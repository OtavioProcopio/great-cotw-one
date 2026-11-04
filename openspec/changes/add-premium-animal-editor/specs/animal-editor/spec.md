## ADDED Requirements

### Requirement: Editar os animais de um grupo rastreado
Ao rastrear um grupo no mapa, o app SHALL listar os animais do grupo e, para quem tem Premium, SHALL oferecer em cada animal um botão de editar que abre um painel com sexo, peso, pontuação e marca de Great One; quem não tem Premium SHALL ver o mesmo botão com um cadeado que leva à tela do Premium.

#### Scenario: Premium edita um animal do grupo
- **WHEN** o usuário Premium rastreia o grupo 136 de cervo-vermelho e aperta editar no segundo animal
- **THEN** o painel abre com os valores atuais desse animal, as faixas aceitas e o botão de gravar

#### Scenario: Usuário gratuito
- **WHEN** o usuário gratuito aperta o botão do animal
- **THEN** o app abre a tela do Premium e nada é aberto para edição

### Requirement: Editar valores do animal
O app SHALL permitir editar sexo, peso, pontuação e a marca de Great One (só em espécies que têm Great One na reserva) de qualquer animal que já existe, sem mudar o tamanho do arquivo.

#### Scenario: Mudar o peso
- **WHEN** o usuário muda o peso de um macho para um valor dentro da faixa aceita
- **THEN** a gravação muda só esse valor e o resto do arquivo fica idêntico

#### Scenario: Great One em espécie sem Great One
- **WHEN** a espécie não tem nenhum Great One na reserva
- **THEN** o botão de Great One fica desativado com a explicação e o app recusa o pedido se ele chegar assim

### Requirement: Equilíbrio do jogo
O app SHALL aceitar peso e pontuação apenas dentro do que já existe no jogo para a espécie e o sexo escolhido (do menor ao maior valor visto, com 10% de folga), SHALL mostrar a faixa aceita e o valor típico antes de gravar e MUST recusar, com uma mensagem simples, qualquer valor fora dela; a recusa MUST valer também no serviço do app, e não só na tela.

#### Scenario: Valor fora da faixa
- **WHEN** o usuário digita um peso acima do máximo aceito
- **THEN** o campo fica marcado, o botão de gravar fica desativado e, se o pedido chegar ao app, ele é recusado com `editorOutOfRange` sem alterar o save

#### Scenario: Valor nunca visto no jogo
- **WHEN** o peso ou a pontuação não é um número válido ou é negativo
- **THEN** o app recusa com `editorInvalidValue`

### Requirement: Trocar o sexo do animal
O app SHALL permitir trocar um animal de macho para fêmea e de fêmea para macho; ao trocar, SHALL sugerir o peso e a pontuação típicos do sexo novo e MUST aplicar à combinação final as faixas do sexo novo.

#### Scenario: Macho vira fêmea
- **WHEN** o usuário troca um macho de 560 kg para fêmea
- **THEN** o painel sugere o peso e a pontuação típicos de fêmea da espécie, e o app só grava se os valores couberem na faixa de fêmea

#### Scenario: Trocar sem ajustar os valores
- **WHEN** o usuário troca o sexo e mantém um peso que só cabe no outro sexo
- **THEN** o botão de gravar fica desativado até os valores caberem

### Requirement: Ações em lote
O app SHALL oferecer "1 diamante por grupo" (escolhendo o macho mais pesado; se não houver macho, transformando o primeiro animal) e "desfazer alterações desta espécie", sem mexer em grupos que já têm diamante ou Great One.

#### Scenario: Lote numa espécie já completa
- **WHEN** todos os grupos já têm diamante ou Great One
- **THEN** o app informa que não há o que mudar

### Requirement: Escolher a pelagem de um animal existente
O app SHALL oferecer, para cada animal, a lista de pelagens da espécie e do sexo (a mesma do guia de pelagens, com a chance e a raridade de cada uma) e SHALL gravar a escolha mudando só a semente de pelagem do animal, sem mudar o tamanho do arquivo.

#### Scenario: Escolher uma pelagem rara
- **WHEN** o usuário escolhe "Albino" para um javali macho
- **THEN** a prévia mostra a pelagem de antes e a de depois, e só a semente desse animal muda ao gravar

#### Scenario: Pelagem atual desconhecida
- **WHEN** a semente atual do animal não está na tabela conhecida
- **THEN** o app mostra "pelagem sorteada pelo jogo" como situação de antes, sem inventar um nome

### Requirement: Somente animais que já existem
O app MUST NOT criar nem remover animais, grupos ou populações: só SHALL editar os valores de animais que já estão no save, mantendo o tamanho do arquivo e a quantidade de animais de cada grupo.

#### Scenario: Releitura depois de gravar
- **WHEN** a gravação de uma edição termina
- **THEN** o arquivo relido tem o mesmo tamanho, a mesma quantidade de animais em cada grupo e só os valores editados diferentes

### Requirement: Ligação com o mapa
O app SHALL abrir o editor no grupo escolhido a partir do rastreador do mapa e SHALL abrir o mapa no grupo a partir do editor; o mapa SHALL refletir as edições ainda não gravadas.

#### Scenario: Editar a partir do mapa
- **WHEN** o usuário clica "Editar este grupo" no grupo 47 de coiote
- **THEN** o editor abre filtrado nesse grupo

### Requirement: Escrita segura obrigatória
Toda gravação do editor SHALL passar pela escrita segura (jogo fechado, backup, prévia com lista de animais alterados, validação, histórico).

#### Scenario: Prévia
- **WHEN** o usuário clica em gravar com 12 animais alterados
- **THEN** a prévia lista os 12 animais com antes e depois
