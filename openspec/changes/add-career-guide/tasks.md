## 1. Missões e conquistas

- [x] 1.1 Ler definições de missões por reserva e textos (arquivo indicado pela configuração da reserva, texto por `missions_N.stringlookup`, chave = 32 bits baixos do hash da descrição); falta o nome de quem dá a missão
- [x] 1.2 Ler `missions_missiondata_adf` e montar cadeias, etapas e objetivos (aba Missões, por reserva, com filtro por estado)
- [x] 1.3 Ler o cache de estatísticas da Steam e o schema para status das conquistas (aba Perfil, em português)
- [ ] 1.4 Mapear conquista → missão de conquista e calcular progresso por tipo
- [ ] 1.5 Itens que faltam: condições dos objetivos e `ValueOccurrences`
- [ ] 1.6 Telas: visão geral e conquistas (feitas nas abas Perfil e Missões); faltam cadeias, contagens e vitórias rápidas

## 2. Pelagens

- [ ] 2.1 Ler `global_animal_types.blo` (RTPC) e calcular chances por sexo
- [ ] 2.2 Resumo do save por reserva/espécie (animais, sexos, diamantes, Great Ones)
- [ ] 2.3 Tela de pelagens com busca e filtro por reserva

## 3. Build

- [ ] 3.1 Ler `player_skills.bin` e `player_skill_trees.bin` (XLS) e montar árvores, níveis e regras
- [ ] 3.2 Ler a build do perfil e validar contra as regras
- [ ] 3.3 Editor de build com bloqueios e descrições por nível
- [ ] 3.4 Gravar a build via `SaveTransaction` e gerar a ordem de compra alternativa

## 4. Verificação

- [ ] 4.1 Conferir contagens contra a tela do jogo em um save real
- [ ] 4.2 Gravar uma build, abrir o jogo e conferir a tela de habilidades
