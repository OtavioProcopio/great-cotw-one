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

- [x] 4.1 Entregar item (posse, quantidade, liberado e registro de estatísticas da arma) em vaga livre, mesmo tamanho, com backup e jogo fechado; também liberar item na loja do jogo (`cotw-core::equipment_editor`, comando `edit_equipment`)
- [x] 4.2 Quantidade de munição, consumíveis, iscas e abrigos (limites do app: 999 e 99). Veículos ficam de fora: `VehicleData` veio vazio no save de teste e o app não sabe os valores máximos
- [ ] 4.3 Montar loadout com checagem de compatibilidade (ADIADO: os conjuntos usam listas de tamanho variável no ADF; mudar o tamanho exige realocar o arquivo, o que já travou o jogo em outra edição. Só vale se o conjunto mantiver o mesmo número de itens)
- [x] 4.4 Dinheiro, até 10.000.000 (limite do app, não do jogo). Falta testar no jogo se conquistas ou o online reagem
- [x] 4.5 Bloqueio duplo (`premiumRequired` no serviço, cadeado na tela) e linhas novas na comparação da tela do Premium

## 5. Verificação

- [x] 5.1 Testes das escritas: unidades com bytes sintéticos (vaga livre, buraco, limites, mesmo tamanho) e de ponta a ponta com o perfil real (mesmo tamanho, poucos bytes mudam, conferidor aceita, leitura mostra a mudança, backup desfaz)
- [ ] 5.2 Teste no jogo: um item por categoria, quantidade, veículo, loadout e dinheiro
