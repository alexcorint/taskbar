<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, emit } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { fade } from "svelte/transition";

  const slideTransition = (node: Element, { direction = "right" } = {}) => {
    const offsetX = direction === "left" ? 100 : -100;
    return {
      duration: 300,
      easing: (t: number) => {
        return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
      },
      css: (t: number) => {
        return `
          opacity: ${t};
          transform: translateX(${offsetX * (1 - t)}px);
        `;
      },
    };
  };

  // --- LÓGICA DE MEDIOS ---
  let data = {
    title: "Now listening...",
    artist: "",
    album: "",
    app_id: "Generic",
    thumbnail_base64: "",
    is_playing: false,
    position_ms: 0,
    duration_ms: 0,
  };
  let isPlaying = false;
  let displayPos = 0;
  let lastBackendPos = 0;
  let lastBackendTime = 0;
  let interval: ReturnType<typeof setInterval>;
  let posInterval: ReturnType<typeof setInterval>;

  let slideDirection: "left" | "right" = "right";

  async function fetchMetadata() {
    try {
      const res = await invoke<string>("get_current_media");
      const parsed = JSON.parse(res);
      data = parsed;
      isPlaying = data.is_playing;
      lastBackendPos = data.position_ms;
      displayPos = data.position_ms;
      lastBackendTime = Date.now();
    } catch (e) {
      console.error("Error fetching media metadata:", e);
    }
  }

  function tickPosition() {
    if (!isPlaying) return;
    const now = Date.now();
    const diff = now - lastBackendTime;
    displayPos = lastBackendPos + diff;
    if (data.duration_ms > 0 && displayPos > data.duration_ms) {
      displayPos = data.duration_ms;
    }
  }

  function formatTime(ms: number) {
    if (!ms || ms < 0) return "0:00";
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  async function togglePlay() {
    isPlaying = !isPlaying;
    await invoke("media_control", { action: "playpause" });
  }

  async function nextTrack() {
    slideDirection = "left";
    await invoke("media_control", { action: "next" });
  }

  async function prevTrack() {
    slideDirection = "right";
    await invoke("media_control", { action: "prev" });
  }

  function getAppIcon(media: any) {
    const appId = (media.app_id || "").toLowerCase();
    if (appId.includes("spotify")) {
      return `<svg viewBox="0 0 24 24" width="18" height="18" fill="white"><path d="M12 0C5.372 0 0 5.372 0 12s5.372 12 12 12 12-5.372 12-12S18.628 0 12 0zm5.503 17.31c-.218.358-.684.474-1.042.256-2.848-1.74-6.432-2.133-10.655-1.168-.41.094-.823-.162-.917-.572-.094-.41.162-.823.572-.917 4.622-1.057 8.583-.615 11.786 1.342.358.218.474.684.256 1.042zm1.47-3.253c-.275.446-.86.592-1.306.317-3.258-2-8.225-2.583-12.08-1.413-.502.152-1.03-.133-1.182-.635-.152-.502.133-1.03.635-1.182 4.41-1.338 9.887-.684 13.616 1.61.446.275.592.86.317 1.306zm.126-3.41c-3.905-2.32-10.347-2.535-14.127-1.388-.598.182-1.232-.165-1.414-.763-.182-.598.165-1.232.763-1.414 4.335-1.316 11.45-1.066 16.002 1.637.538.32.715 1.015.395 1.553-.32.538-1.015.715-1.553.395z"/></svg>`;
    }
    if (appId.includes("chrome")) {
      return `<svg viewBox="0 0 24 24" width="18" height="18" fill="white"><path d="M12 0C8.21 0 4.89 1.74 2.71 4.48l3.96 6.84c.15-.31.33-.6.55-.87 1.03-1.26 2.58-2.07 4.31-2.07h10.15C20.35 3.52 16.51 0 12 0zM5.33 13.5c.34 2.1 1.77 3.86 3.65 4.81l-5.07 8.79C1.51 24.36 0 21.36 0 18c0-1.57.3-3.07.84-4.5h4.49zM12 24c4.39 0 8.16-2.36 10.16-5.88l-5.08-8.8c.22.27.4.56.55.87.56 1.15.87 2.44.87 3.81 0 3.1-1.63 5.81-4.08 7.33L12 24zM12 15.5c-1.93 0-3.5-1.57-3.5-3.5s1.57-3.5 3.5-3.5 3.5 1.57 3.5 3.5-1.57 3.5-3.5 3.5z"/></svg>`;
    }
    if (appId.includes("msedge")) {
      return `<svg viewBox="0 0 24 24" width="18" height="18" fill="white"><path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0z"/></svg>`;
    }
    return "🎵";
  }

  // --- LÓGICA DE VENTANA Y ANIMACIÓN ---
  let isExpanded = false;
  let isOverControls = false;
  let hoverTimer: ReturnType<typeof setInterval> | null = null;
  let leaveTimer: ReturnType<typeof setInterval> | null = null;

  let anchorCenterX = 0;
  let anchorBottom = 0;
  let scaleFactor = 1;

  // 🌟 NUEVO: Variable reactiva para el Modo Navegador
  let isBrowser = false;

  $: {
    const id = (data.app_id || "").toLowerCase();
    const newIsBrowser =
      id.includes("chrome") ||
      id.includes("edge") ||
      id.includes("firefox") ||
      id.includes("opera") ||
      id.includes("brave");

    if (newIsBrowser !== isBrowser) {
      isBrowser = newIsBrowser;
      // Redimensionamos al vuelo si la tarjeta ya está abierta
      if (isExpanded) applyWindowSize(true);
    }
  }

  onMount(async () => {
    fetchMetadata();
    interval = setInterval(fetchMetadata, 1000);
    posInterval = setInterval(tickPosition, 250);

    await listen<any>("sync-widget-anchor", (event) => {
      // 🌟 Unificación: Solo procesamos si el evento es para nosotros
      if (event.payload.widgetId !== "media") return;

      console.log("[Media-Card] Received anchor sync:", event.payload);
      anchorCenterX = event.payload.centerX;
      anchorBottom = event.payload.taskbarBottom;
      scaleFactor = event.payload.scale;
      if (!isExpanded) applyWindowSize(false);
    });

    await emit("request-sync");
  });

  onDestroy(() => {
    clearInterval(interval);
    clearInterval(posInterval);
  });

  let lastW = 0;
  let lastH = 0;

  async function applyWindowSize(expanded: boolean) {
    if (anchorCenterX === 0) return;

    const w = 300;
    // 🌟 NUEVO: Altura dinámica dependiendo de la fuente
    const h = expanded ? (isBrowser ? 280 : 380) : 48;

    const physicalW = Math.round(w * scaleFactor);
    const physicalH = Math.round(h * scaleFactor);
    const physicalX = Math.round(anchorCenterX - physicalW / 2);
    const physicalY = Math.round(anchorBottom - physicalH);

    if (physicalW === lastW && physicalH === lastH) return;
    lastW = physicalW;
    lastH = physicalH;

    await invoke("manage_window", {
      label: "media_card",
      action: {
        type: "update",
        payload: {
          x: physicalX,
          y: physicalY,
          w: physicalW,
          h: physicalH,
        },
      },
    });
  }

  function handleEnter() {
    if (leaveTimer) {
      clearTimeout(leaveTimer);
      leaveTimer = null;
    }
    if (!isExpanded && !isOverControls) {
      if (hoverTimer) clearTimeout(hoverTimer);
      hoverTimer = setTimeout(async () => {
        if (isOverControls || isExpanded) return;
        await applyWindowSize(true);
        isExpanded = true;
      }, 650);
    }
  }

  function handleControlsEnter() {
    isOverControls = true;
    if (hoverTimer) {
      clearTimeout(hoverTimer);
      hoverTimer = null;
    }
  }

  function handleControlsLeave() {
    isOverControls = false;
    handleEnter();
  }

  function handleLeave() {
    if (hoverTimer) {
      clearTimeout(hoverTimer);
      hoverTimer = null;
    }
    leaveTimer = setTimeout(async () => {
      isExpanded = false;
      setTimeout(async () => {
        if (!isExpanded) {
          await applyWindowSize(false);
        }
      }, 300);
    }, 150);
  }

  $: progressPercent =
    data.duration_ms > 0 ? (displayPos / data.duration_ms) * 100 : 0;
  $: thumbnailUrl = data.thumbnail_base64
    ? `data:image/png;base64,${data.thumbnail_base64}`
    : "";

  // --- LÓGICA DE MARQUEE (SCROLL) ---
  function marqueeAction(node: HTMLElement, _deps: any) {
    const check = () => {
      const container = node;
      const content = node.querySelector(".marquee-content") as HTMLElement;
      if (!content) return;

      const diff = content.scrollWidth - container.clientWidth;
      if (diff > 0) {
        container.classList.add("is-overflowing");
        // Añadimos un pequeño extra (20px) para que el final se vea bien con el fade
        const scrollDistance = diff + 20;
        container.style.setProperty(
          "--scroll-distance",
          `-${scrollDistance}px`,
        );
        // Recalculamos la duración con la nueva distancia
        const duration = scrollDistance / 15 + 6;
        container.style.setProperty("--scroll-duration", `${duration}s`);
      } else {
        container.classList.remove("is-overflowing");
        container.style.removeProperty("--scroll-distance");
        container.style.removeProperty("--scroll-duration");
      }
    };

    const ro = new ResizeObserver(check);
    ro.observe(node);

    // Pequeño delay para asegurar que el DOM se ha renderizado
    setTimeout(check, 50);

    return {
      update() {
        setTimeout(check, 50);
      },
      destroy() {
        ro.disconnect();
      },
    };
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<main on:mouseenter={handleEnter} on:mouseleave={handleLeave}>
  <!-- 🌟 NUEVO: Añadimos la clase browser-mode condicionalmente -->
  <div
    class="media-morph {isExpanded ? 'expanded' : 'collapsed'} {isExpanded &&
    isBrowser
      ? 'browser-mode'
      : ''}"
  >
    {#if isExpanded}
      <div
        class="card-content-wrapper"
        in:fade={{ duration: 250, delay: 100 }}
        out:fade={{ duration: 150 }}
      >
        <div class="card-art-bg">
          {#if thumbnailUrl}
            <!-- 🌟 NUEVO: Desenfoque para disimular la pixelación del navegador -->
            <img
              src={thumbnailUrl}
              alt="Album art"
              class="card-art-img {isBrowser ? 'blurred-bg' : ''}"
              in:slideTransition={{
                direction: slideDirection === "left" ? "left" : "right",
              }}
              out:slideTransition={{
                direction: slideDirection === "left" ? "right" : "left",
              }}
            />
          {/if}
          <div class="card-art-overlay"></div>
        </div>

        <div class="card-content">
          <div class="card-track-info">
            <div
              in:slideTransition={{
                direction: slideDirection === "left" ? "left" : "right",
              }}
              out:slideTransition={{
                direction: slideDirection === "left" ? "right" : "left",
              }}
            >
              <div class="marquee-container" use:marqueeAction={data.title}>
                <span class="card-title marquee-content">{data.title}</span>
              </div>
              {#if data.artist || data.album}
                <div
                  class="marquee-container"
                  use:marqueeAction={data.artist + data.album}
                >
                  <span class="card-subtitle marquee-content">
                    {#if data.artist}{data.artist}{/if}{#if data.artist && data.album}
                      &nbsp;-&nbsp;
                    {/if}{#if data.album}{data.album}{/if}
                  </span>
                </div>
              {/if}
            </div>
          </div>

          <!-- 🌟 NUEVO: Solo mostramos la barra de progreso si no es un navegador -->
          {#if data.duration_ms > 0 && !isBrowser}
            <div class="card-progress-area">
              <div class="card-progress-bar">
                <div
                  class="card-progress-fill"
                  style="width: {progressPercent}%"
                ></div>
              </div>
              <div class="card-progress-times">
                <span>{formatTime(displayPos)}</span>
                <span>{formatTime(data.duration_ms)}</span>
              </div>
            </div>
          {/if}

          <div class="card-controls">
            <button
              class="card-btn"
              on:click={prevTrack}
              aria-label="Previous track"
            >
              <svg
                viewBox="0 0 24 24"
                width="22"
                height="22"
                fill="currentColor"
                ><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" /></svg
              >
            </button>
            <button
              class="card-btn card-btn-play"
              on:click={togglePlay}
              aria-label={isPlaying ? "Pause" : "Play"}
            >
              {#if isPlaying}
                <svg
                  viewBox="0 0 24 24"
                  width="28"
                  height="28"
                  fill="currentColor"
                  ><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg
                >
              {:else}
                <svg
                  viewBox="0 0 24 24"
                  width="28"
                  height="28"
                  fill="currentColor"><path d="M8 5v14l11-7z" /></svg
                >
              {/if}
            </button>
            <button
              class="card-btn"
              on:click={nextTrack}
              aria-label="Next track"
            >
              <svg
                viewBox="0 0 24 24"
                width="22"
                height="22"
                fill="currentColor"
                ><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" /></svg
              >
            </button>
          </div>
        </div>
      </div>
    {:else}
      <!-- INTERFAZ DE PÍLDORA -->
      <div
        class="pill-content-wrapper"
        in:fade={{ duration: 250, delay: 100 }}
        out:fade={{ duration: 150 }}
      >
        <div class="backgrounds">
          {#key data.title}
            <div
              class="bg-layer"
              style="--bg-img: {thumbnailUrl
                ? `url('${thumbnailUrl}')`
                : 'none'}"
              in:slideTransition={{
                direction: slideDirection === "left" ? "left" : "right",
              }}
              out:slideTransition={{
                direction: slideDirection === "left" ? "right" : "left",
              }}
            ></div>
          {/key}
        </div>

        <span class="app-icon z-1">{@html getAppIcon(data)}</span>

        <div class="track-info-container z-1">
          {#key data.title}
            <div
              class="track-info"
              in:slideTransition={{
                direction: slideDirection === "left" ? "left" : "right",
              }}
              out:slideTransition={{
                direction: slideDirection === "left" ? "right" : "left",
              }}
            >
              <div class="marquee-container" use:marqueeAction={data.title}>
                <span class="title marquee-content">{data.title}</span>
              </div>
              {#if data.artist}
                <div class="marquee-container" use:marqueeAction={data.artist}>
                  <span class="artist marquee-content">
                    {data.artist}
                  </span>
                </div>
              {/if}
            </div>
          {/key}
        </div>

        <div
          class="controls z-1"
          on:mouseenter={handleControlsEnter}
          on:mouseleave={handleControlsLeave}
        >
          <button on:click={prevTrack} aria-label="Previous">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"
              ><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" /></svg
            >
          </button>
          <button
            on:click={togglePlay}
            aria-label={isPlaying ? "Pause" : "Play"}
          >
            {#if isPlaying}
              <svg
                viewBox="0 0 24 24"
                width="16"
                height="16"
                fill="currentColor"
                ><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg
              >
            {:else}
              <svg
                viewBox="0 0 24 24"
                width="16"
                height="16"
                fill="currentColor"><path d="M8 5v14l11-7z" /></svg
              >
            {/if}
          </button>
          <button on:click={nextTrack} aria-label="Next">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"
              ><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" /></svg
            >
          </button>
        </div>
      </div>
    {/if}
  </div>
</main>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background-color: transparent;
    font-family: "Segoe UI", system-ui, sans-serif;
  }
  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  main {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: center;
  }

  .media-morph {
    background: #18181b;
    overflow: hidden;
    position: relative;
    transition:
      width 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      height 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      border-radius 0.3s cubic-bezier(0.2, 0.8, 0.2, 1),
      margin-bottom 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .media-morph.collapsed {
    width: 300px;
    height: 40px;
    border-radius: 50px;
    border: 1px solid #3f3f46;
    margin-bottom: 4px;
    transition: all 0.25s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .media-morph.collapsed:hover {
    background: #27272a;
    border-color: #52525b;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
  }

  /* Estado Tarjeta Estándar (Spotify, etc.) */
  .media-morph.expanded {
    width: 300px;
    height: 380px;
    border-radius: 16px 16px 0 0;
    border: 1px solid rgba(255, 255, 255, 0.08);
    margin-bottom: 0;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.6);
  }

  /* 🌟 NUEVO: Estado Tarjeta Navegador (Más bajita) */
  .media-morph.expanded.browser-mode {
    height: 280px;
  }

  .card-content-wrapper,
  .pill-content-wrapper {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .pill-content-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 16px;
    color: #e0e0e0;
  }

  .backgrounds {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 0;
    border-radius: 50px;
    overflow: hidden;
  }

  .bg-layer {
    width: 100%;
    height: 100%;
    background-image: linear-gradient(rgba(0, 0, 0, 0.7), rgba(0, 0, 0, 0.7)),
      var(--bg-img);
    background-size: cover;
    background-position: center;
  }

  .z-1 {
    z-index: 1;
  }

  .app-icon {
    font-size: 1.2rem;
    display: flex;
    align-items: center;
    color: #fff;
  }

  .track-info-container {
    display: grid;
    flex: 1;
    min-width: 0;
  }

  .track-info {
    grid-area: 1 / 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1px;
    min-width: 0;
  }

  .title {
    font-size: 0.85rem;
    font-weight: 700;
    color: #ffffff;
    line-height: 1.2;
  }

  .artist {
    font-size: 0.7rem;
    color: #c7c7c7;
    line-height: 1.2;
  }

  /* --- ESTILOS DE MARQUEE (SPOTIFY STYLE) --- */
  .marquee-container {
    display: flex;
    align-items: center;
    width: 100%;
    overflow: hidden;
    position: relative;
    mask-image: linear-gradient(
      to right,
      transparent 0%,
      black 1px,
      black calc(100% - 15px),
      transparent 100%
    );
  }

  .marquee-content {
    display: block;
    white-space: nowrap;
    will-change: transform;
  }

  :global(.is-overflowing) .marquee-content {
    animation: spotify-marquee var(--scroll-duration, 15s) linear infinite;
  }

  @keyframes spotify-marquee {
    0%,
    15% {
      transform: translateX(0);
    }
    45%,
    65% {
      transform: translateX(var(--scroll-distance, 0));
    }
    90%,
    100% {
      transform: translateX(0);
    }
  }

  .controls {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .controls button {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #fff;
    cursor: pointer;
    border-radius: 4px;
    width: 24px;
    height: 24px;
    transition:
      background 0.2s,
      transform 0.1s;
    outline: none;
  }

  .controls button:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: scale(1.18);
  }

  .controls button:active {
    transform: scale(0.9);
  }

  .card-art-bg {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }
  .card-art-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  /* 🌟 NUEVO: Clase condicional para desenfocar imágenes de baja calidad */
  .card-art-img.blurred-bg {
    filter: blur(12px);
    transform: scale(1.1); /* Evita márgenes blancos por el difuminado */
  }

  .card-art-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      to top,
      rgba(0, 0, 0, 0.92) 0%,
      rgba(0, 0, 0, 0.6) 40%,
      rgba(0, 0, 0, 0.15) 100%
    );
  }
  .card-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    height: 100%;
    padding: 20px 16px 24px;
  }
  .card-track-info {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
    min-width: 0;
  }
  .card-track-info > div {
    min-width: 0;
  }
  .card-title {
    display: block;
    font-size: 1rem;
    font-weight: 700;
    color: #fff;
    line-height: 1.3;
    margin-bottom: 2px;
  }
  .card-subtitle {
    display: block;
    font-size: 0.78rem;
    color: rgba(255, 255, 255, 0.6);
  }
  .card-progress-area {
    margin-bottom: 10px;
  }
  .card-progress-bar {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 2px;
    overflow: hidden;
  }
  .card-progress-fill {
    height: 100%;
    background: #fff;
    border-radius: 2px;
    transition: width 0.3s linear;
  }
  .card-progress-times {
    display: flex;
    justify-content: space-between;
    margin-top: 4px;
    font-size: 0.65rem;
    color: rgba(255, 255, 255, 0.5);
    font-variant-numeric: tabular-nums;
  }
  .card-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
  }
  .card-btn {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.8);
    cursor: pointer;
    transition:
      color 0.15s,
      background 0.15s;
  }
  .card-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }
  .card-btn-play {
    width: 48px;
    height: 48px;
    background: #fff;
    color: #18181b;
  }
  .card-btn-play:hover {
    background: #e4e4e7;
    transform: scale(1.05);
  }
</style>
