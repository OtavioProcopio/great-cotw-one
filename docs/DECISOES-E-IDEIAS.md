# Decisões e ideias

Registro vivo do que foi decidido e do que surgiu no caminho. Atualize quando algo mudar.

## Decisões tomadas

- **Sem edição em lote.** Os animais se editam um a um, para não desequilibrar o jogo.
- **Troca de sexo** fica como está.
- **Pelagem:** só volta à pauta quando soubermos como a seed vira pelagem. Hoje o app mostra nomes e chances, mas não edita.
- **Dados do jogo embutidos como fatos numéricos** (`crates/cotw-core/data/species_stats.json`), gerados por `tools/gen-species-stats`. Nenhum arquivo, textura ou texto do jogo vai para o repositório. Rode o gerador de novo quando o jogo atualizar.
- **Diamante** = pontuação mínima + 90 % da faixa de pontos do macho da espécie (conferido com guias da comunidade). Bronze abaixo de 20 %, prata até 60 %, ouro até 90 %.
- **Imagens do jogo:** o usuário quer retrabalhá-las com arte própria. Os retratos dos animais não foram encontrados (os caminhos chutados eram colisões de hash); ficam dentro dos `.gfx` da interface. Os ícones por classe são provisórios.
- **Aviso de PC fraco** ao liberar mais de 8.000 lugares; **aviso de ponto aproximado** no rastreador de grupo.

## Ideias em aberto

1. **Teste grátis:** uma edição de animal por instalação (contador local); o resto é Premium.
2. **Reagrupar animais:** trocar as áreas da rota de cada grupo (`NeedZonePathGuids`, edição do mesmo tamanho) para que vários grupos frequentem as mesmas áreas, perto de lagos. Não testado no jogo; começar com um grupo, prévia e backup.
3. **Página de equipamentos:** o save traz posse, quantidades, estatísticas por arma (tiros, acertos, abates, recorde, tiro mais distante em órgão vital), loadouts, veículos e skins. Nomes, descrições e ícones vêm de `settings/hp_settings/equipment_data.bin` (991 itens: 138 armas, 118 munições, 79 iscas e chamados, 44 miras, 411 skins etc.; todos os ícones existem nos arquivos). Os números (peso, preço, nível) ainda não foram decodificados.
4. **Códices:** ainda não inspecionados (`codex_seen_status_adf` e os textos do jogo).
5. **Testes com saves sintéticos** para voltar a cobertura a 80 %.
6. **Publicar a v0.1.0**, testar o instalador e subir o servidor de licenças na VPS.

## Mercado (pesquisa de nov/2026)

- Concorrente mais próximo: *Call of the Wild Tools*, analisador só de leitura, compra única por volta de US$ 9,99 (informação de listagens de terceiros). O APC é o único editor gratuito e o original não é mais mantido.
- Público na Steam: 2 mil a 7 mil jogadores simultâneos, pico histórico perto de 25 mil. Discord oficial com cerca de 135 mil membros.
- Nenhuma ferramenta achada junta liberar mapa, rastrear grupos e editar animais com backup.
