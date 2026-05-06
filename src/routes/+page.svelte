<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, emit } from "@tauri-apps/api/event";

  import Clock from "$lib/components/Clock.svelte";
  import Battery from "$lib/components/Battery.svelte";
  import Volume from "$lib/components/Volume.svelte";
  import Network from "$lib/components/Network.svelte";
  import Icon, { iconifyer, exceptions } from "$lib/iconMap";

  import { anchor } from "$lib/actions/anchor";
  import { Keyframes } from "$lib/animations";
  import { startSystemPolling, getMonitors } from "$lib/stores/system";
  import type { TaskbarApp, DragOrigin } from "$lib/types";

  // ---------------------------------------------------------------------------
  // Estado
  // ---------------------------------------------------------------------------

  let apps = $state<TaskbarApp[]>([]);
  let errorMsg = $state("");
  let appsInterval: ReturnType<typeof setInterval>;
  let startMenuOpen = $state(false);
  let isMenuVisible = $state(false);
  let isMouseInTaskbar = $state(false);
  let isMouseInControlMenu = $state(false);
  let controlMenuTimer: ReturnType<typeof setTimeout> | null = null;

  // Drag & drop: objeto cohesivo en lugar de 4 variables sueltas
  let dragOrigin = $state<DragOrigin | null>(null);
  let dragSrcId = $state<string | null>(null);
  let dragOverId = $state<string | null>(null);
  let isDragging = $state(false);

  // ---------------------------------------------------------------------------
  // Control del menú de control
  // ---------------------------------------------------------------------------

  async function toggleControlMenu(force?: boolean) {
    if (force === false || (force === undefined && isMenuVisible)) {
      isMenuVisible = false;
      await invoke("manage_window", {
        label: "control_menu",
        action: { type: "hide" },
      });
      await emit("toggle-control-menu", false);
    } else {
      isMenuVisible = true;
      await invoke("manage_window", {
        label: "control_menu",
        action: { type: "show" },
      });
      await emit("toggle-control-menu", true);
    }
  }

  function checkCloseControlMenu() {
    if (controlMenuTimer) clearTimeout(controlMenuTimer);
    controlMenuTimer = setTimeout(() => {
      if (!isMouseInTaskbar && !isMouseInControlMenu && isMenuVisible) {
        toggleControlMenu(false);
      }
    }, 300);
  }

  // ---------------------------------------------------------------------------
  // Apps de la barra de tareas
  // ---------------------------------------------------------------------------

  async function fetchApps() {
    if (isDragging) return;
    try {
      const allApps = await invoke<TaskbarApp[]>("get_taskbar_apps");
      // Filtrar la propia aplicación para que no aparezca en la barra
      apps = allApps.filter(
        (app) => !app.exec_path.toLowerCase().includes("vibrant-dawn-taskbar"),
      );
    } catch (e: unknown) {
      errorMsg = String(e);
    }
  }

  async function interactApp(app: TaskbarApp) {
    try {
      await invoke("interact_app", { hwnd: app.hwnd, execPath: app.exec_path });
      fetchApps();
    } catch (e: unknown) {
      errorMsg = String(e);
    }
  }

  async function toggleStartMenu() {
    if (startMenuOpen) {
      startMenuOpen = false;
    } else {
      startMenuOpen = true;
      await invoke("open_start_menu");
    }
  }

  // ---------------------------------------------------------------------------
  // Drag & drop con pointer events
  // ---------------------------------------------------------------------------

  function onBtnPointerDown(e: PointerEvent, app: TaskbarApp) {
    if (e.button !== 0) return;
    dragOrigin = { id: app.id, app, startX: e.clientX, startY: e.clientY };
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }

  function onBtnPointerMove(e: PointerEvent, app: TaskbarApp) {
    if (!dragOrigin || dragOrigin.id !== app.id) return;
    const dx = Math.abs(e.clientX - dragOrigin.startX);
    const dy = Math.abs(e.clientY - dragOrigin.startY);
    if (!isDragging && (dx > 6 || dy > 6)) {
      isDragging = true;
      dragSrcId = app.id;
    }
  }

  function onBtnPointerEnter(app: TaskbarApp) {
    if (isDragging && dragSrcId !== app.id) {
      dragOverId = app.id;
    }
  }

  function onBtnPointerLeave(app: TaskbarApp) {
    if (isDragging && dragOverId === app.id) {
      dragOverId = null;
    }
  }

  function onBtnPointerUp(e: PointerEvent, app: TaskbarApp) {
    if (!dragOrigin || dragOrigin.id !== app.id) return;

    if (isDragging) {
      const targetId = dragOverId;
      isDragging = false;
      dragSrcId = null;
      dragOverId = null;
      dragOrigin = null;

      if (targetId && targetId !== app.id) {
        const newApps = [...apps];
        const srcIdx = newApps.findIndex((a) => a.id === app.id);
        const tgtIdx = newApps.findIndex((a) => a.id === targetId);
        if (srcIdx !== -1 && tgtIdx !== -1) {
          const [moved] = newApps.splice(srcIdx, 1);
          newApps.splice(tgtIdx, 0, moved);
          apps = newApps;
          invoke("reorder_apps", {
            orderedIds: newApps.map((a) => a.id),
          }).catch(() => {});
        }
      }
    } else {
      isDragging = false;
      dragSrcId = null;
      dragOverId = null;
      dragOrigin = null;
      interactApp(app);
    }
  }

  // ---------------------------------------------------------------------------
  // Ciclo de vida
  // ---------------------------------------------------------------------------

  onMount(() => {
    // Arrancar el store centralizado (una sola vez para toda la app)
    const stopPolling = startSystemPolling();

    async function setup() {
      fetchApps();
      appsInterval = setInterval(fetchApps, 2000);

      try {
        await invoke("init_taskbar_environment");

        const monitors = await getMonitors(); // usa caché del store
        const monitor = monitors[0];

        if (monitor) {
          const taskbarLogical = 48;
          const taskbarH = Math.round(taskbarLogical * monitor.scale_factor);

          await invoke("move_window", {
            x: monitor.x,
            y: monitor.y + monitor.height - taskbarH,
            w: monitor.width,
            h: taskbarH,
          }).catch((e) => console.error("[taskbar] move_window:", e));

          await invoke("set_window_to_bottom").catch((e) =>
            console.error("[taskbar] set_window_to_bottom:", e),
          );
        }
      } catch (e) {
        console.error("[taskbar] setup error:", e);
      }
    }

    setup();

    let unlistenHover: (() => void) | undefined;
    listen<boolean>("control-menu-hover", (event) => {
      isMouseInControlMenu = event.payload;
      if (!isMouseInControlMenu) checkCloseControlMenu();
    }).then((fn) => (unlistenHover = fn));

    return () => {
      clearInterval(appsInterval);
      stopPolling();
      unlistenHover?.();
    };
  });
