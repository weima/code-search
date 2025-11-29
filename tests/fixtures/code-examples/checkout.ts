import { processPayment, calculateTotal, ERROR_MESSAGES } from './utils';

interface CartItem {
  id: string;
  price: number;
  quantity: number;
}

export class CheckoutService {
  private userId: string;

  constructor(userId: string) {
    this.userId = userId;
  }

  async checkout(items: CartItem[]): Promise<boolean> {
    try {
      const itemPrices = items.map(item => item.price * item.quantity);
      const total = calculateTotal(itemPrices);

      console.log(`Checkout for user ${this.userId}, total: $${total}`);

      if (total === 0) {
        throw new Error(ERROR_MESSAGES.INVALID_AMOUNT);
      }

      const paymentSuccess = processPayment(total, this.userId);

      if (paymentSuccess) {
        this.sendConfirmationEmail();
        this.clearCart();
      }

      return paymentSuccess;
    } catch (error) {
      console.error('Checkout failed:', error);
      return false;
    }
  }

  private sendConfirmationEmail(): void {
    console.log(`Sending confirmation email to user ${this.userId}`);
  }

  private clearCart(): void {
    console.log('Cart cleared');
  }
}
