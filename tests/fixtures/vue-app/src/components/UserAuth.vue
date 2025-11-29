<template>
  <div class="user-auth">
    <h2>{{ $t('user.messages.welcome') }}</h2>

    <button @click="handleLogin">
      {{ $t('user.labels.login') }}
    </button>

    <button @click="handleLogout">
      {{ $t('user.labels.logout') }}
    </button>

    <button @click="handleSignup">
      {{ $t('user.labels.signup') }}
    </button>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'UserAuth',

  methods: {
    handleLogin() {
      const loginLabel = this.$t('user.labels.login');
      console.log(`Logging in: ${loginLabel}`);
    },

    handleLogout() {
      const logoutLabel = this.$t('user.labels.logout');
      console.log(`Logging out: ${logoutLabel}`);
    },

    handleSignup() {
      const signupLabel = this.$t('user.labels.signup');
      console.log(`Signing up: ${signupLabel}`);
    }
  }
});
</script>

<style scoped>
.user-auth {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 20px;
}

button {
  padding: 10px 20px;
  cursor: pointer;
}
</style>
