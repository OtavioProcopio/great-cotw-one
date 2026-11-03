## Context

Ver proposal.md. Algoritmos já provados no jogo real: `research/reference` (`nzio.py`, `nzwrite.py`, `nzfile.py`) e seções 3.1, 3.2, 4.1 e 5 da skill de conhecimento. O jogo aceitou as zonas injetadas e regravou o arquivo depois.

## Goals / Non-Goals

**Goals:** mapa rápido com dezenas de milhares de zonas; cálculo determinístico das zonas a liberar.
**Non-Goals:** mostrar posição real de animais (o save grava `MapPosition` sempre 0), revelar ícones e mover estruturas (changes futuras).

## Decisions

1. **Fonte das áreas**: mapas de spawn por espécie dentro do atlas da reserva (`spawn_<espécie>.bmp_datac` ou `reserve_N_<espécie>.bmp_datac`; camadas comer/beber/descansar). Na versão atual do jogo cada área é uma lista de células de uma grade 256 x 256, sem caixa pronta. Posição = centro da caixa das células x 32 m + deslocamento fixo por reserva, que o app descobre nas zonas que o próprio jogo gravou (erro zero em 14 reservas). Altura = média das zonas vizinhas gravadas pelo jogo (mediana de erro 2 m); nunca 1,0.
2. **Espécie do grupo**: maioria das chaves de área da rota (o `NameHashId` da população não cobre todas as espécies).
3. **Horários**: tabela por espécie/tipo a partir das zonas que o jogo gravou (manter faixas vistas ≥2 vezes e ≥20% da mais comum). Para cada zona nova: primeiro a faixa cujo índice de horário é igual à posição da parada na rota (o jogo grava assim), desde que caiba entre as paradas conhecidas vizinhas; senão a primeira que caiba; senão a mais comum, marcada como estimada. Acerta o horário exato em cerca de 67% das zonas escondidas de teste.
4. **Escrita**: reconstruir a instância inteira de `found_need_zones_adf` (alinhamento 8, ponteiros, cadeia, tamanho), copiar o campo @40 mais comum da espécie no save, id único aleatório de 8 bytes.
5. **Imagem do mapa**: decodificar em tempo de execução a textura de mapa (AVTX, BC1) da instalação do usuário, uma por reserva, e entregá-la à interface como JPEG em data URL; nunca embutir no instalador. O quadrado do mundo que a imagem cobre vem de uma tabela por reserva. Reservas sem textura conhecida (as mais novas) mostram as zonas sobre fundo liso.

## Risks / Trade-offs

- [Altura aproximada em terreno acidentado] → usar a altura de zona vizinha gravada pelo jogo quando houver uma a menos de 50 m.
- [Zona liberada num horário errado] → marcar no app como "horário estimado"; o jogo corrige ao visitar.
- [Atualização do jogo muda chaves de área] → zonas cuja chave não existe no atlas atual são ignoradas e listadas no diagnóstico.

## Open Questions

- Calibração exata do mapa para reservas novas (só afeta a imagem de fundo).

## Atualização: imagens de mapa e reconhecimento das reservas

- **Imagem do mapa**: o caminho dela não consta em nenhuma configuração do jogo. As imagens são texturas BC3 de 1024 x 1024 com exatamente 1.398.256 bytes, guardadas em blocos regulares dos `game77/78`; a tabela `MAP_TEXTURES` liga cada hash a uma reserva. As cinco mais novas (10, 18, 19, 20, 21) foram achadas nesse bloco e ligadas por formato do território e ordem no arquivo; 19 e 21 precisam de confirmação de quem tem esses DLCs.
- **Alinhamento**: reservas sem retângulo conhecido usam `infer_rect` (centro da parte iluminada da imagem = centro das zonas, 16 m por pixel). Validado nas 14 reservas conhecidas: erro de 96 a 944 m num mapa de 16 km. O app marca `approximate` e mostra o aviso.
- **Reservas e capas**: a lista vem das reservas com dados no save e no jogo; a capa é o `select_reserve_image` citado em `reserve_N.bin`, reduzida a 360 px.
- **Decodificador**: aceita cadeia de mips cortada (necessário para as capas).
