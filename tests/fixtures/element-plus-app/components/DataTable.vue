<template>
  <div class="data-table">
    <el-table :data="tableData" :empty-text="$t('el.table.emptyText')">
      <el-table-column prop="name" :label="$t('el.table.confirmFilter')" />
      <el-table-column prop="date" :label="$t('el.datepicker.selectDate')" />
    </el-table>
    
    <el-pagination
      :total="total"
      :page-size="pageSize"
      :current-page="currentPage"
      @current-change="handlePageChange"
    >
      <template #prev>{{ $t('el.pagination.prev') }}</template>
      <template #next>{{ $t('el.pagination.next') }}</template>
    </el-pagination>
    
    <el-upload
      :on-remove="handleRemove"
      :on-preview="handlePreview"
    >
      <el-button>{{ $t('el.upload.continue') }}</el-button>
      <div slot="tip">{{ $t('el.upload.deleteTip') }}</div>
    </el-upload>
  </div>
</template>

<script>
export default {
  name: 'DataTable',
  data() {
    return {
      tableData: [],
      total: 0,
      pageSize: 10,
      currentPage: 1,
    };
  },
  methods: {
    handlePageChange(page) {
      this.currentPage = page;
    },
    handleRemove(file) {
      console.log('Remove file:', file);
    },
    handlePreview(file) {
      console.log('Preview file:', file);
    },
  },
};
</script>