# Desenvolvimento

App Tauri 2: backend em Rust, interface em TypeScript (Vite).

## Estrutura

```
Cargo.toml               workspace: versão, edição e lints compartilhados
rust-toolchain.toml      versão do Rust fixa para todo mundo
rustfmt.toml / clippy.toml / .editorconfig
.cargo/config.toml       atalhos (cargo lint, cargo cov ...)
crates/
  cotw-formats/          formatos de arquivo: só bytes <-> estruturas (envelope, ADF leitor e
                         escritor, .tab/.arc, AAF, SARC, RTPC, XLS). Sem disco do usuário.
  cotw-core/             regras que tocam o computador: descoberta do save e do jogo, processo do
                         jogo, backup, histórico (SQLite), SaveTransaction, diagnóstico, serviço
src-tauri/               casca do app: comandos Tauri que só repassam ao AppService
ui/                      interface (TypeScript, sem framework): telas, textos pt-BR, testes
scripts/                 geradores dos oráculos (Python) e do ícone
fixtures/                saves e arquivos do desenvolvedor, fora do git
openspec/                specs e changes (fluxo spec-driven, ver README)
```

Dependências: `ui` → (comandos Tauri) → `src-tauri` → `cotw-core` → `cotw-formats`. Nada aponta para trás.
Toda escrita no save passa por `cotw-core::transaction::SaveTransaction`.

## Comandos

| Comando | O que faz |
|---|---|
| `cargo test --workspace` | roda todos os testes Rust |
| `cargo fmt --all` / `cargo fmt-check` | formata / confere formatação |
| `cargo lint` | Clippy com warnings tratados como erro |
| `cargo cov` / `cargo cov-html` | cobertura no terminal / relatório HTML |
| `npm --prefix ui ci` | instala o frontend (uma vez) |
| `npm --prefix ui run typecheck` / `lint` / `test` / `coverage` | checagens do frontend |
| `npm --prefix ui run dev` | servidor da interface; abra `http://localhost:1420/?mock` no navegador para ver as telas com dados de exemplo |
| `ui/node_modules/.bin/tauri dev` | app completo em desenvolvimento |
| `ui/node_modules/.bin/tauri build` | instalador NSIS em `target/release/bundle/nsis` |

Na primeira compilação o Tauri leva alguns minutos. O perfil de desenvolvimento usa `opt-level = 1`
porque os testes com saves reais processam centenas de milhares de ponteiros.

## Ferramentas a instalar uma vez

