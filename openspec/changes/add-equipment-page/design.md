## Context

Ver proposal.md e a nota de memória `equipment-data-in-save`. O perfil (`PlayerProfileData`) guarda `EquipmentUnlocked/Owned/Amount/BackPack` (1024 hashes cada), `WeaponData` (36 bytes por arma), `LoadoutInfo`, `VehicleData`, `Cash`. Os hashes são `hashlittle("equipment_<nome>")`. O catálogo do jogo está em `settings/hp_settings/equipment_data.bin` (RTPC, 991 itens, 10 grupos) e `settings/ammunition.bin`.

## Goals / Non-Goals

**Goals:** página de leitura completa e útil no plano grátis; escritas do mesmo tamanho no Premium; recomendador baseado em números do jogo.
**Non-Goals:** criar itens que o jogo não tem; editar multiplayer; distribuir ícones ou textos do jogo.

## Decisions

1. **Dados embutidos como fatos**: `equipment.json` guarda id, categoria, chave de nome, nome em inglês e números (peso, preço, nível, dano, munição compatível) tirados do jogo por um gerador, no mesmo modelo de `species_stats.json`. Nenhum arquivo ou texto copiado do jogo além do nome do item.
2. **Ícones em cache local**: na primeira abertura da página o app extrai os ícones da instalação do usuário (caminho `textures\ui\items\...`, achado por hash) para a pasta de dados do app e usa de lá. Sem instalação encontrada, a página mostra ícones genéricos por categoria.
3. **Decodificação por etapas**: primeiro nomes e categorias (já feito), depois peso, preço e nível (campos sem rótulo, comparando com valores conhecidos do jogo), depois dano e compatibilidade arma→munição (nós filhos). Cada etapa só entra no app depois de conferida contra pelo menos três itens conhecidos.
4. **Escrita do mesmo tamanho**: dar um item é preencher uma vaga livre dos arrays de 1024; nunca crescer o arquivo. Se não houver vaga, o app recusa e explica.
5. **Bloqueio duplo**: igual ao editor de animais; a tela mostra o cadeado e o serviço (`premiumRequired`) decide.
6. **Dinheiro**: entra no Premium como ajuste simples, com a mesma regra de backup e jogo fechado. Antes de liberar, testar se o jogo ou a Steam reagem a valores muito altos.
7. **Recomendador**: calcula pontuação de adequação por espécie com classe de arma, dano e distância; mostra o porquê de cada sugestão. Só entra depois da etapa 3 da decodificação.

## Risks / Trade-offs

- [Números decodificados errados] → conferir com valores conhecidos e marcar na página o que for aproximado.
- [Item entregue que o jogo rejeita] → teste em jogo com um item por categoria antes de liberar; backup sempre.
- [Dinheiro alto afetar conquistas ou o online] → testar e avisar; se houver risco, deixar de fora.
- [Ícones não encontrados em instalação diferente] → fallback por categoria.

## Open Questions

- Nomes em português e espanhol: o jogo só tem inglês nesse arquivo; verificar os textos de localização.
- Como o jogo trata `Cash` e itens entregues em relação a conquistas e ao nível do jogador.
