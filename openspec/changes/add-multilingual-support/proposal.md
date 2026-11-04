## Why

O app é global: jogadores do mundo todo usam theHunter: Call of the Wild. A interface precisa falar a língua de quem usa, e o que vem do jogo (nomes de animais, textos de missões e de conquistas) também.

## What Changes

- Interface em português do Brasil, inglês e espanhol, escolhida pelo jogador em Ajustes, com o idioma do sistema como padrão.
- Nomes das espécies, textos das missões (do próprio jogo) e nomes e descrições das conquistas (da Steam) acompanham o idioma escolhido.
- Formatos de número, data e dinheiro seguem o idioma.

## Capabilities

### New Capabilities
- `app-localization`: idiomas da interface e do conteúdo vindo do jogo.

### Modified Capabilities
(nenhuma)

## Impact

- Interface: um dicionário por idioma em `ui/src/locales/`; o app (Rust) recebe o idioma pelo comando `set_language`.
- Lê, no jogo, a língua certa de `missions/*.stringlookup` (0 inglês, 3 espanhol, 7 português do Brasil) e, na Steam, o idioma do esquema de conquistas.
- Não grava nada no save.
