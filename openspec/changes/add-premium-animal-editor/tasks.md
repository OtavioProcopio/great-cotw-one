## 1. Edição

- [x] 1.1 Localizar e editar no mesmo tamanho os campos do animal (sexo, peso, pontuação, Great One), reversível byte a byte e verificado nos saves reais
- [x] 1.2 Regras de equilíbrio por espécie e sexo, tiradas dos próprios animais do save (faixa com 10% de folga e valores típicos)
- [ ] 1.2b Atalhos de diamante e Great One por espécie (precisam das faixas de diamante do jogo)
- [x] 1.3 Painel de edição de um animal no rastreador do mapa, com trocar de sexo e faixas aceitas
- [x] 1.3b Ajustar o grupo todo de uma vez (valores típicos ou os maiores do jogo), dentro das faixas aceitas
- [ ] 1.3c Atalho "1 diamante por grupo" (precisa das faixas de diamante do jogo)
- [x] 1.4 Ligação mapa ↔ editor: botão em cada animal do grupo rastreado (cadeado para quem é gratuito) e recarga do mapa depois de gravar

## 2. Pelagens (só de animais que já existem)

- [ ] 2.1 Lista de pelagens por espécie e sexo, com chance e raridade (compartilhada com o guia de pelagens, `add-career-guide` 2.1)
- [ ] 2.2 Escolher a pelagem de um animal no editor, gravando só a semente
- [ ] 2.3 Testes: mesmo tamanho de arquivo, mesma quantidade de animais por grupo, demais bytes idênticos

## 3. Premium

- [x] 3.1 Bloqueio no serviço do app: `unlock_zones` recusa sem licença (`premiumRequired`) e a interface mostra o que é Premium (selo, faixa e botão)
- [x] 3.2 Verificação offline da chave (assinatura Ed25519, plano e vencimento), guardar a chave e ativação na tela do Premium
- [x] 3.5 Ferramenta `gco-license` (gerar o par de chaves fora do repositório, emitir chave mensal ou anual, inspecionar)
- [ ] 3.6 Canal de venda: página de compra, webhook de pagamento, emissão e envio da chave por e-mail
- [x] 3.7 Aviso de renovação 30 dias antes do vencimento (tela inicial e tela do Premium)
- [x] 3.8 Tela do Premium: vantagens, comparação com o gratuito, planos (R$ 15/mês e R$ 75/ano) e economia do anual
- [x] 3.3 Modo leitura do editor para usuários gratuitos (botão com cadeado que leva ao Premium)
- [x] 3.4 Tratamento de chave inválida, adulterada ou vencida (volta ao modo gratuito)

## 4. Verificação

- [ ] 4.1 Teste no jogo: peso, pontuação, sexo e Great One editados aparecem ao caçar
- [ ] 4.2 Teste no jogo: pelagem escolhida aparece no animal
