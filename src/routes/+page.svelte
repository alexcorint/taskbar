<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";

  import Clock from "$lib/components/Clock.svelte";
  import Media from "$lib/components/Media.svelte";
  import Battery from "$lib/components/Battery.svelte";
  import Volume from "$lib/components/Volume.svelte";
  import { listen, emit } from "@tauri-apps/api/event";
  import { anchor } from "$lib/actions/anchor";

  let apps: Array<{
    id: string;
    title: string;
    icon_base64: string;
    is_active: boolean;
    is_pinned: boolean;
    hwnd: number;
    exec_path: string;
  }> = $state([]);

  let errorMsg = $state("");
  let appsInterval: any;
  let startMenuOpen = $state(false);
  let isMenuVisible = $state(false);
  let isMouseInTaskbar = $state(false);
  let isMouseInControlMenu = $state(false);
  let controlMenuTimer: ReturnType<typeof setTimeout> | null = null;

  async function toggleControlMenu(force?: boolean) {
    if (force === false || (force === undefined && isMenuVisible)) {
      isMenuVisible = false;
      await invoke("manage_window", {
        label: "battery_menu",
        action: { type: "hide" },
      });
      await emit("toggle-battery-menu", false);
    } else {
      isMenuVisible = true;
      await invoke("manage_window", {
        label: "battery_menu",
        action: { type: "show" },
      });
      await emit("toggle-battery-menu", true);
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

  // --- Reordenación con pointer events (más fiable que HTML5 drag en WebView2/Tauri) ---
  let dragSrcId = $state<string | null>(null); // id del icono que se está arrastrando
  let dragOverId = $state<string | null>(null); // id del icono sobre el que está el cursor
  let isDragging = $state(false); // arrastrando activamente (pasado el threshold)

  let _pointerDownId: string | null = null; // id al hacer mousedown (no reactivo)
  let _pointerDownApp: any = null; // app al hacer mousedown
  let _startX = 0;
  let _startY = 0;

  async function fetchApps() {
    if (isDragging) return;
    try {
      const result = await invoke<typeof apps>("get_taskbar_apps");
      apps = result;
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  async function interactApp(app: any) {
    try {
      await invoke("interact_app", { hwnd: app.hwnd, execPath: app.exec_path });
      fetchApps();
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  async function toggleStartMenu() {
    if (startMenuOpen) {
      // El clic en nuestro botón ya cerró el menú (por cambio de foco).
      // Solo actualizamos el estado, sin enviar la tecla Win (evita re-abrirlo).
      startMenuOpen = false;
    } else {
      // El menú estaba cerrado → lo abrimos con la tecla Win.
      startMenuOpen = true;
      await invoke("open_start_menu");
    }
  }

  // Pointer down: registrar punto de inicio
  function onBtnPointerDown(e: PointerEvent, app: any) {
    if (e.button !== 0) return; // solo botón izquierdo
    _pointerDownId = app.id;
    _pointerDownApp = app;
    _startX = e.clientX;
    _startY = e.clientY;
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }

  // Pointer move: iniciar drag si supera el threshold
  function onBtnPointerMove(e: PointerEvent, app: any) {
    if (_pointerDownId !== app.id) return;
    const dx = Math.abs(e.clientX - _startX);
    const dy = Math.abs(e.clientY - _startY);
    if (!isDragging && (dx > 6 || dy > 6)) {
      isDragging = true;
      dragSrcId = app.id;
    }
  }

  // Pointer enter en un botón mientras se arrastra: marcar como destino
  function onBtnPointerEnter(app: any) {
    if (isDragging && dragSrcId !== app.id) {
      dragOverId = app.id;
    }
  }

  // Pointer leave de un botón mientras se arrastra
  function onBtnPointerLeave(app: any) {
    if (isDragging && dragOverId === app.id) {
      dragOverId = null;
    }
  }

  // Pointer up: soltar — click o drop según si hubo movimiento
  function onBtnPointerUp(e: PointerEvent, app: any) {
    if (_pointerDownId !== app.id) return;

    if (isDragging) {
      // Fue un drag → reordenar si hay destino
      const targetId = dragOverId;
      isDragging = false;
      dragSrcId = null;
      dragOverId = null;
      _pointerDownId = null;
      _pointerDownApp = null;

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
      // Fue un click → lanzar/enfocar la app
      isDragging = false;
      dragSrcId = null;
      dragOverId = null;
      _pointerDownId = null;
      _pointerDownApp = null;
      interactApp(app);
    }
  }

  onMount(() => {
    async function setup() {
      fetchApps();
      appsInterval = setInterval(fetchApps, 2000);

      try {
        await invoke("init_taskbar_environment");

        const appWindow = getCurrentWindow();

        // Obtener monitores desde Rust (evita problemas de caché con la API JS de Tauri)
        const monitors = await invoke<
          Array<{
            x: number;
            y: number;
            width: number;
            height: number;
            scale_factor: number;
          }>
        >("get_monitor_rects");

        // Usar el segundo monitor (índice 1) para tests; si solo hay uno, usar el primero
        const monitor = monitors[0];

        if (monitor) {
          // La altura lógica de la barra es 48px; convertirla a píxeles físicos según el DPI del monitor
          const taskbarLogical = 48;
          const taskbarH = Math.round(taskbarLogical * monitor.scale_factor);

          const targetX = monitor.x;
          const targetY = monitor.y + monitor.height - taskbarH;
          const targetW = monitor.width;
          const targetH = taskbarH;

          try {
            await invoke("move_window", {
              x: targetX,
              y: targetY,
              w: targetW,
              h: targetH,
            });
          } catch (e) {
            console.error("[taskbar] move_window error:", e);
          }

          try {
            await invoke("set_window_to_bottom");
          } catch (e) {
            console.error("[taskbar] set_window_to_bottom error:", e);
          }
        }
      } catch (e) {
        console.error("[taskbar] onMount setup error:", e);
      }
    }

    setup();

    let unlistenHover: any;
    listen<boolean>("battery-menu-hover", (event) => {
      isMouseInControlMenu = event.payload;
      if (!isMouseInControlMenu) checkCloseControlMenu();
    }).then((fn) => (unlistenHover = fn));

    return () => {
      if (appsInterval) clearInterval(appsInterval);
      if (unlistenHover) unlistenHover();
    };
  });

  onDestroy(() => {
    // handled in onMount return
  });
</script>

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
        {#if app.icon_base64}
          <img
            src={`data:image/png;base64,${app.icon_base64}`}
            alt={app.title}
            draggable="false"
          />
        {:else}
          <div class="generic-icon">📦</div>
        {/if}
        {#if app.is_active}
          <div class="active-indicator"></div>
        {/if}
      </button>
    {/each}
  </div>

  <div class="divider"></div>

  <section class="module media">
    <Media />
  </section>

  <div class="divider"></div>

  <section class="module utils">
    <button
      class="utils-container"
      onclick={() => toggleControlMenu()}
      use:anchor={"battery"}
      type="button"
      aria-label="Ajustes rápidos"
    >
      <Battery />
      <Volume />
    </button>
  </section>

  <div class="divider"></div>

  <section class="module others">
    <div class="clock">
      <Clock />
    </div>
  </section>
</main>

<style>
  /* ESTILOS GLOBALES PARA LA VENTANA TRANSPARENTE */
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

  /* CONTENEDOR PRINCIPAL FLEXBOX */
  .taskbar {
    display: flex;
    align-items: center;
    background-color: rgba(16, 16, 19, 0.1);
    backdrop-filter: blur(12px); /* Efecto Glassmorphism */
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
    /* Alinear desde la izquierda y no contraerse */
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

  /* ESTILOS DE APLICACIONES */
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
    touch-action: none; /* necesario para pointer capture */
  }

  .program-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  /* App siendo arrastrada: semitransparente */
  .program-btn.dragging {
    opacity: 0.35;
    cursor: grabbing;
    transform: scale(0.9);
  }

  /* Destino del drop: resaltar con borde azul */
  .program-btn.drag-over {
    background: rgba(0, 120, 212, 0.25);
    outline: 2px solid rgba(0, 120, 212, 0.7);
    outline-offset: -2px;
    border-radius: 6px;
  }

  .program-btn img {
    width: 24px;
    height: 24px;
    object-fit: contain;
    pointer-events: none;
  }

  .generic-icon {
    font-size: 1.2rem;
    pointer-events: none;
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

  .program-btn.active .active-indicator {
    width: 16px;
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
</style>
