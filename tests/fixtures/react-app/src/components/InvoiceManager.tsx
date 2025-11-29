import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

interface Invoice {
  id: number;
  amount: number;
  description: string;
}

const InvoiceManager: React.FC = () => {
  const { t, i18n } = useTranslation();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [message, setMessage] = useState<string>('');

  const handleAddNew = () => {
    console.log('Adding new invoice...');
    const newInvoice: Invoice = {
      id: Date.now(),
      amount: 0,
      description: 'New invoice'
    };
    setInvoices([...invoices, newInvoice]);
    setMessage(t('invoice.messages.created'));
  };

  const handleEdit = (id: number) => {
    console.log(`Editing invoice ${id}...`);
    setMessage(i18n.t('invoice.messages.updated'));
  };

  const handleDelete = (id: number) => {
    setInvoices(invoices.filter(inv => inv.id !== id));
    setMessage(t('invoice.messages.deleted'));
  };

  const handleError = () => {
    setMessage(i18n.t('invoice.errors.not_found'));
  };

  return (
    <div className="invoice-manager">
      <h1>{t('invoice.labels.add_new')}</h1>

      <button onClick={handleAddNew}>
        {t('invoice.labels.add_new')}
      </button>

      <div className="invoice-list">
        {invoices.map(invoice => (
          <div key={invoice.id} className="invoice-item">
            <span>{invoice.description}</span>
            <button onClick={() => handleEdit(invoice.id)}>
              {t('invoice.labels.edit')}
            </button>
            <button onClick={() => handleDelete(invoice.id)}>
              {t('invoice.labels.delete')}
            </button>
          </div>
        ))}
      </div>

      {message && (
        <div className="message">
          {message}
        </div>
      )}
    </div>
  );
};

// User component using useTranslation hook
const UserAuth: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="user-auth">
      <button>{t('user.labels.login')}</button>
      <button>{t('user.labels.logout')}</button>
      <p>{t('user.messages.welcome')}</p>
    </div>
  );
};

export { InvoiceManager, UserAuth };
