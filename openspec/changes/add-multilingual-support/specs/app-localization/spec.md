## ADDED Requirements

### Requirement: Idiomas da interface
O app SHALL oferecer a interface em português do Brasil, inglês e espanhol, SHALL usar o idioma escolhido pelo jogador, na falta dele o idioma do sistema e, se o sistema estiver em outro idioma, o inglês.

#### Scenario: Primeira abertura
- **WHEN** o jogador abre o app pela primeira vez com o sistema em espanhol
- **THEN** a interface aparece em espanhol

#### Scenario: Escolher outro idioma
- **WHEN** o jogador escolhe English em Ajustes
- **THEN** toda a interface passa a inglês na hora, a escolha fica guardada e vale na próxima abertura

#### Scenario: Sistema em um idioma que o app não tem
- **WHEN** o sistema está em francês e o jogador nunca escolheu um idioma
- **THEN** a interface aparece em inglês

### Requirement: Conteúdo do jogo no idioma escolhido
O app SHALL mostrar os nomes das espécies, os textos das missões e os nomes e descrições das conquistas no idioma escolhido, usando o que o próprio jogo e a Steam já têm nesse idioma, e SHALL atualizar essas telas assim que o idioma mudar.

#### Scenario: Missões em inglês
- **WHEN** o jogador troca para inglês com a aba Missões aberta
- **THEN** os textos das missões aparecem em inglês, como no jogo

#### Scenario: Espécie sem nome traduzido
- **WHEN** o jogo traz uma espécie que a tabela do app não conhece
- **THEN** o app mostra o identificador da espécie de forma legível, em qualquer idioma

### Requirement: Formatos do idioma
O app SHALL formatar números, datas e valores em dinheiro conforme o idioma escolhido.

#### Scenario: Número grande
- **WHEN** a interface está em inglês
- **THEN** 1234567 aparece como 1,234,567 e, em português, como 1.234.567

### Requirement: Mensagens de erro no idioma
O app SHALL mostrar as mensagens de erro no idioma escolhido, e SHALL usar o texto técnico original apenas para erros que ainda não têm tradução.

#### Scenario: Jogo aberto
- **WHEN** o jogador tenta gravar com o jogo aberto, com a interface em espanhol
- **THEN** a mensagem pede, em espanhol, para fechar o jogo
