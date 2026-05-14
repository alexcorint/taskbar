<script lang="ts">
  import MediaCard from "$lib/components/MediaCard.svelte";
  import { isExpanded, setIsExpanded } from "$lib/stores/media";

  let leaveTimer: ReturnType<typeof setTimeout> | null = null;

  function handleMainEnter() {
    if (leaveTimer) {
      clearTimeout(leaveTimer);
      leaveTimer = null;
    }
  }

  function handleLeave() {
    leaveTimer = setTimeout(() => {
      setIsExpanded(false);
    }, 150);
  }
</script>

<main on:mouseenter={handleMainEnter} on:mouseleave={handleLeave}>
  <MediaCard />
</main>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background-color: transparent;
    font-family: "Segoe UI", system-ui, sans-serif;
  }

  main {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: center;
  }
</style>
