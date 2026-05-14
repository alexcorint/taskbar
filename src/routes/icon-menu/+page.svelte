<script lang="ts">
  import { onMount } from "svelte";
  import { listen, emit } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "$lib/iconMap";
  import type { TaskbarApp } from "$lib/types";

  let app = $state<TaskbarApp | null>(null);
  let isVisible = $state(false);
  let isMouseInMenu = false;
  let isMouseInIcon = false;
  let closeTimer: ReturnType<typeof setTimeout> | null = null;
  
  const appWindow = getCurrentWindow();

  async function closeMenu() {
    isVisible = false;
    setTimeout(async () => {
      if (!isVisible) {
        await appWindow.hide();
      }
    }, 200); // Duración de la animación slideDown
  }

  function checkClose() {
    if (closeTimer) clearTimeout(closeTimer);
    closeTimer = setTimeout(() => {
      if (!isMouseInMenu && !isMouseInIcon) {
        closeMenu();
      }
    }, 300);
  }

  onMount(() => {
    const unlistenData = listen<TaskbarApp>("icon-menu-data", (event) => {
      app = event.payload;
      isVisible = true;
    });

    const unlistenIconHover = listen<boolean>("icon-hover", (event) => {
      isMouseInIcon = event.payload;
      if (!isMouseInIcon) checkClose();
    });

    const unlistenBlur = appWindow.onFocusChanged((event) => {
      if (!event.payload) {
        closeMenu();
      }
    });

    return () => {
      unlistenData.then(fn => fn());
      unlistenIconHover.then(fn => fn());
      unlistenBlur.then(fn => fn());
    };
  });

  function handleMouseEnter() {
    isMouseInMenu = true;
    if (closeTimer) clearTimeout(closeTimer);
    emit("icon-menu-hover", true);
  }

  function handleMouseLeave() {
    isMouseInMenu = false;
    emit("icon-menu-hover", false);
    checkClose();
  }

  async function handlePin() {
    if (!app) return;
    if (app.is_pinned) {
      await invoke("unpin_app", { id: app.id });
    } else {
      await invoke("pin_app", {
        app: { id: app.id, name: app.title, exec_path: app.exec_path }
      });
    }
    closeMenu();
  }

  async function handleNewWindow() {
    if (!app) return;
    await invoke("interact_app", { hwnd: 0, execPath: app.exec_path });
    closeMenu();
  }

  async function handleOpenExplorer() {
    if (!app) return;
    await invoke("interact_app", { hwnd: 0, execPath: `explorer.exe /select,"${app.exec_path}"` });
    closeMenu();
  }
</script>

<main 
  class="menu-container {isVisible ? 'visible' : 'closing'}"
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
>
  {#if app}
    <button class="menu-item" onclick={handlePin} title={app.is_pinned ? "Desanclar" : "Anclar"}>
      <Icon icon={app.is_pinned ? "fluent:pin-off-24-filled" : "fluent:pin-24-filled"} />
    </button>
    <button class="menu-item" onclick={handleNewWindow} title="Nueva ventana">
      <Icon icon="fluent:add-24-filled" />
    </button>
    <button class="menu-item" onclick={handleOpenExplorer} title="Abrir ubicación">
      <Icon icon="fluent:folder-open-24-filled" />
    </button>
    <button class="menu-item" title="Más opciones">
      <Icon icon="fluent:more-horizontal-24-filled" />
    </button>
  {/if}
</main>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
    font-family: "Segoe UI", system-ui, sans-serif;
  }

  .menu-container {
    display: flex;
    align-items: center;
    background: rgba(20, 20, 25, 0.7);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 8px;
    gap: 4px;
    width: fit-content;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    opacity: 0;
    transform: translateY(10px);
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .menu-container.visible {
    opacity: 1;
    transform: translateY(0);
  }

  .menu-container.closing {
    opacity: 0;
    transform: translateY(10px);
    pointer-events: none;
  }

  .menu-item {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #e0e0e0;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .menu-item:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
    transform: translateY(-2px);
  }

  .menu-item:active {
    transform: translateY(0) scale(0.95);
  }

  .menu-item :global(svg) {
    width: 20px;
    height: 20px;
  }
</style>
