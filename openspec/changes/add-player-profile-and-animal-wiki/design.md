## Context

Ver proposal.md. Princípio da change: o app se monta com o que existe no computador de cada pessoa, sem tabelas por reserva quando o jogo, o save ou a Steam já trazem a informação. Fontes e mapeamentos provados estão em `docs/DEVELOPMENT.md` (seções Perfil e Missões, Animais).

## Goals / Non-Goals

**Goals:** funcionar para qualquer save e qualquer combinação de expansões; texto em português; nada gravado no save.
**Non-Goals:** imagens de animais (o jogo não traz uma por espécie e as da wiki têm direitos próprios); classes 1 a 9 das espécies (vêm de `global_animal_types.blo`, fora desta change); ler a API web da Steam.

## Decisions

1. **Missões por reserva sem tabela**: a configuração de cada reserva (`reserve_N.bin`) cita o arquivo `missions/missions_K.hpmissions`; o app lê `missions_K.hpmissionsc` (definições) e `missions_K.stringlookup` (textos). O texto é achado pelos 32 bits de baixo do hash da descrição.
2. **Conquistas pela Steam local**: esquema (`UserGameStatsSchema`) dá nomes e descrições em português; o cache da conta (`UserGameStats_<id>`) dá os bits das obtidas; a conta vem do Steam ID do save menos 76561197960265728. Sem os arquivos, o perfil aparece sem conquistas e explica por quê.
3. **Perfil do save**: `thp_player_profile_adf` (nível, XP, dinheiro, `WeaponData` somado: tiros, abates, recordes) e `trophy_lodges_adf` (troféus).
4. **Wiki por espécie**: as espécies de cada reserva vêm dos mapas de spawn do jogo; os animais vivos vêm de `animal_population_N` com a espécie de cada grupo por maioria das chaves da rota; os melhores abates vêm de `hunting_log_adf` (`TopHarvestsBySpecies`, chave `hashlittle(id da espécie)`).
5. **Ícone por espécie**: tom derivado do identificador, para a lista ser reconhecível sem imagens de terceiros.

## Risks / Trade-offs

- [Cerca de 10% das missões sem texto] → mostrar "Missão N".
- [Nomes de espécies de reservas novas em inglês] → tabela de nomes em português cresce junto; sem tradução, o app mostra o identificador legível.
- [Conquistas secretas] → só mostram o nome depois de obtidas.
