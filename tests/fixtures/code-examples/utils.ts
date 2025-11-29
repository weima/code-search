/**
 * Utility functions for data processing
 */

export function processPayment(amount: number, userId: string): boolean {
  console.log(`Processing payment of ${amount} for user ${userId}`);

  if (!validateAmount(amount)) {
    throw new Error('Invalid payment amount');
  }

  const result = chargeCard(userId, amount);
  logTransaction(userId, amount, result);

  return result;
}

function validateAmount(amount: number): boolean {
  if (amount <= 0) {
    console.error('Amount must be positive');
    return false;
  }

  if (amount > 10000) {
    console.error('Amount exceeds limit');
    return false;
  }

  return true;
}

function chargeCard(userId: string, amount: number): boolean {
  // Simulate card charging
  console.log(`Charging card for user ${userId}: $${amount}`);
  return true;
}

function logTransaction(userId: string, amount: number, success: boolean): void {
  const status = success ? 'SUCCESS' : 'FAILED';
  console.log(`Transaction ${status}: User ${userId}, Amount ${amount}`);
}

export function calculateTotal(items: number[]): number {
  return items.reduce((sum, item) => sum + item, 0);
}

export const ERROR_MESSAGES = {
  INVALID_AMOUNT: 'Invalid payment amount',
  CARD_DECLINED: 'Card was declined',
  USER_NOT_FOUND: 'User not found',
};