1. Rust via [rustup](https://rustup.rs) e, no Windows, as *Build Tools do Visual Studio* com "Desktop development with C++".
2. `cargo install cargo-llvm-cov --locked` (cobertura).
3. Node.js 22 ou mais novo.
4. WebView2 (já vem no Windows 11).

## Convenções de teste

- **Unitário (Rust)**: `#[cfg(test)] mod tests` no próprio arquivo.
- **Integração (Rust)**: um arquivo por assunto em `crates/<crate>/tests/`; ajudantes em `tests/common/`.
- **Frontend**: `ui/src/**/*.test.ts` (Vitest + jsdom); API do Tauri falsa em `ui/src/test-data.ts`.
- Testes que dependem de saves ou do jogo reais **pulam** se não existirem; o CI roda só o que não depende deles.
- Cobertura mínima no CI: 80% das linhas (Rust) e limites em `ui/vite.config.ts`.
- Todo formato novo precisa de teste de ida e volta e, quando existir, comparação com `research/reference/`.
- Ao mudar o escritor ADF: a propriedade "crescer arrays e reverter devolve os bytes originais" nos saves reais
  (`writer_fixture`) e a conferência interna (`rewrite_arrays` relê e valida a cadeia) são a rede de segurança.

## Oráculos (comparação com o código de referência em Python)

Os testes com arquivos reais comparam o Rust com `research/reference/`. Os arquivos gerados ficam em
`fixtures/`, fora do git (derivam do save e do jogo do desenvolvedor).

- ADF de um save: `python -I scripts/make_adf_oracle.py research/reference fixtures/animal_population_0 fixtures/animal_population_0.adf.oracle.txt`
- Saves valor a valor: copie os arquivos de save para `fixtures/saves/` e rode `python -I scripts/make_save_oracle.py research/reference fixtures/saves`.
  O índice guarda só contagens e SHA-256; para investigar uma divergência acrescente `--dump <pasta fora do OneDrive>`
  (gera centenas de MB) e compare com `diff`.
- Arquivos do jogo: `python -I scripts/make_game_oracle.py "<jogo>/archives_win64" research/reference fixtures/game.oracle.txt`.
  O teste usa `COTW_GAME_DIR` (ou o caminho padrão da Steam).
- O `rtpc.py` guarda propriedades num `dict` e perde as de hash repetido; o leitor em Rust mantém todas
  (o teste compara com a mesma semântica do dict).
- Ícone: `python -I scripts/make_icon.py app-icon.png` e `ui/node_modules/.bin/tauri icon app-icon.png`.

## Teste manual 5.2 (gravar sem mudanças e abrir o jogo)

Confirma que o jogo aceita um arquivo regravado pelo app. Tudo pela janela, na tela **Início**:

1. Feche o jogo e abra o Great COTW One (`target/release/great-cotw-one.exe` ou o instalador).
2. No cartão **Testar se está tudo funcionando**, clique em **Fazer o teste** e confirme.
3. O app passa pela `SaveTransaction`: confere que o jogo está fechado, faz o backup completo em
   `%APPDATA%\Great COTW One\backups\<data>`, regrava um arquivo (mesmo conteúdo, recomprimido), relê e confere,
   e registra no histórico. O arquivo usado é `animal_population_2` (ou outra população, se faltar).
4. Abra o jogo, carregue a carreira e confirme que a reserva abre normalmente.
5. Se algo estiver errado, volte ao app, aba **Backups**, e clique em **Voltar para este momento**.

## Mapa das zonas

A aba **Mapa** mostra, por reserva, o mapa do próprio jogo (decodificado da instalação na hora, nada é copiado
para o repositório nem para o instalador) com as zonas de comer, beber e descansar. Filtros por necessidade,
espécie e horário; "Seguir um grupo" desenha a rota do dia com os horários e destaca onde o grupo deve estar
no horário escolhido. Zonas que o app ainda não gravou aparecem com contorno tracejado.

- Textura do mapa: tabela `MAP_TEXTURES` e `map_rect` em `cotw-formats/src/spawn_map.rs`; decodificador AVTX
  (BC1, BC3, BC4, BGRA) em `texture.rs`. As 19 reservas do jogo têm imagem. As 14 mais antigas têm retângulo
  conhecido; as cinco mais novas (10 Te Awaroa, 18 Salzwiesen Park, 19 Askiy Ridge, 20 Tòrr nan Sithean, 21 Intisuyu)
  têm o alinhamento calculado a partir das zonas (`infer_rect` em `mapview.rs`: centro da parte iluminada da imagem
  = centro das zonas; nas 14 reservas conhecidas o erro foi de 96 a 944 m num mapa de 16 km) e o app avisa que é
  aproximado.
- Dados da tela: `cotw-core/src/mapview.rs` (comando `reserve_map`); lógica pura do frontend em `ui/src/maplogic.ts`.
- Reconhecimento das reservas instaladas: a lista vem das reservas que têm dados no save **e** nos arquivos do jogo.
  Cada uma ganha a foto de capa do próprio jogo, lida de `settings/hp_settings/reserve_N.bin` (o caminho
  `select_reserve_image/*.ddsc` está dentro do arquivo; `cover_path` e `load_cover_image` em `spawn_map.rs`), reduzida
  para 360 px (`shrink` em `mapview.rs`). Aparece na tela Início, na lista de reservas do mapa e funciona para qualquer
  reserva que o jogador tenha, sem tabela fixa.
- Como as cinco imagens novas foram achadas: o caminho do mapa não está em nenhuma configuração, mas todas as imagens de
  mapa são texturas BC3 de 1024 x 1024 com exatamente 1.398.256 bytes, guardadas em blocos regulares nos `game77/78`;
  as cinco que faltavam estavam no bloco (as DLCs vêm instaladas mesmo sem posse). A ligação textura → reserva foi feita
  pelo formato do território (caixa dos mapas de spawn) e pela ordem no arquivo; **19 e 21 (Askiy Ridge e Intisuyu)
  são a parte menos certa** e precisam de confirmação de quem tem esses DLCs. Uma reserva futura entra acrescentando o
  hash em `MAP_TEXTURES` (`spawn_map.rs`) e o nome em `RESERVES` (`discovery/files.rs`).
- Interface: barra lateral com ícones (`ui/src/icons.ts`, mesmo desenho no HTML e no canvas via `Path2D`), cartões com
  cantos arredondados, chaves liga/desliga e controle segmentado, tema claro/escuro automático e barra reduzida a ícones
  em janelas estreitas.

## Animais (wiki)

A aba **Animais** (`cotw-core/src/animals.rs`) lista todas as reservas que o jogo tem (as com mapa de spawn), com a
capa, e as espécies de cada uma (o nome em português vem de `species.rs`). Do save vem, por espécie e reserva: grupos,
animais vivos, machos e fêmeas, maior peso, maior pontuação e Great Ones (`animal_population_N`). Do diário de caça
(`hunting_log_adf`, `TopHarvestsBySpecies`; a chave é `hashlittle(id da espécie)`) vêm os cinco melhores abates da conta.
Cada ficha mostra em que outras reservas a espécie vive. Sem imagens de animais: o jogo não traz uma por espécie e a
wiki da comunidade tem direitos próprios, então usamos um ícone colorido por espécie. Os nomes das reservas novas foram
conferidos na wiki ([Reserves](https://thehuntercotw.fandom.com/wiki/Reserves)). Pendente: classe (1 a 9) de cada
espécie e dados de peso/pontuação por espécie, que vêm de `global_animal_types.blo`.

## Perfil e Missões

As abas **Perfil** e **Missões** são montadas só com o que existe no computador de cada pessoa, sem tabelas por reserva:

- **Missões** (`cotw-core/src/career.rs`, `cotw-formats/src/missions.rs`): o progresso vem de `missions_missiondata_adf`
  (uma campanha por reserva, com missões e objetivos). O arquivo de definições de cada reserva é o que a própria
  configuração dela cita (`settings/hp_settings/reserve_N.bin` → `missions/missions_K.hpmissions`; o `K` nem sempre é o
  número da reserva), então reservas de expansões novas aparecem sozinhas. O texto vem de `missions_K.stringlookup`: a
  chave é os 32 bits de baixo do hash da descrição (`Description`). Cerca de 90 % das missões têm texto; as outras
  aparecem como "Missão N".
- **Perfil**: nível, experiência, dinheiro e armas vêm de `thp_player_profile_adf` (`WeaponData`: tiros, abates, recordes);
  troféus de `trophy_lodges_adf`. As conquistas vêm da Steam instalada no computador
  (`appcache/stats/UserGameStatsSchema_518790.bin` com nomes e descrições em português e
  `UserGameStats_<conta>_518790.bin` com os bits das obtidas; `cotw-formats/src/steam_kv.rs`). Sem a Steam o app avisa e
  mostra o resto do perfil. Conquistas secretas só mostram o nome depois de obtidas.
- Pendente: nome de quem dá a missão, progresso detalhado por tipo de conquista e "o que falta" de cada uma
  (`add-career-guide`, tarefas 1.4 a 1.6).

## Teste manual 4.2 (Liberar o mapa no jogo)

Confirma que o jogo mostra no mapa as zonas que o app acrescentou. Pela janela, na tela **Início**:

1. Feche o jogo e abra o Great COTW One.
2. No cartão **Liberar o mapa**, clique em **Ver o que será liberado**. O app analisa o progresso (alguns segundos)
   e lista, por reserva, quantos lugares novos aparecerão. Nada é gravado nessa etapa.
3. Deixe marcada só uma reserva pequena para o primeiro teste e clique em **Liberar os selecionados**; confirme.
4. Abra o jogo, carregue a carreira naquela reserva e abra o mapa: os lugares de comer, beber e descansar dos
   grupos devem aparecer. Compare a posição de alguns com os que o jogo já tinha descoberto.
5. Se algo estiver errado, volte ao app, aba **Backups**, e clique em **Voltar para este momento**.

Como o cálculo foi validado (testes `zones_real` e `unlock_e2e`, com o save e o jogo reais): escondendo zonas que
o próprio jogo gravou e recalculando, a posição e o tipo saem 100% iguais, a constante da espécie 99%, o horário
exato 67% e a altura com erro mediano de 2 m. O horário que não bate é marcado como aproximado na tela.

## Princípio de interface

O jogador não conhece nada técnico, só o jogo. Telas e textos falam em "progresso", "backup" e "reserva";
caminhos, nomes de arquivo, builds e índices ficam só na aba **Detalhes técnicos** e nos "Detalhes" de cada backup.
Toda função nova precisa de um cartão na tela **Início** com título e explicação em linguagem simples, um botão
claro e confirmação antes de gravar.

## Local do projeto

Mantenha o repositório fora do OneDrive: a pasta `target/` passa de 10 GB e o OneDrive trava sincronizando.

## Limites conhecidos do escritor ADF

`Adf::rewrite_arrays` troca os elementos de arrays que já têm elementos, sem ponteiros dentro dos trechos
trocados e com variação de tamanho múltipla de 8 bytes; fora disso devolve erro em vez de gravar algo
duvidoso. Isso cobre a edição de animais. Reescrever estruturas com arrays aninhados (por exemplo, criar
zonas de necessidade, que têm a lista de grupos descobertos dentro de cada zona) exige estender o escritor
na change `add-reserve-map-and-zone-unlock`.
