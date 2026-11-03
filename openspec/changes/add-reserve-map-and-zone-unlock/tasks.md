## 1. Dados

- [x] 1.1 Ler mapas de spawn da reserva (atlas → AAF → SARC → ADF) e montar o índice chave→(espécie, tipo, centro, AabbMin)
- [x] 1.2 Ler `animal_population_N`: populações, grupos, rotas e animais; espécie por maioria das chaves
- [x] 1.3 Ler `found_need_zones_adf` e montar a tabela de horários por espécie/reserva

## 2. Mapa

- [x] 2.1 Renderizar fundo do mapa a partir da instalação com calibração por reserva (sem cache: a textura decodifica em milissegundos; 14 reservas têm imagem)
- [x] 2.2 Zonas com filtros por espécie, tipo e horário (com virada da meia-noite)
- [x] 2.3 Rastreador de grupo no painel de espécies: rota, horários, destaque da hora, animais do grupo
- [x] 2.4 Coordenadas do cursor iguais às do jogo
- [x] 2.7 Reservas reconhecidas pelo save e pelo jogo, com capa (tela inicial e faixa do mapa)
- [x] 2.8 Aviso quando o alinhamento da imagem é calculado (`approximate`)
- [x] 2.5 Interface no estilo Apple: barra lateral com ícones, capas das reservas reconhecidas, chaves e controle segmentado
- [x] 2.6 Imagem do mapa das cinco reservas mais novas (alinhamento calculado pelas zonas)
- [ ] 2.9 Confirmar com quem tem os DLCs que as imagens de Askiy Ridge (19) e Intisuyu (21) são as certas e fixar o retângulo delas

## 3. Liberar zonas

- [x] 3.1 Calcular zonas faltantes com posição, altura e horário (votação entre paradas vizinhas)
- [ ] 3.2 Tela de prévia por reserva com contagem e lista (feita a contagem por reserva; falta a lista das zonas)
- [x] 3.3 Escritor de `found_need_zones_adf` com reconstrução completa e verificação (sem y=1,0, cadeia íntegra, zonas antigas idênticas)
- [x] 3.4 Integração com `SaveTransaction`

## 4. Verificação

- [x] 4.1 Teste com save real: ida e volta sem mudanças idêntica
- [ ] 4.2 Teste manual no jogo: zonas aparecem no mapa do jogo na posição certa
