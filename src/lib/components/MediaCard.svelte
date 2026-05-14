<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, emit } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { isExpanded, setIsExpanded, mediaData } from "$lib/stores/media";
  import { marqueeAction } from "$lib/actions/marquee";
  import { slideTransition } from "$lib/animations";

  let displayPos = 0;
  let lastBackendPos = 0;
  let lastBackendTime = 0;
  let posInterval: ReturnType<typeof setInterval>;

  let isBrowser = false;
  let anchorCenterX = 0;
  let anchorBottom = 0;
  let scaleFactor = 1;

  let isDraggingProgress = false;
  let lastSeekTime = 0;
  let lastTitle = "";

  $: {
    const id = ($mediaData.app_id || "").toLowerCase();
    isBrowser =
      id.includes("chrome") ||
      id.includes("edge") ||
      id.includes("firefox") ||
      id.includes("opera") ||
      id.includes("brave");
  }

  // Reactividad para el redimensionamiento de ventana
  $: if (anchorCenterX !== 0) {
    applyWindowSize($isExpanded);
  }

  function tickPosition() {
    if (!$mediaData.is_playing || isDraggingProgress) return;
    const now = Date.now();
    const diff = now - lastBackendTime;
    displayPos = lastBackendPos + diff;
    if ($mediaData.duration_ms > 0 && displayPos > $mediaData.duration_ms) {
      displayPos = $mediaData.duration_ms;
    }
  }

  function formatTime(ms: number) {
    if (!ms || ms < 0) return "0:00";
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  let lastX = 0,
    lastY = 0,
    lastW = 0,
    lastH = 0;

  async function applyWindowSize(expanded: boolean) {
    if (anchorCenterX === 0) return;

    const w = 300;
    // El alto de la tarjeta es 332 (o 232 en browser), sin los 48px de la píldora
    const h = expanded ? (isBrowser ? 300 : 400) : 0;

    const physicalW = Math.round(w * scaleFactor);
    const physicalH = Math.round(h * scaleFactor);
    const physicalX = Math.round(anchorCenterX * scaleFactor - physicalW / 2);
    // Posicionar el borde inferior de la ventana a 48px (barra) + 8px (margen) desde el fondo
    const physicalY = Math.round(
      (anchorBottom - 48 - 8) * scaleFactor - physicalH,
    );

    if (
      physicalX === lastX &&
      physicalY === lastY &&
      physicalW === lastW &&
      physicalH === lastH
    ) {
      return;
    }
    lastX = physicalX;
    lastY = physicalY;
    lastW = physicalW;
    lastH = physicalH;

    await invoke("manage_window", {
      label: "media_card",
      action: {
        type: "update",
        x: physicalX,
        y: physicalY,
        w: physicalW,
        h: physicalH,
      },
    });
  }

  let unlistenSync: () => void;

  onMount(() => {
    posInterval = setInterval(tickPosition, 250);

    listen<any>("sync-widget-anchor", (event) => {
      if (event.payload.widgetId !== "media") return;
      anchorCenterX = event.payload.centerX;
      anchorBottom = event.payload.taskbarBottom;
      scaleFactor = event.payload.scale;
    }).then((u) => (unlistenSync = u));

    emit("request-sync");
  });

  onDestroy(() => {
    clearInterval(posInterval);
    if (unlistenSync) unlistenSync();
  });

  $: progressPercent =
    $mediaData.duration_ms > 0
      ? (displayPos / $mediaData.duration_ms) * 100
      : 0;
  $: thumbnailUrl = $mediaData.thumbnail_base64
    ? `data:image/png;base64,${$mediaData.thumbnail_base64}`
    : "";

  // Sincronizar posición de medios cuando cambia el store
  $: {
    const now = Date.now();
    const titleChanged = $mediaData.title !== lastTitle;

    if (titleChanged || (!isDraggingProgress && now - lastSeekTime > 2000)) {
      lastBackendPos = $mediaData.position_ms;
      displayPos = $mediaData.position_ms;
      lastBackendTime = now;
      lastTitle = $mediaData.title;
    }
  }

  function handleSeek(e: PointerEvent) {
    const bar = e.currentTarget as HTMLElement;
    const rect = bar.getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const percent = x / rect.width;
    displayPos = Math.round(percent * $mediaData.duration_ms);
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    isDraggingProgress = true;
    handleSeek(e);
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (isDraggingProgress) {
      handleSeek(e);
    }
  }

  function onPointerUp(e: PointerEvent) {
    if (isDraggingProgress) {
      isDraggingProgress = false;
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);

      // Sincronizar estados locales para evitar saltos
      lastBackendPos = displayPos;
      lastBackendTime = Date.now();
      lastSeekTime = lastBackendTime;

      invoke("media_seek", { positionMs: displayPos });
    }
  }

  function handleLeave() {
    // Timer para cerrar si el ratón sale
    setTimeout(() => {
      // Solo cerramos si no se ha vuelto a entrar (esto solía estar en main)
      // Pero ahora el componente es más independiente.
      // Por ahora mantendremos la lógica de cierre en el componente si es necesario.
    }, 150);
  }
