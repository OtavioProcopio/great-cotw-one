## ADDED Requirements

### Requirement: O que é premium
Gravar alterações de animais (editar e criar) SHALL exigir licença premium ativa; ver os animais, o mapa, liberar zonas, a carreira e a build MUST continuar gratuitos.

#### Scenario: Usuário gratuito no editor
- **WHEN** um usuário sem licença abre o editor de animais
- **THEN** ele vê os animais e pode simular edições, mas o botão de gravar mostra como ativar o premium

### Requirement: Ativação e uso offline
O app SHALL ativar a licença uma vez com conexão e SHALL continuar funcionando offline depois, revalidando periodicamente quando houver internet.

#### Scenario: Sem internet depois de ativar
- **WHEN** o usuário com licença ativa abre o app sem internet
- **THEN** o editor grava normalmente

#### Scenario: Licença inválida
- **WHEN** a revalidação recusa a licença
- **THEN** o editor volta ao modo leitura e o resto do app continua funcionando

### Requirement: Privacidade
A ativação MUST NOT enviar dados do save, do jogo ou caminhos de arquivos; só a chave da licença e um identificador da máquina.

#### Scenario: Ativação
- **WHEN** o usuário ativa a licença
- **THEN** a requisição contém apenas a chave e o identificador da máquina
