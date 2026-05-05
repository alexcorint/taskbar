<script lang="ts">
  import { listen, emit } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ControlMenu from "$lib/components/ControlMenu.svelte";

  let isMenuVisible = false;

  onMount(() => {
    let unlistenToggle: (() => void) | null = null;

    listen<boolean>("toggle-control-menu", (event) => {
      if (event.payload) {
        isMenuVisible = true;
        emit("request-sync");
      } else {
        isMenuVisible = false;
        setTimeout(() => {
          if (!isMenuVisible) {
            invoke("manage_window", {
              label: "control_menu",
              action: { type: "hide" },
            });
          }
        }, 300);
      }
    }).then((fn) => (unlistenToggle = fn));

    emit("request-sync");

    return () => {
      if (unlistenToggle) unlistenToggle();
    };
  });
</script>

<main>
  <div class="menu-wrapper" class:visible={isMenuVisible}>
    <ControlMenu isVisible={isMenuVisible} />
  </div>
</main>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent !important;
    font-family: "Segoe UI", system-ui, sans-serif;
    color: #e0e0e0;
  }

  main {
    width: 100vw;
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: flex-end;
    padding-bottom: 20px;
  }

  .menu-wrapper {
    opacity: 0;
    transform: translateY(10px) scale(0.95);
    transition:
      opacity 0.2s ease,
      transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    pointer-events: none;
  }

  .menu-wrapper.visible {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
  }
</style>
