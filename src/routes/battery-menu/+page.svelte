<script lang="ts">
  import { listen, emit } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ControlMenu from "$lib/components/ControlMenu.svelte";

  let isMenuVisible = false;

  onMount(() => {
    let unlistenToggle: (() => void) | null = null;

    listen<boolean>("toggle-battery-menu", (event) => {
      if (event.payload) {
        isMenuVisible = true;
        emit("request-sync");
      } else {
        isMenuVisible = false;
        setTimeout(() => {
          if (!isMenuVisible) {
            invoke("manage_window", {
              label: "battery_menu",
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
    transform: scale(0.95);
    transition:
      opacity 0.25s cubic-bezier(0.215, 0.61, 0.355, 1),
      transform 0.25s cubic-bezier(0.215, 0.61, 0.355, 1);
    pointer-events: none;
  }

  .menu-wrapper.visible {
    opacity: 1;
    transform: scale(1);
    pointer-events: auto;
  }
</style>
