## Context

Ver proposal.md. Algoritmos já provados no jogo real: `research/reference` (`nzio.py`, `nzwrite.py`, `nzfile.py`) e seções 3.1, 3.2, 4.1 e 5 da skill de conhecimento. O jogo aceitou as zonas injetadas e regravou o arquivo depois.

## Goals / Non-Goals

**Goals:** mapa rápido com dezenas de milhares de zonas; cálculo determinístico das zonas a liberar.
**Non-Goals:** mostrar posição real de animais (o save grava `MapPosition` sempre 0), revelar ícones e mover estruturas (changes futuras).

## Decisions

1. **Fonte das áreas**: mapas de spawn por espécie dentro do atlas da reserva (`spawn_<espécie>.bmp_datac`, camadas comer/beber/descansar). Centro do AABB = posição; `AabbMin.y + 128,7` = altura (mediana medida nas zonas gravadas pelo jogo).
2. **Espécie do grupo**: maioria das chaves de área da rota (o `NameHashId` da população não cobre todas as espécies).
3. **Horários**: tabela por espécie/reserva a partir das zonas que o jogo gravou (manter faixas vistas ≥2 vezes e ≥20% da mais comum); para cada zona nova, votar a faixa que cabe entre as paradas conhecidas vizinhas.
4. **Escrita**: reconstruir a instância inteira de `found_need_zones_adf` (alinhamento 8, ponteiros, cadeia, tamanho), copiar o campo @40 mais comum da espécie no save, id único aleatório de 8 bytes.
5. **Imagem do mapa**: renderizar a partir das texturas de mapa da instalação do usuário, em cache local; nunca embutir no instalador.

## Risks / Trade-offs

- [Altura aproximada em terreno acidentado] → usar a altura de zona vizinha gravada pelo jogo quando houver uma a menos de 50 m.
- [Zona liberada num horário errado] → marcar no app como "horário estimado"; o jogo corrige ao visitar.
- [Atualização do jogo muda chaves de área] → zonas cuja chave não existe no atlas atual são ignoradas e listadas no diagnóstico.

## Open Questions

- Calibração exata do mapa para reservas novas (só afeta a imagem de fundo).
