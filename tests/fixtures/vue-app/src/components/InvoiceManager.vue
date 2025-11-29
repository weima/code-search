<template>
  <div class="invoice-manager">
    <h1>{{ $t('invoice.labels.add_new') }}</h1>

    <button @click="handleAddNew">
      {{ $t('invoice.labels.add_new') }}
    </button>

    <div class="invoice-list">
      <div v-for="invoice in invoices" :key="invoice.id" class="invoice-item">
        <span>{{ invoice.description }}</span>
        <button @click="handleEdit(invoice.id)">
          {{ $t('invoice.labels.edit') }}
        </button>
        <button @click="handleDelete(invoice.id)">
          {{ $t('invoice.labels.delete') }}
        </button>
      </div>
    </div>

    <div v-if="message" class="message">
      {{ message }}
    </div>

    <!-- Example using this.$t in template -->
    <p>{{ this.$t('invoice.messages.created') }}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';

interface Invoice {
  id: number;
  amount: number;
  description: string;
}

export default defineComponent({
  name: 'InvoiceManager',

  setup() {
    const invoices = ref<Invoice[]>([]);
    const message = ref<string>('');

    return {
      invoices,
      message
    };
  },

  methods: {
    handleAddNew() {
      console.log('Adding new invoice...');
      const newInvoice: Invoice = {
        id: Date.now(),
        amount: 0,
        description: 'New invoice'
      };
      this.invoices.push(newInvoice);
      this.message = this.$t('invoice.messages.created');
    },

    handleEdit(id: number) {
      console.log(`Editing invoice ${id}...`);
      this.message = this.$t('invoice.messages.updated');
    },

    handleDelete(id: number) {
      this.invoices = this.invoices.filter(inv => inv.id !== id);
      this.message = this.$t('invoice.messages.deleted');
    },

    handleError() {
      this.message = this.$t('invoice.errors.not_found');
    },

    handleSave() {
      // Using $t in method
      const saveLabel = this.$t('invoice.labels.save');
      console.log(saveLabel);
    }
  }
});
</script>

<style scoped>
.invoice-manager {
  padding: 20px;
}

.invoice-item {
  display: flex;
  gap: 10px;
  margin: 10px 0;
}

.message {
  padding: 10px;
  background-color: #f0f0f0;
  margin-top: 10px;
}
</style>
