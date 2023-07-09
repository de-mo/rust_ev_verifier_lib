<script setup>
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import HeaderFrame from "./components/HeaderFrame.vue";
import InformationFrame from "./components/InformationFrame.vue";
import VerificationList from "./components/VerificationList.vue";
import DebugInformation from "./components/utils/DebugInformation.vue";
import { useSharedApplication } from "./composables/application"

const { hasDirectory } = useSharedApplication()
</script>

<template>
  <div class="container">
    <div style="text-align: center"><h1>Welcome to the Verifier</h1></div>
      <DebugInformation></DebugInformation>
      <div><HeaderFrame/></div>
      <div><InformationFrame/></div>
      <div v-if="hasDirectory"><VerificationList/></div>
    </div>
</template>

<style scoped>
</style>
