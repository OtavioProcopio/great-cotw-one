/** Preços do Premium, em reais. */
export const PRICES = { monthly: 15, yearly: 75 } as const;

/** Endereço da loja (vazio até a loja abrir; a tela explica isso ao jogador). */
export const STORE_URL = '';

/** Quanto o plano anual economiza sobre 12 meses do mensal, em reais. */
export function yearlySaving(): number {
  return PRICES.monthly * 12 - PRICES.yearly;
}

/** O desconto do plano anual sobre 12 meses do mensal, em porcentagem inteira. */
export function yearlyDiscountPercent(): number {
  return Math.round((yearlySaving() / (PRICES.monthly * 12)) * 100);
}

/** Quanto o plano anual custa por mês. */
export function yearlyPerMonth(): number {
  return PRICES.yearly / 12;
}