</script>

<div
  class="card-morph {$isExpanded ? 'visible' : 'hidden'} {isBrowser
    ? 'browser-mode'
    : ''}"
>
  <div class="card-content-wrapper {$isExpanded ? 'visible' : 'hidden'}">
    <div class="card-art-bg">
      {#if thumbnailUrl}
        <img
          src={thumbnailUrl}
          alt="Album art"
          class="card-art-img {isBrowser ? 'blurred-bg' : ''}"
          in:slideTransition={{
            direction: $mediaData.slideDirection,
          }}
          out:slideTransition={{
            direction: $mediaData.slideDirection === "left" ? "right" : "left",
          }}
        />
      {/if}
      <div class="card-art-overlay"></div>
    </div>

    <div class="card-content">
      <div class="spacer" style="flex: 1;"></div>

      <div class="card-track-info">
        <div
          in:slideTransition={{
            direction: $mediaData.slideDirection,
          }}
          out:slideTransition={{
            direction: $mediaData.slideDirection === "left" ? "right" : "left",
          }}
        >
          <div class="marquee-container" use:marqueeAction={$mediaData.title}>
            <span class="card-title marquee-content">{$mediaData.title}</span>
          </div>
          {#if $mediaData.artist || $mediaData.album}
            <div
              class="marquee-container"
              use:marqueeAction={$mediaData.artist + $mediaData.album}
            >
              <span class="card-subtitle marquee-content">
                {#if $mediaData.artist}{$mediaData.artist}{/if}{#if $mediaData.artist && $mediaData.album}
                  &nbsp;-&nbsp;
                {/if}{#if $mediaData.album}{$mediaData.album}{/if}
              </span>
            </div>
          {/if}
        </div>
      </div>

      {#if $mediaData.duration_ms > 0}
        <div class="card-progress-area">
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div
            class="card-progress-bar {isDraggingProgress ? 'is-dragging' : ''}"
            on:pointerdown={onPointerDown}
            on:pointermove={onPointerMove}
            on:pointerup={onPointerUp}
          >
            <div
              class="card-progress-fill"
              style="width: {progressPercent}%"
            ></div>
            <div
              class="card-progress-knob"
              style="left: {progressPercent}%"
            ></div>
          </div>
          <div class="card-progress-times">
            <span>{formatTime(displayPos)}</span>
            <span>{formatTime($mediaData.duration_ms)}</span>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .card-morph {
    width: 300px;
    height: 100%;
    box-sizing: border-box;
    border-radius: 16px;
    background: #18181b;
    position: relative;
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    transition:
      opacity 0.2s ease,
      transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    opacity: 1;
    transform: translateY(0) scale(1);
  }

  .card-morph.hidden {
    opacity: 0;
    transform: translateY(10px) scale(0.95);
    pointer-events: none;
    border-width: 0;
  }

  .card-content-wrapper {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
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

  .card-art-img.blurred-bg {
    filter: blur(12px);
    transform: scale(1.1);
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
    box-sizing: border-box;
    padding: 20px 20px 20px;
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
    border-radius: 4px;
    position: relative;
    cursor: pointer;
    transition: height 0.2s;
  }
  .card-progress-bar:hover {
    height: 6px;
  }
  .card-progress-fill {
    height: 100%;
    background: #fff;
    border-radius: 4px;
    transition: width 0.3s linear;
  }
  .is-dragging .card-progress-fill {
    transition: none;
  }
  .card-progress-knob {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 12px;
    height: 12px;
    background: #fff;
    border-radius: 50%;
    opacity: 0;
    transition: opacity 0.2s;
    pointer-events: none;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
  }
  .card-progress-bar:hover .card-progress-knob,
  .is-dragging .card-progress-knob {
    opacity: 1;
  }
  .is-dragging .card-progress-knob {
    transition: none;
  }
  .card-progress-times {
    display: flex;
    justify-content: space-between;
    width: 100%;
    margin-top: 8px;
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.8);
    font-variant-numeric: tabular-nums;
    font-weight: 500;
  }

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

  .hidden {
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
  }

  .visible {
    opacity: 1;
    visibility: visible;
    pointer-events: auto;
  }
</style>
