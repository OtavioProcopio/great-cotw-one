## Why

Todo jogador mexe com equipamentos o tempo todo (armas, munição, miras, iscas, veículos), mas o jogo não mostra o histórico de cada item nem ajuda a montar o conjunto certo para cada animal. O save guarda tudo isso: posse, quantidades, estatísticas por arma, loadouts e veículos. Hoje o app só soma os dados de armas num total geral.

## What Changes

- Nova página **Equipamentos** (grátis, só leitura): arsenal por categoria com ícone, posse e quantidade; ficha de cada arma (tiros, acertos, precisão, abates, abates de um tiro, recorde com a espécie, tiro mais longe em órgão vital, data da compra); ranking pessoal das armas; loadouts e itens que faltam; progresso de coleção por categoria.
- **Premium** (escreve no save): liberar ou entregar itens, ajustar a quantidade de munição e consumíveis, reabastecer e consertar veículos, montar loadouts com checagem de compatibilidade e ajustar o dinheiro.
- **Recomendador**: dada uma espécie, sugerir arma e munição adequadas (exige decodificar dano e classe das armas).
- Dados do catálogo (nome, categoria, números) embutidos como fatos numéricos, gerados a partir do jogo do próprio usuário; os ícones são extraídos na hora para um cache local e nunca vão para o repositório.

## Capabilities

### New Capabilities
- `equipment-page`: leitura e exibição de equipamentos, estatísticas por arma, loadouts, veículos e recomendação.
- `equipment-editor`: escrita de itens, quantidades, veículos, loadouts e dinheiro, com as regras de segurança.

### Modified Capabilities
- `premium-access`: a lista de vantagens do Premium passa a incluir as escritas de equipamento.

## Impact

- Lê `thp_player_profile_adf` por item (hoje só soma). Escreve esse mesmo arquivo, sempre no mesmo tamanho (arrays de 1024 entradas), com backup e jogo fechado.
- Novo gerador `tools/gen-equipment` (a partir de `settings/hp_settings/equipment_data.bin` e `settings/ammunition.bin`) e novo dado `crates/cotw-core/data/equipment.json`.
- Novo extrator de ícones para cache local (usa o leitor de texturas existente).
- Depende de `add-premium-animal-editor` (licença e bloqueio duplo).
