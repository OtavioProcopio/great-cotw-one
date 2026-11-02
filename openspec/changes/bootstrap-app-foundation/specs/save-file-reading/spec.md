## ADDED Requirements

### Requirement: Leitura sem efeitos colaterais
O app SHALL ler qualquer arquivo do save ou do jogo sem alterar o arquivo, sua data de modificação ou seus atributos.

#### Scenario: Abrir o app com o jogo aberto
- **WHEN** o jogo está em execução e o usuário abre uma tela de consulta
- **THEN** o app lê os arquivos normalmente e nenhum arquivo do save tem a data de modificação alterada

### Requirement: Decodificar os formatos do save
O app SHALL decodificar o envelope do save (cabeçalho de 32 bytes, zlib, prefixo de 5 bytes) e o ADF interno, expondo os dados com os nomes de campos do próprio arquivo.

#### Scenario: Ida e volta sem mudanças
- **WHEN** um arquivo de save é lido e regravado sem nenhuma alteração
- **THEN** o conteúdo descomprimido gerado é idêntico byte a byte ao original

#### Scenario: Arquivo corrompido ou de versão diferente
- **WHEN** um arquivo não tem o cabeçalho ou a assinatura ADF esperados
- **THEN** o app mostra o arquivo como "não reconhecido" e nenhuma tela que depende dele permite edição

### Requirement: Ler dados do jogo instalado
O app SHALL ler, a partir da instalação do usuário, os dados necessários às funções (mapas de spawn, definições de missões e conquistas, árvores de habilidades, variações de pelagem), localizando cada arquivo pelo hash do caminho nos índices `.tab`.

#### Scenario: Atualização do jogo muda um arquivo
- **WHEN** um arquivo esperado não é encontrado ou não tem o formato esperado
- **THEN** a função que depende dele fica indisponível com uma mensagem clara, e as demais continuam funcionando

### Requirement: Não distribuir dados do jogo
O app MUST NOT embutir, exportar ou enviar arquivos extraídos do jogo; dados do jogo existem apenas em memória ou em cache local do próprio usuário.

#### Scenario: Cache local
- **WHEN** o app guarda um cache de dados lidos do jogo para abrir mais rápido
- **THEN** o cache fica na pasta de dados do app no computador do usuário e é invalidado quando os arquivos do jogo mudam
