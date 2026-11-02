## 1. Dados

- [ ] 1.1 Ler mapas de spawn da reserva (atlas → AAF → SARC → ADF) e montar o índice chave→(espécie, tipo, centro, AabbMin)
- [ ] 1.2 Ler `animal_population_N`: populações, grupos, rotas e animais; espécie por maioria das chaves
- [ ] 1.3 Ler `found_need_zones_adf` e montar a tabela de horários por espécie/reserva

## 2. Mapa

- [ ] 2.1 Renderizar fundo do mapa a partir da instalação (cache local) com calibração por reserva
- [ ] 2.2 Zonas com filtros por espécie, tipo e horário (com virada da meia-noite)
- [ ] 2.3 Rastreador de grupo no painel de espécies: rota, horários, destaque da hora, animais do grupo
- [ ] 2.4 Coordenadas do cursor iguais às do jogo

## 3. Liberar zonas

- [ ] 3.1 Calcular zonas faltantes com posição, altura e horário (votação entre paradas vizinhas)
- [ ] 3.2 Tela de prévia por reserva com contagem e lista
- [ ] 3.3 Escritor de `found_need_zones_adf` com reconstrução completa e verificação (sem y=1,0, cadeia íntegra, zonas antigas idênticas)
- [ ] 3.4 Integração com `SaveTransaction`

## 4. Verificação

- [ ] 4.1 Teste com save real: ida e volta sem mudanças idêntica
- [ ] 4.2 Teste manual no jogo: zonas aparecem no mapa do jogo na posição certa
