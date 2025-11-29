// Invoice list component
class InvoiceList {
  private container: HTMLElement;

  constructor(containerId: string) {
    this.container = document.getElementById(containerId)!;
    this.render();
  }

  private render(): void {
    const header = document.createElement('h2');
    header.textContent = I18n.t('invoice.labels.add_new');
    this.container.appendChild(header);

    const addButton = this.createAddButton();
    this.container.appendChild(addButton);
  }

  private createAddButton(): HTMLButtonElement {
    const button = document.createElement('button');
    button.className = 'btn-primary';
    button.textContent = I18n.t('invoice.labels.add_new');
    button.onclick = () => this.onAddClick();
    return button;
  }

  private onAddClick(): void {
    console.log('Add button clicked');
    alert(I18n.t('invoice.labels.add_new'));
  }

  // Namespace caching pattern - cache the parent namespace
  private setupLabels(): void {
    const labels = I18n.t('invoice.labels');
    const addText = labels.add_new;
    const editText = labels.edit;
  }

  // Another namespace caching pattern - using relative key
  private setupWithRelativeKey(): void {
    // Simulate caching invoice namespace and using relative path
    const invoiceLabels = {} as any; // Would be I18n.t('invoice')
    const text = invoiceLabels.t('labels.add_new'); // Relative key without 'invoice'
  }
}

export { InvoiceList };
