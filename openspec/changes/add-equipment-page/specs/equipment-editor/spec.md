## ADDED Requirements

### Requirement: Escritas de equipamento são Premium
O app SHALL oferecer as escritas de equipamento (liberar ou entregar item, quantidade, veículo, loadout, dinheiro) só com licença ativa, e o serviço do app SHALL recusá-las sem licença, mesmo que a tela seja contornada.

#### Scenario: Usuário gratuito
- **WHEN** o usuário gratuito tenta entregar um item
- **THEN** o app mostra o cadeado que leva ao Premium e nada é gravado

### Requirement: Liberar ou entregar item
O app SHALL permitir marcar um item do catálogo como liberado e possuído, ocupando uma vaga livre dos arrays do perfil sem mudar o tamanho do arquivo.

#### Scenario: Entregar uma mira
- **WHEN** o usuário Premium entrega uma mira que não tem
- **THEN** a mira aparece como possuída e o resto do arquivo fica idêntico

#### Scenario: Sem vaga livre
- **WHEN** os arrays do perfil estão cheios
- **THEN** o app recusa o pedido e explica o motivo

### Requirement: Quantidades e veículos
O app SHALL permitir ajustar a quantidade de munição e consumíveis dentro de um limite do jogo e reabastecer e consertar veículos.

#### Scenario: Reabastecer
- **WHEN** o usuário Premium reabastece um veículo
- **THEN** o combustível e a vida dele passam ao máximo conhecido

### Requirement: Montar loadouts
O app SHALL permitir montar e salvar um loadout só com itens que o jogador possui, recusando combinações incompatíveis.

#### Scenario: Arma sem munição compatível
- **WHEN** o loadout tem uma arma e uma munição que não servem nela
- **THEN** o app avisa e não grava

### Requirement: Dinheiro
O app SHALL permitir ajustar o dinheiro do jogador dentro de um limite seguro e avisar do risco antes de gravar.

#### Scenario: Valor acima do limite
- **WHEN** o usuário pede um valor acima do limite
- **THEN** o app recusa e mostra o limite

### Requirement: Segurança da gravação
Toda escrita de equipamento SHALL fazer backup antes, SHALL recusar com o jogo aberto e SHALL manter o tamanho do arquivo.

#### Scenario: Jogo aberto
- **WHEN** o jogo está em execução
- **THEN** o app recusa a gravação e pede para fechá-lo
