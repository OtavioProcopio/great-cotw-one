## Context

Ver proposal.md (Why). O conhecimento dos formatos está em `.claude/skills/cotw-save-knowledge/SKILL.md` e o código provado em `research/reference/` (Python e JS usados nas páginas protótipo: mapa de zonas, Central do Caçador, editor de animais e build). Ponto crítico já aprendido: editar um ADF mudando o tamanho sem reconstruir a cadeia de relocação (campo +4 de cada ponteiro) fez o jogo travar.

## Goals / Non-Goals

**Goals:** um núcleo de formatos testável e isolado da interface; uma única porta de escrita com as garantias de `safe-save-writing`; instalador pequeno para Windows 10/11.

**Non-Goals:** macOS/Linux, consoles, saves de multiplayer, sincronização em nuvem, telemetria.

## Decisions

1. **Tauri 2 (Rust + WebView2) em vez de Electron ou .NET/WPF.** Instalador de poucos MB, acesso nativo a processos e arquivos em Rust, e a interface web reaproveita os protótipos e os parsers JS que já funcionam. Electron daria o mesmo reaproveitamento com instalador ~10× maior; WPF exigiria reescrever toda a interface.
2. **Núcleo de formatos em Rust (`crates/cotw-formats`)**: save envelope, ADF genérico (leitura e escrita com reconstrução de ponteiros e cadeia), tab/arc, AAF, SARC, RTPC, XLS book, hashlittle. A interface só recebe JSON já decodificado. Os parsers JS de `research/reference/core.js` servem de oráculo nos testes.
3. **Todas as gravações passam por `SaveTransaction`** no Rust: checar processo → backup (cópia da pasta inteira para `%APPDATA%\Great COTW One\backups\<data>`) → aplicar → reler/validar → commit no histórico (SQLite local) ou rollback. Nenhum comando da interface escreve arquivo diretamente.
4. **Leitura do jogo por hash de caminho**: indexar todos os `.tab` uma vez (≈90 mil entradas), resolver caminhos conhecidos (`settings/hp_settings/player_skills.bin`, `missions/*.hpmissionsc`, spawn maps…) e validar a assinatura de cada achado para descartar colisões de 32 bits. Cache em `%LOCALAPPDATA%`, invalidado pelo tamanho/data dos `.tab`.
5. **Testes de ida e volta**: fixtures vêm de saves do desenvolvedor numa pasta fora do git (`fixtures/` no `.gitignore`); os testes pulam quando a pasta não existe.

## Risks / Trade-offs

- [Patch do jogo muda um formato] → cada leitor valida assinatura/campos e desliga só a função afetada; tela de diagnóstico mostra o que falhou.
- [WebView2 ausente em Windows antigos] → usar o bootstrapper oficial do Tauri no instalador.
- [Antivírus estranhar app que lê pastas de jogo] → binário assinado quando houver certificado; nenhum acesso à memória do jogo.
- [Usuário apaga backups manualmente] → histórico avisa quando o backup referenciado não existe mais.

## Migration Plan

Primeira versão; não há migração. Rollback = restaurar backup pela própria tela de histórico.

## Open Questions

- Nome definitivo do app e ícone (não muda specs).
- Certificado de assinatura de código (afeta só distribuição).
