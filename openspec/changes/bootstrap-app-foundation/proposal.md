## Why

Hoje o conhecimento sobre o save do theHunter: Call of the Wild está espalhado em scripts e páginas feitas para um único jogador. Para oferecer uma ferramenta para qualquer pessoa no Windows, precisamos de uma base que encontre o save sozinha, leia os arquivos sem risco e só escreva com backup, com o jogo fechado e com verificação. Todas as outras funções (mapa, missões, build, editor premium) dependem dessa base.

## What Changes

- Novo app desktop para Windows com instalador.
- Descoberta automática da instalação do jogo (Steam) e da pasta de saves, com escolha manual quando houver mais de uma conta ou caminho fora do padrão (ex.: OneDrive).
- Leitura somente-leitura dos formatos do save (wrapper SAVE/COMP + zlib + ADF) e dos arquivos do jogo (`.tab`/`.arc`, AAF, SARC, RTPC, XLS).
- Pipeline único de escrita segura: verificar jogo fechado → backup completo com data → gravar → reler e validar → registrar no histórico, com restauração de qualquer backup pela interface.
- Diagnóstico: tela que mostra versão do jogo detectada, reservas presentes no save e quais arquivos foram reconhecidos.

## Capabilities

### New Capabilities
- `game-install-discovery`: localizar o jogo, as contas e as pastas de save, e identificar as reservas presentes.
- `save-file-reading`: ler e decodificar com segurança os arquivos do save e do jogo, sem alterar nada.
- `safe-save-writing`: regras obrigatórias para qualquer gravação (jogo fechado, backup, validação, histórico, restauração).

### Modified Capabilities
(nenhuma)

## Impact

- Repositório novo: app Tauri 2 (Rust + TypeScript), testes de ida e volta com saves de exemplo do próprio desenvolvedor (nunca versionados).
- Nenhum arquivo do jogo é distribuído; tudo é lido da instalação do usuário.
- Base para as changes `add-reserve-map-and-zone-unlock`, `add-career-guide` e `add-premium-animal-editor`.
