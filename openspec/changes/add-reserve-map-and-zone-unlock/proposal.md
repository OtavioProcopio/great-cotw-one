## Why

O que mais trava o jogador é achar os animais: o jogo só mostra uma zona de necessidade depois que você a descobre, e não diz em que horário cada grupo passa por ela. O save já guarda a rota diária de cada grupo e o jogo guarda a área de cada zona; juntando os dois dá para mostrar onde cada grupo está a cada hora e liberar no jogo as zonas que os grupos do usuário realmente usam. Isso é o coração da versão gratuita.

## What Changes

- Mapa da reserva com as zonas de necessidade (comer, beber, descansar) do save, inclusive as ainda não descobertas que algum grupo usa.
- Filtro por espécie, por tipo de necessidade e por horário; rastreador de grupo junto do filtro de espécie, com a rota do dia, os horários de cada parada e onde o grupo deve estar na hora escolhida.
- Lista de animais de cada grupo (sexo, peso, pontuação, marca de diamante e Great One) — somente leitura na versão gratuita.
- Reservas reconhecidas sozinhas a partir do save e do jogo, com a capa de cada uma, e imagem do mapa do próprio jogo por baixo das zonas.
- Função gratuita "Liberar zonas": grava no save as zonas que os grupos atuais usam e o usuário ainda não descobriu, com posição, altura e horário calculados.

## Capabilities

### New Capabilities
- `reserve-map`: visualização das zonas e dos grupos da reserva com horários.
- `need-zone-unlock`: liberar no save as zonas de necessidade usadas pelos grupos, com horários.

### Modified Capabilities
(nenhuma)

## Impact

- Lê `animal_population_N`, `found_need_zones_adf` e os spawn maps do jogo (atlas da reserva).
- Escreve somente `found_need_zones_adf`, pela escrita segura de `bootstrap-app-foundation`.
- Depende de `bootstrap-app-foundation`.
