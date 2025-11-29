import { CheckoutService } from './checkout';
import { ERROR_MESSAGES } from './utils';

export async function handleCheckoutRequest(userId: string, cartItems: any[]): Promise<void> {
  if (!userId) {
    throw new Error(ERROR_MESSAGES.USER_NOT_FOUND);
  }

  const checkoutService = new CheckoutService(userId);
  const success = await checkoutService.checkout(cartItems);

  if (!success) {
    throw new Error('Checkout process failed');
  }
}

export function getUserId(request: any): string {
  const userId = request.headers['user-id'];

  if (!userId) {
    console.error(ERROR_MESSAGES.USER_NOT_FOUND);
    throw new Error(ERROR_MESSAGES.USER_NOT_FOUND);
  }

  return userId;
}
