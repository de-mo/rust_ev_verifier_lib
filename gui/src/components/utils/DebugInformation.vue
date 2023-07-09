<script setup>
import { useSharedApplication } from "../../composables/application"

const { hasDirectory, isNotStarted, isRunning, isFinished, status } = useSharedApplication()
</script>

<template>
    <div>
        <p>hasDirectory: {{ hasDirectory }}</p>
          <p>status: {{ status }}</p>
          <p>isNotStarted: {{ isNotStarted }}</p>
          <p>isRunning: {{ isRunning }}</p>
          <p>isFinished: {{ isFinished }}</p>
      </div>
</template>

<style scoped>
</style>
