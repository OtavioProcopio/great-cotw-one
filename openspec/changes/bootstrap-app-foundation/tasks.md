## 1. Projeto

- [x] 1.1 Criar o app Tauri 2 (Rust + TypeScript/Vite) com CI de build para Windows
- [x] 1.2 Criar o crate `cotw-formats` e portar `hashlittle` com testes contra valores conhecidos (`missions/achievements.hpmissionsc` → `b0e88e20`)
- [x] 1.3 Configurar `.gitignore` para `fixtures/`, backups e qualquer arquivo extraído do jogo

## 2. Formatos (somente leitura)

- [x] 2.1 Envelope do save (cabeçalho 32 bytes, zlib, prefixo 5 bytes) com teste de ida e volta
- [x] 2.2 Leitor ADF genérico (typedefs, instâncias, structs, arrays, arrays inline, enums, string hash)
- [x] 2.3 Índice `.tab`/`.arc` por hash de caminho, AAF, SARC, RTPC e XLS book
- [x] 2.4 Testes comparando a saída com `research/reference` nos saves de fixture

## 3. Descoberta

- [x] 3.1 Encontrar pastas de save (Documentos e OneDrive), listar contas e reservas
- [x] 3.2 Encontrar a instalação via `libraryfolders.vdf` da Steam, com seleção manual
- [x] 3.3 Tela de diagnóstico (jogo, save, reservas, arquivos reconhecidos)

## 4. Escrita segura

- [x] 4.1 Detecção do processo do jogo
- [x] 4.2 Backup completo com data em `%APPDATA%` e histórico em SQLite
- [x] 4.3 Escritor ADF com reconstrução de offsets, contagens e cadeia de relocação + testes (cadeia sem quebras)
- [x] 4.4 `SaveTransaction` com prévia, validação pós-gravação e rollback automático
- [x] 4.5 Tela de histórico com restauração de backup

## 5. Verificação

- [x] 5.1 Testes de ida e volta byte a byte em todos os arquivos de save conhecidos
- [x] 5.2 Teste manual: gravar sem mudanças, abrir o jogo, confirmar que o save carrega
