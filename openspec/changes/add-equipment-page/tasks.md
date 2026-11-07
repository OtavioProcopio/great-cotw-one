## 1. Decodificação do catálogo

- [x] 1.1 Gerador `tools/gen-equipment` com id, categoria, nome em inglês, preço, peso, caixa, classe e ícone dos 437 itens (peles e suportes de troféu ficam de fora) em `equipment.json`
- [x] 1.2 Decodificados preço (`cash_price`), peso (0x3d214106), classe da arma (0x765fbed2: rifle, pistola, espingarda, arco) e unidades por caixa (0xb19e6900); o nível de desbloqueio não está nesse arquivo (campo 0xb304f782 não decifrado, fica de fora)
- [x] 1.3 Ligação arma↔munição e arma↔mira decodificada (nó filho 0x77c2b3ba); dano e balística NÃO estão em `equipment_data.bin` nem em `ammunition.bin` (este só tem multiplicadores), então o recomendador por dano depende de achar essa fonte
- [ ] 1.4 Verificar nomes em português e espanhol nos textos de localização do jogo

## 2. Leitura (grátis)

- [x] 2.1 Núcleo: `cotw-core::equipment` lê posse, quantidades, `WeaponData` por item e loadouts do perfil (veículos: `VehicleData` vem vazio no save de teste, ainda não lido)
- [x] 2.2 Ícones lidos do jogo do usuário, convertidos em PNG e guardados em cache local; sem o jogo, a página usa o símbolo da categoria
- [x] 2.3 Página Equipamentos: arsenal por categoria com filtro, busca e progresso de coleção
- [x] 2.4 Ficha da arma e ranking pessoal
- [x] 2.5 Loadouts e itens que faltam
- [x] 2.6 Textos em pt-BR, EN e ES e testes de interface

## 3. Recomendador

- [ ] 3.1 Pontuação de adequação por espécie (classe de arma, dano, distância) com o motivo
- [ ] 3.2 Tela de recomendação com indicação de posse

## 4. Escrita (Premium)

- [ ] 4.1 Entregar item em vaga livre, mesmo tamanho, com backup e jogo fechado
- [ ] 4.2 Quantidade de munição e consumíveis, e veículos
- [ ] 4.3 Montar loadout com checagem de compatibilidade
- [ ] 4.4 Dinheiro, depois de testar o comportamento do jogo
- [ ] 4.5 Bloqueio duplo (`premiumRequired`) e atualização da tela do Premium

## 5. Verificação

- [ ] 5.1 Testes com saves sintéticos (mesmo tamanho, demais bytes idênticos; vale para as escritas da seção 4)
- [ ] 5.2 Teste no jogo: um item por categoria, quantidade, veículo, loadout e dinheiro
