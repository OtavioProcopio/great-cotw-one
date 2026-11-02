## ADDED Requirements

### Requirement: Chances reais de pelagem
O app SHALL mostrar, por espécie, cada pelagem com a chance para macho e fêmea calculada a partir dos pesos do arquivo do jogo, a raridade e se é pelagem de Great One.

#### Scenario: Pelagem rara
- **WHEN** o usuário abre o Javali
- **THEN** pelagens como Melânico e Albino aparecem com a chance em porcentagem e marcadas como raras

### Requirement: Situação do save por reserva
O app SHALL mostrar, por reserva e espécie, quantos animais existem, quantos machos e fêmeas, quantos estão na faixa de diamante e quantos Great Ones.

#### Scenario: Espécie sem diamante
- **WHEN** uma espécie da reserva não tem nenhum animal na faixa de diamante
- **THEN** o app mostra 0 diamantes para essa espécie

### Requirement: Limites conhecidos
O app SHALL avisar que o jogo não guarda qual pelagem cada animal tem hoje (só uma semente), em vez de inventar a pelagem.

#### Scenario: Animal com semente desconhecida
- **WHEN** a semente de pelagem de um animal não está na tabela conhecida
- **THEN** o app mostra "pelagem sorteada pelo jogo" e não um nome de pelagem
