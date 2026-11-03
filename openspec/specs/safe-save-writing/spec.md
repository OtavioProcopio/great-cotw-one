# safe-save-writing Specification

## Purpose
Garantir que qualquer gravação no save seja segura: só com o jogo fechado, com backup completo antes, validação depois, restauração automática em caso de falha e histórico que permite voltar atrás.

## Requirements

### Requirement: Jogo fechado antes de gravar
O app MUST NOT gravar nenhum arquivo do save enquanto o processo do jogo estiver em execução.

#### Scenario: Usuário tenta aplicar com o jogo aberto
- **WHEN** o usuário confirma uma alteração e o processo `theHunterCotW_F` está rodando
- **THEN** o app bloqueia a gravação, explica o motivo e mantém a alteração pendente até o jogo ser fechado

### Requirement: Backup automático
O app SHALL copiar a pasta inteira do save para uma pasta de backup com data e hora antes de cada gravação, e MUST NOT apagar backups automaticamente.

#### Scenario: Primeira alteração do dia
- **WHEN** o usuário aplica uma alteração
- **THEN** existe um backup completo do estado anterior antes de qualquer byte do save ser alterado

#### Scenario: Falha ao criar o backup
- **WHEN** o backup não pode ser criado (sem espaço, sem permissão)
- **THEN** o app não grava nada e mostra o erro

### Requirement: Prévia e confirmação
O app SHALL mostrar ao usuário o que vai mudar (arquivo, itens afetados, antes e depois) e SHALL gravar somente após confirmação explícita.

#### Scenario: Usuário cancela
- **WHEN** o usuário cancela na tela de prévia
- **THEN** nenhum arquivo é alterado e nenhum backup novo é criado

### Requirement: Validação depois de gravar
O app SHALL reler cada arquivo gravado e conferir estrutura, contagens e que só os dados pretendidos mudaram; se a conferência falhar, SHALL restaurar o arquivo a partir do backup recém-criado.

#### Scenario: Gravação inconsistente
- **WHEN** a releitura encontra um ponteiro inválido ou uma contagem diferente da esperada
- **THEN** o app restaura o arquivo original e informa que nada foi alterado

### Requirement: Histórico e restauração
O app SHALL manter um histórico das alterações aplicadas (data, função, arquivos, resumo) e SHALL permitir restaurar qualquer backup com o jogo fechado.

#### Scenario: Jogo não abre depois de uma alteração
- **WHEN** o usuário escolhe restaurar o último backup
- **THEN** a pasta do save volta exatamente ao estado do backup e a restauração também fica registrada no histórico
