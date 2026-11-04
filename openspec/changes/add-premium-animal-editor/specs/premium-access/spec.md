## ADDED Requirements

### Requirement: O que é premium
Toda função que grava no jogo SHALL exigir licença premium ativa: liberar as zonas de necessidade e editar animais que já existem no save. Ver o mapa (com as zonas descobertas e as que ainda faltam), os animais, as pelagens, o perfil, as missões, a carreira e a build MUST continuar gratuitos; as funções de backup e restauração MUST continuar gratuitas.

#### Scenario: Usuário gratuito no editor
- **WHEN** um usuário sem licença abre o editor de animais
- **THEN** ele vê os animais e pode simular edições, mas o botão de gravar mostra como ativar o premium

#### Scenario: Usuário gratuito em Liberar mapa
- **WHEN** um usuário sem licença abre a tela de liberar o mapa
- **THEN** ele vê quantas zonas cada reserva, animal e necessidade ganharia, mas o botão de gravar mostra que é do premium e nada é gravado

#### Scenario: O app também recusa
- **WHEN** um pedido de gravação chega ao app sem licença ativa
- **THEN** o app recusa com o código `premiumRequired` e não altera nenhum arquivo, mesmo que a interface seja contornada

### Requirement: Planos e vencimento
O Premium SHALL ser vendido em dois planos: mensal (R$ 15,00 por mês, válido por 31 dias) e anual (R$ 75,00 por ano, válido por 366 dias). O app SHALL avisar o usuário nos 30 dias antes do vencimento e, ao vencer, SHALL voltar ao modo gratuito sem apagar nada e sem perder o acesso aos backups e à restauração.

#### Scenario: Licença válida
- **WHEN** a licença está dentro da validade
- **THEN** as funções que gravam no jogo funcionam e a tela do Premium mostra o plano e a data de vencimento

#### Scenario: Perto de vencer
- **WHEN** faltam 30 dias ou menos para o vencimento
- **THEN** o app mostra um aviso na tela inicial e na do Premium com como renovar, sem bloquear nada

#### Scenario: Licença vencida
- **WHEN** a validade termina
- **THEN** as funções Premium voltam ao modo leitura, o resto do app continua igual e o usuário consegue renovar colando a nova chave

### Requirement: Tela de vendas do Premium
Ao abrir o Premium, o app SHALL mostrar tudo o que o usuário ganha em relação a quem não é Premium: as vantagens, uma comparação função por função entre o gratuito e o Premium, os dois planos com preço, a economia do plano anual sobre doze meses do mensal e o campo para colar a chave. A comparação MUST marcar como "em breve" qualquer função que ainda não funcione, em vez de prometê-la.

#### Scenario: Usuário gratuito abre o Premium
- **WHEN** um usuário sem licença abre a tela do Premium
- **THEN** ele vê as vantagens, a tabela com o que cada versão faz, o plano mensal a R$ 15,00 e o anual a R$ 75,00 com o desconto de 58% (economia de R$ 105,00 ao ano, R$ 6,25 por mês)

#### Scenario: Chegar pelo bloqueio
- **WHEN** um usuário gratuito tenta liberar zonas ou editar animais
- **THEN** o app leva à tela do Premium em vez de só mostrar um aviso

#### Scenario: Loja ainda fechada
- **WHEN** o usuário aperta o botão de um plano e a loja ainda não abriu
- **THEN** o app explica isso em palavras simples e lembra que quem já tem a chave pode colá-la na mesma tela

#### Scenario: Usuário já Premium
- **WHEN** um usuário com licença ativa abre a tela
- **THEN** o plano dele aparece marcado e a data de vencimento fica visível

### Requirement: Ativação e uso offline
O app SHALL aceitar a chave de licença colada pelo usuário e SHALL verificá-la offline pela assinatura digital (chave pública embutida no app), sem depender de servidor para funcionar depois de comprada.

#### Scenario: Sem internet
- **WHEN** o usuário com licença válida abre o app sem internet
- **THEN** o editor e a liberação das zonas gravam normalmente

#### Scenario: Chave inválida
- **WHEN** o usuário cola uma chave adulterada, de outro produto ou com assinatura inválida
- **THEN** o app recusa com uma mensagem simples e continua no modo gratuito

#### Scenario: Chave guardada
- **WHEN** o usuário ativa a licença uma vez
- **THEN** o app guarda a chave no computador e não pede de novo ao reabrir

### Requirement: Privacidade
O app MUST NOT enviar dados do save, do jogo ou caminhos de arquivos para ativar ou usar a licença; como a verificação é offline, a ativação não envia nada.

#### Scenario: Ativação
- **WHEN** o usuário ativa a licença
- **THEN** nenhuma requisição de rede é feita pelo app
