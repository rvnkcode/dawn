<script lang="ts">
  let { title } = $props();
</script>

<header>
  <h1>{title}</h1>
</header>

<style>
  header {
    margin-top: 0.25rem;
  }

  h1 {
    font-size: 1.75rem;
    font-weight: bold;
  }
</style>
