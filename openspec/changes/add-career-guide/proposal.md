## Why

Platinar o jogo exige saber o que falta: quais missões estão abertas em cada cadeia, quantos abates faltam para cada conquista, qual animal falta fotografar, onde conseguir uma pelagem rara e como distribuir os pontos de habilidade. Tudo isso está no save e nos arquivos do jogo, mas o jogo não mostra junto. Esta change entrega o "know-how da carreira" na versão gratuita.

## What Changes

- Painel de carreira: missões por reserva e por cadeia (com quem dá a missão, etapa e cada objetivo), conquistas da Steam com status, progresso de cada conquista (contagens, objetivos e itens que faltam), vitórias rápidas.
- Guia de pelagens: chance real de cada pelagem por espécie e sexo, raridade, Great One, diamantes por reserva no save.
- Planejador de build: árvores de habilidades e vantagens na posição do jogo, com as regras de desbloqueio, carregando a build do save; gravar a build é opcional e passa pela escrita segura.

## Capabilities

### New Capabilities
- `career-progress`: missões, cadeias, conquistas e progresso detalhado.
- `fur-guide`: pelagens, chances e raridades por espécie e reserva.
- `skill-build-planner`: ver, planejar e (opcional) aplicar a build de habilidades e vantagens.

### Modified Capabilities
(nenhuma)

## Impact

- Lê `missions_missiondata_adf`, `achievements_progress_missiondata_adf`, `thp_player_profile_adf`, `animal_population_N`, o cache de estatísticas da Steam e, do jogo: `missions/*.hpmissionsc`, `missions/missions.stringlookup`, `global/global_animal_types.blo`, `settings/hp_settings/player_skills.bin` e `player_skill_trees.bin`.
- Escreve apenas `thp_player_profile_adf` (build), pela escrita segura.
