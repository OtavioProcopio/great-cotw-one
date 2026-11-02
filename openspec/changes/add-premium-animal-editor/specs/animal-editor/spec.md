## ADDED Requirements

### Requirement: Editar valores do animal
O app SHALL permitir editar sexo, peso, pontuação, pelagem (entre as sementes conhecidas da espécie e sexo) e a marca de Great One (só em espécies que têm Great One) de qualquer animal, sem mudar o tamanho do arquivo.

#### Scenario: Deixar um animal diamante
- **WHEN** o usuário aplica "Diamante" num animal
- **THEN** o animal vira macho com peso e pontuação dentro da faixa de diamante da espécie e é marcado como diamante

#### Scenario: Great One em espécie sem faixa conhecida
- **WHEN** a espécie tem Great One no jogo mas não há faixa publicada e o save do usuário tem um Great One dela
- **THEN** o app usa peso, pontuação e pelagem desse Great One real como referência

#### Scenario: Valor fora da faixa
- **WHEN** o usuário digita peso ou pontuação acima do máximo conhecido da espécie
- **THEN** o campo fica marcado em alerta, mas a edição é permitida

### Requirement: Ações em lote
O app SHALL oferecer "1 diamante por grupo" (escolhendo o macho mais pesado; se não houver macho, transformando o primeiro animal) e "desfazer alterações desta espécie", sem mexer em grupos que já têm diamante ou Great One.

#### Scenario: Lote numa espécie já completa
- **WHEN** todos os grupos já têm diamante ou Great One
- **THEN** o app informa que não há o que mudar

### Requirement: Criar animal com segurança
O app SHALL permitir acrescentar um animal a um grupo copiando outro do mesmo grupo, MUST NOT deixar o grupo passar do maior tamanho que o jogo já gerou para a espécie, e SHALL reconstruir offsets, contagens e a cadeia de relocação, marcando a função como experimental.

#### Scenario: Grupo no limite
- **WHEN** o grupo já está no tamanho máximo da espécie
- **THEN** o botão de criar fica desativado com a explicação

#### Scenario: Verificação da estrutura
- **WHEN** o arquivo com animais novos é gerado
- **THEN** a releitura mostra a contagem nova, nenhum outro animal alterado e nenhuma quebra na cadeia de ponteiros

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
