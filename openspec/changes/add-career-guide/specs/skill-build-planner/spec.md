## ADDED Requirements

### Requirement: Árvores fiéis ao jogo
O app SHALL mostrar as árvores Perseguidor e Emboscador e as vantagens de Rifles, Pistolas, Espingardas e Arcos com a mesma posição, número de níveis e ligações do jogo, lidos da instalação do usuário.

#### Scenario: Vantagem em outra árvore
- **WHEN** o usuário abre a árvore de Rifles
- **THEN** Windage aparece nela, ligada a Steady Hands, como no jogo

### Requirement: Regras de desbloqueio
O app SHALL permitir colocar um ponto numa habilidade só se os pontos nas linhas de cima da mesma árvore atingem o custo da linha, e numa vantagem só se houver ao menos um ponto numa vantagem ligada; SHALL impedir tirar um ponto que deixaria a build inválida; e SHALL respeitar o total de pontos do nível do jogador.

#### Scenario: Linha bloqueada
- **WHEN** a árvore tem 4 pontos nas linhas de cima e o usuário tenta uma habilidade da linha que pede 5
- **THEN** o app não deixa e mostra quantos pontos faltam

#### Scenario: Tirar pré-requisito
- **WHEN** o usuário tenta zerar uma vantagem da qual outra vantagem comprada depende sozinha
- **THEN** o app não deixa e explica qual vantagem depende dela

### Requirement: Carregar e aplicar a build
O app SHALL carregar a build atual do save e SHALL, se o usuário pedir, gravar a nova build (habilidades, níveis, pontos gastos por árvore, pontos livres e habilidade ativa) pela escrita segura, sem mudar o tamanho do arquivo.

#### Scenario: Habilidade ativa removida
- **WHEN** a nova build não tem mais a habilidade que estava equipada como ativa
- **THEN** o app limpa a habilidade ativa ao gravar

#### Scenario: Alternativa sem editar o save
- **WHEN** o usuário não quer editar o save
- **THEN** o app mostra a ordem de compra para reproduzir a build no jogo usando o reset pago da árvore
