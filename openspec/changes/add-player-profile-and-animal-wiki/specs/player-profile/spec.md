## ADDED Requirements

### Requirement: Mostrar o perfil do jogador
O app SHALL mostrar o nível, a experiência e o dinheiro do jogador e as marcas de caça somadas de todas as armas: animais abatidos, abates com um tiro, tiros disparados, precisão, maior pontuação, tiro mais longe, troféus no salão e reservas jogadas.

#### Scenario: Marcas de um save real
- **WHEN** o usuário abre a aba Perfil
- **THEN** o app mostra os números lidos de `thp_player_profile_adf` e `trophy_lodges_adf` e a precisão como o percentual de tiros que acertaram

#### Scenario: Nenhum tiro disparado
- **WHEN** o jogador ainda não atirou
- **THEN** a precisão aparece como 0%

### Requirement: Mostrar as conquistas da Steam
O app SHALL mostrar todas as conquistas do jogo com nome e descrição em português, marcar as obtidas pela conta do jogador e permitir filtrar (todas, obtidas, faltam) e buscar por texto.

#### Scenario: Conquistas obtidas e faltantes
- **WHEN** a Steam deste computador tem o esquema e o cache da conta
- **THEN** o app mostra "X de N conquistas" com o anel de progresso e as obtidas primeiro

#### Scenario: Conquista secreta
- **WHEN** uma conquista é secreta e ainda não foi obtida
- **THEN** o app não mostra o nome nem a descrição dela

#### Scenario: Steam indisponível
- **WHEN** os arquivos de estatísticas da Steam não são encontrados
- **THEN** o app mostra o resto do perfil e explica que precisa da Steam aberta e com a conta do jogador

#### Scenario: Busca sem resultado
- **WHEN** o texto digitado não bate com nenhuma conquista
- **THEN** o app avisa que nenhuma conquista tem esse filtro