</script>

<Keyframes />

<main
  class="taskbar"
  onmouseenter={() => {
    isMouseInTaskbar = true;
    if (controlMenuTimer) clearTimeout(controlMenuTimer);
  }}
  onmouseleave={() => {
    isMouseInTaskbar = false;
    checkCloseControlMenu();
  }}
>
  <section class="module win-menu">
    <button
      class="icon-btn {startMenuOpen ? 'active' : ''}"
      onclick={toggleStartMenu}
      title="Menú de Inicio">⊞</button
    >
  </section>

  <div class="divider"></div>

  <div
    class="module programs"
    role="toolbar"
    aria-label="Aplicaciones abiertas"
  >
    {#if errorMsg}
      <div style="color: red; font-size: 10px;">{errorMsg}</div>
    {/if}
    {#each apps as app (app.id)}
      <!-- 1. Limpiamos la extensión de forma robusta, incluyendo los .lnk -->
      {@const baseName =
        app.exec_path
          .split(/[\\/]/)
          .pop()
          ?.toLowerCase()
          .replace(/\.(exe|lnk|url|bat|cmd|com)$/, "") || ""}

      <!-- 2. Obtenemos el ID de Iconify y comprobamos si está explícitamente en el diccionario -->
      {@const icon = iconifyer(app.exec_path)}
      {@const isGuaranteed = !!exceptions[baseName]}

      <button
        class="program-btn {app.is_active ? 'active' : ''} {dragSrcId === app.id
          ? 'dragging'
          : ''} {dragOverId === app.id ? 'drag-over' : ''}"
        title={app.title}
        onpointerdown={(e) => onBtnPointerDown(e, app)}
        onpointermove={(e) => onBtnPointerMove(e, app)}
        onpointerup={(e) => onBtnPointerUp(e, app)}
        onpointerenter={() => onBtnPointerEnter(app)}
        onpointerleave={() => onBtnPointerLeave(app)}
      >
        <div class="icon-wrapper">
          {#if isGuaranteed}
            <!-- OPCIÓN A: Está mapeado en el diccionario, mostramos el minimalista -->
            <Icon {icon} />
          {:else if app.icon_base64}
            <!-- OPCIÓN B: No está mapeado, pero Rust nos dio el original. Lo mostramos -->
            <img
              src={`data:image/png;base64,${app.icon_base64}`}
              alt={app.title}
              draggable="false"
            />
          {:else}
            <!-- OPCIÓN C (Ultimísima): No está mapeado Y Rust falló al extraerlo. -->
            <Icon {icon} />
          {/if}
        </div>

        {#if app.is_active}
          <div class="active-indicator"></div>
        {/if}
      </button>
    {/each}
  </div>

  <div class="divider"></div>

  <section class="module media">
    <div class="media-ghost-slot" use:anchor={"media"}></div>
  </section>

  <div class="divider"></div>

  <section class="module utils">
    <button
      class="utils-container {isMenuVisible ? 'active' : ''}"
      onclick={() => toggleControlMenu()}
      use:anchor={"battery"}
      type="button"
      aria-label="Ajustes rápidos"
    >
      <Network />
      <Battery />
      <Volume />
    </button>
  </section>

  <div class="divider"></div>

  <section class="module others">
    <button class="clock-container" type="button">
      <Clock />
    </button>
  </section>
</main>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background-color: transparent;
    font-family: "Segoe UI", system-ui, sans-serif;
    color: #e0e0e0;
  }

  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  .taskbar {
    display: flex;
    align-items: center;
    background-color: rgba(16, 16, 19, 0.1);
    backdrop-filter: blur(12px);
    width: 100vw;
    height: 48px;
    position: fixed;
    bottom: 0;
    left: 0;
    padding: 0 15px;
    border: none;
    box-shadow: none;
  }

  .module {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .programs {
    flex-grow: 1;
    margin-left: 15px;
    gap: 4px;
    flex-shrink: 0;
    min-width: 0;
    justify-content: flex-start;
  }

  .divider {
    width: 1px;
    height: 20px;
    background-color: #3f3f46;
    margin: 0 15px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  button {
    background: transparent;
    border: none;
    color: inherit;
    font-size: 1.2rem;
    cursor: pointer;
    border-radius: 6px;
    padding: 5px 10px;
    transition: background 0.2s;
  }

  button:hover {
    background: #27272a;
  }

  .icon-btn.active {
    background: rgba(0, 120, 212, 0.3);
    color: #60a5fa;
  }

  .icon-btn.active:hover {
    background: rgba(0, 120, 212, 0.45);
  }

  .program-btn {
    position: relative;
    width: 40px;
    height: 40px;
    display: flex;
    justify-content: center;
    align-items: center;
    border-radius: 6px;
    background: transparent;
    border: none;
    cursor: pointer;
    transition:
      background 0.15s,
      opacity 0.15s,
      transform 0.15s;
    padding: 0;
    flex-shrink: 0;
    user-select: none;
    touch-action: none;
  }

  .program-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .program-btn.dragging {
    opacity: 0.35;
    cursor: grabbing;
    transform: scale(0.9);
  }

  .program-btn.drag-over {
    background: rgba(0, 120, 212, 0.25);
    outline: 2px solid rgba(0, 120, 212, 0.7);
    outline-offset: -2px;
    border-radius: 6px;
  }

  .icon-wrapper {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .icon-wrapper img,
  .icon-wrapper :global(svg) {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .active-indicator {
    position: absolute;
    bottom: 2px;
    width: 16px;
    height: 3px;
    background-color: #0078d4;
    border-radius: 2px;
    transition: width 0.2s;
  }

  .media {
    gap: 15px;
  }

  .utils-container {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 40px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 50px;
    padding: 0 12px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    user-select: none;
    color: inherit;
    font-family: inherit;
    outline: none;
  }

  .utils-container:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .utils-container:active {
    transform: scale(0.98);
  }

  .utils-container.active {
    background: #000;
    border-color: rgba(255, 255, 255, 0.15);
  }

  .utils-container.active:hover {
    background: #000;
    border-color: rgba(255, 255, 255, 0.25);
  }

  .clock-container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 40px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 50px;
    padding: 0 16px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    user-select: none;
    color: inherit;
    font-family: inherit;
    outline: none;
  }

  .clock-container:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .clock-container:active {
    transform: scale(0.98);
  }

  .media-ghost-slot {
    position: relative;
    width: 300px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
