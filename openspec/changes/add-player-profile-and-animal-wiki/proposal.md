## Why

Depois do mapa, o jogador quer ver o próprio progresso num lugar só (nível, marcas, conquistas) e consultar os animais de cada reserva sem sair do app. Tudo isso já está no save, na Steam instalada e nos arquivos do jogo; falta juntar e mostrar em português simples, e de um jeito que funcione para qualquer pessoa e qualquer save, tenha ela quais expansões tiver.

## What Changes

- Aba **Perfil**: nível, experiência, dinheiro, abates, precisão de tiro, maior pontuação, tiro mais longe, troféus, reservas jogadas e as conquistas da Steam (nome e descrição em português, obtidas e faltantes, filtro e busca).
- Aba **Animais**: wiki das espécies de cada reserva do jogo, com a capa da reserva, o que o save sabe de cada espécie (animais, sexos, maior peso e pontuação, Great Ones) e os cinco melhores abates da conta; cada ficha mostra em que outras reservas a espécie vive.
- Aba **Missões** (parte já entregue de `add-career-guide`): progresso por reserva montado só com o save e os arquivos do jogo.
- Barra lateral com ícones e telas divididas por assunto (Início, Perfil, Animais, Mapa, Missões, Liberar mapa, Backups, Ajustes).

## Capabilities

### New Capabilities
- `player-profile`: perfil do jogador com marcas de caça e conquistas da Steam.
- `animal-wiki`: espécies por reserva, com dados do save e do diário de caça.

### Modified Capabilities
(nenhuma)

## Impact

- Lê `thp_player_profile_adf`, `trophy_lodges_adf`, `hunting_log_adf`, `animal_population_N`, `missions_missiondata_adf` e, do jogo, `settings/hp_settings/reserve_N.bin`, os mapas de spawn e `missions/*`; da Steam, `appcache/stats/UserGameStatsSchema_518790.bin` e `UserGameStats_<conta>_518790.bin`.
- Não escreve nada no save.
- Depende de `bootstrap-app-foundation` e `add-reserve-map-and-zone-unlock`.
