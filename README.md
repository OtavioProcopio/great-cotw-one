# Great COTW One

App para Windows que lê o seu save do **theHunter: Call of the Wild** e os arquivos do jogo instalado e mostra tudo o que ajuda na carreira.

| Gratuito | Premium |
|---|---|
| Mapa de zonas de necessidade com horários e rastreador de grupos | Editor de animais: pontuação, peso, sexo, pelagem, Great One |
| **Liberar zonas** que os seus grupos usam, com horários calculados | "1 diamante por grupo" e outros atalhos |
| Missões por cadeia, conquistas da Steam, contagens e o que falta | Criar animais num grupo (experimental) |
| Guia de pelagens com chances reais do jogo | |
| Planejador de build (habilidades e vantagens) | |

Projeto de fã, sem ligação com a Expansive Worlds / Avalanche Studios. Só single-player.

## Regras do projeto

- Backup automático do save antes de qualquer gravação; nunca grava com o jogo aberto.
- Nada de arquivos do jogo no repositório ou no instalador: o app lê a instalação do usuário.
- Toda edição é mostrada antes e verificada depois.

## Como o projeto é desenvolvido

Usamos [OpenSpec](https://github.com/Fission-AI/OpenSpec) (spec-driven):

```
openspec/
  config.yaml          contexto do projeto (lido pelos agentes)
  specs/               specs vigentes (preenchidas ao arquivar changes)
  changes/             propostas em andamento
    bootstrap-app-foundation/          base: descoberta, leitura, escrita segura (concluída)
    add-reserve-map-and-zone-unlock/   mapa, rastreador e liberar zonas (free; falta o teste no jogo)
    add-player-profile-and-animal-wiki/ perfil, conquistas e wiki de animais (free; entregue em grande parte)
    add-career-guide/                  missões, conquistas, pelagens, build (free; missões por reserva entregues)
    add-premium-animal-editor/         editor de animais + licença (premium)
```

## Estado atual

Abas prontas: **Início**, **Perfil** (marcas de caça e conquistas da Steam), **Animais** (wiki por reserva), **Mapa** (zonas, rastreador de grupos), **Missões** (andamento por reserva), **Liberar mapa**, **Backups** e **Ajustes**. Tudo é montado com o save e o jogo de cada pessoa, em português. Veja `docs/DEVELOPMENT.md` e as tarefas de cada change em `openspec/changes/`.

Fluxo com Claude Code (comandos já instalados em `.claude/`): `/opsx:explore` → `/opsx:propose <ideia>` → `/opsx:apply <change>` → `/opsx:archive <change>`. Validar: `npx @fission-ai/openspec validate --all`.

A base (`bootstrap-app-foundation`) está concluída; as outras dependem dela.

## Conhecimento do jogo

- `.claude/skills/cotw-save-knowledge/SKILL.md` — formatos do save e do jogo, hashes, offsets, o que é seguro editar e o que já quebrou o jogo. Leia antes de mexer em qualquer arquivo.
- `research/reference/` — implementações de referência (Python e JS) usadas nos protótipos e testadas em save real. Servem de oráculo para os testes do app.
