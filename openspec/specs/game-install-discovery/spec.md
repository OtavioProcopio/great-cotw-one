# game-install-discovery Specification

## Purpose
Encontrar sozinho, no computador do jogador, a pasta dos saves e a instalação do jogo (com escolha manual quando a busca falhar), para que o app funcione sem o usuário saber onde nada fica.

## Requirements

### Requirement: Localizar a pasta de saves
O app SHALL encontrar automaticamente a pasta `Avalanche Studios\COTW\Saves\<steamid>` do usuário, inclusive quando Documentos está redirecionado para o OneDrive, e SHALL permitir escolher a pasta manualmente.

#### Scenario: Save no caminho padrão
- **WHEN** o usuário abre o app pela primeira vez e existe uma única pasta de conta em Documentos
- **THEN** o app seleciona essa pasta e mostra o caminho completo

#### Scenario: Mais de uma conta Steam
- **WHEN** existem duas ou mais pastas de conta em `Saves`
- **THEN** o app lista as contas com a data da última modificação e pede para o usuário escolher

#### Scenario: Save não encontrado
- **WHEN** nenhuma pasta de save é encontrada
- **THEN** o app explica onde o jogo costuma guardar o save e oferece o seletor de pasta

### Requirement: Localizar a instalação do jogo
O app SHALL encontrar a pasta `theHunterCotW\archives_win64` pelas bibliotecas da Steam e SHALL funcionar em modo limitado (sem nomes, mapas e regras vindos do jogo) quando a instalação não for encontrada.

#### Scenario: Jogo em outra biblioteca Steam
- **WHEN** o jogo está instalado numa biblioteca Steam fora do disco C:
- **THEN** o app encontra a instalação lendo as bibliotecas configuradas da Steam

#### Scenario: Jogo não encontrado
- **WHEN** o app não acha a instalação
- **THEN** o app avisa quais telas ficam limitadas e permite indicar a pasta manualmente

### Requirement: Identificar reservas e arquivos do save
O app SHALL listar as reservas presentes no save (pelos arquivos `animal_population_N`) com o nome de cada reserva, e SHALL marcar arquivos que ele não reconhece sem falhar.

#### Scenario: Reserva conhecida
- **WHEN** existe `animal_population_2`
- **THEN** o app mostra "Medved-Taiga" na lista de reservas

#### Scenario: Reserva nova de um DLC futuro
- **WHEN** existe um `animal_population_N` com N desconhecido
- **THEN** o app mostra "Reserva N" e continua funcionando para as demais
