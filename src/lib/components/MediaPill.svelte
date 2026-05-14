<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import {
    isExpanded,
    setIsExpanded,
    mediaData,
    setMediaData,
  } from "$lib/stores/media";
  import { marqueeAction } from "$lib/actions/marquee";
  import { slideTransition } from "$lib/animations";
  import Icon from "$lib/iconMap";

  let interval: ReturnType<typeof setInterval>;
  let posInterval: ReturnType<typeof setInterval>;
  let displayPos = 0;
  let lastBackendPos = 0;
  let lastBackendTime = 0;
  let lastSeekTime = 0;

  async function fetchMetadata() {
    try {
      const res = await invoke<string>("get_current_media");
      const parsed = JSON.parse(res);
      setMediaData({ ...parsed, slideDirection: $mediaData.slideDirection });

      const now = Date.now();
      if (now - lastSeekTime > 2000) {
        lastBackendPos = parsed.position_ms;
        displayPos = parsed.position_ms;
        lastBackendTime = now;
      }
    } catch (e) {
      console.error("Error fetching media metadata:", e);
    }
  }

  function tickPosition() {
    if (!$mediaData.is_playing) return;
    const now = Date.now();
    if (now - lastSeekTime < 2000) return;
    const diff = now - lastBackendTime;
    displayPos = lastBackendPos + diff;
    if ($mediaData.duration_ms > 0 && displayPos > $mediaData.duration_ms) {
      displayPos = $mediaData.duration_ms;
    }
  }

  async function togglePlay() {
    await invoke("media_control", { action: "playpause" });
    fetchMetadata();
  }

  async function nextTrack() {
    setMediaData({ ...$mediaData, slideDirection: "left" });
    await invoke("media_control", { action: "next" });
    fetchMetadata();
  }

  async function prevTrack() {
    setMediaData({ ...$mediaData, slideDirection: "right" });
    await invoke("media_control", { action: "prev" });
    fetchMetadata();
  }

  async function seekForward() {
    const newPos = Math.min($mediaData.duration_ms, displayPos + 5000);
    displayPos = newPos;
    lastBackendPos = newPos;
    lastBackendTime = Date.now();
    lastSeekTime = lastBackendTime;
    await invoke("media_seek", { positionMs: newPos });
  }

  async function seekBackward() {
    const newPos = Math.max(0, displayPos - 5000);
    displayPos = newPos;
    lastBackendPos = newPos;
    lastBackendTime = Date.now();
    lastSeekTime = lastBackendTime;
    await invoke("media_seek", { positionMs: newPos });
  }

  function getAppIconName(media: any) {
    const appId = (media.app_id || "").toLowerCase();
    if (appId.includes("spotify")) return "simple-icons:spotify";
    if (appId.includes("chrome")) return "simple-icons:googlechrome";
    if (appId.includes("msedge")) return "simple-icons:microsoftedge";
    if (appId.includes("youtube")) return "simple-icons:youtube";
    return "fluent:music-note-2-24-filled";
  }

  function toggleExpand() {
    setIsExpanded(!$isExpanded);
  }

  onMount(() => {
    fetchMetadata();
    interval = setInterval(fetchMetadata, 1000);
    posInterval = setInterval(tickPosition, 250);
  });

  onDestroy(() => {
    clearInterval(interval);
    clearInterval(posInterval);
  });

  $: thumbnailUrl = $mediaData.thumbnail_base64
    ? `data:image/png;base64,${$mediaData.thumbnail_base64}`
    : "";
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="pill-morph {$isExpanded ? 'expanded' : 'collapsed'}"
  on:click={toggleExpand}
>
  <div class="backgrounds">
    {#key $mediaData.title}
      <div
        class="bg-layer"
        style="--bg-img: {thumbnailUrl ? `url('${thumbnailUrl}')` : 'none'}"
        in:slideTransition={{
          direction: $mediaData.slideDirection,
        }}
        out:slideTransition={{
          direction: $mediaData.slideDirection === "left" ? "right" : "left",
        }}
      ></div>
    {/key}
  </div>

  <div class="pill-content-wrapper {$isExpanded ? 'expanded' : ''}">
    <div class="pill-info-section">
      <span class="app-icon z-1"
        ><Icon icon={getAppIconName($mediaData)} width="18" height="18" /></span
      >

      <div class="track-info-container z-1">
        {#key $mediaData.title}
          <div
            class="track-info"
            in:slideTransition={{
              direction: $mediaData.slideDirection,
            }}
            out:slideTransition={{
              direction:
                $mediaData.slideDirection === "left" ? "right" : "left",
            }}
          >
            <div class="marquee-container" use:marqueeAction={$mediaData.title}>
              <span class="title marquee-content">{$mediaData.title}</span>
            </div>
            {#if $mediaData.artist}
              <div
                class="marquee-container"
                use:marqueeAction={$mediaData.artist}
              >
                <span class="artist marquee-content">
                  {$mediaData.artist}
                </span>
              </div>
            {/if}
          </div>
        {/key}
      </div>
    </div>

    <div class="controls z-1">
      {#if $isExpanded}
        <button on:click|stopPropagation={seekBackward} aria-label="Back 10s">
          <Icon icon="fluent:skip-back-10-24-filled" width="20" height="20" />
        </button>
      {/if}

      <button on:click|stopPropagation={prevTrack} aria-label="Previous">
        <Icon icon="fluent:previous-24-filled" width="16" height="16" />
      </button>

      <button
        on:click|stopPropagation={togglePlay}
        aria-label={$mediaData.is_playing ? "Pause" : "Play"}
      >
        {#if $mediaData.is_playing}
          <Icon
            icon="fluent:pause-24-filled"
            width={$isExpanded ? "24" : "16"}
            height={$isExpanded ? "24" : "16"}
          />
        {:else}
          <Icon
            icon="fluent:play-24-filled"
            width={$isExpanded ? "24" : "16"}
            height={$isExpanded ? "24" : "16"}
          />
        {/if}
      </button>

      <button on:click|stopPropagation={nextTrack} aria-label="Next">
        <Icon icon="fluent:next-24-filled" width="16" height="16" />
      </button>

      {#if $isExpanded}
        <button on:click|stopPropagation={seekForward} aria-label="Forward 10s">
          <Icon
            icon="fluent:skip-forward-10-24-filled"
            width="20"
            height="20"
          />
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .pill-morph {
    width: 300px;
    height: 40px;
    border-radius: 50px;
    background: #18181b;
    border: 1px solid #3f3f46;
    position: relative;
    overflow: hidden;
    transition: all 0.25s cubic-bezier(0.2, 0.8, 0.2, 1);
    cursor: pointer;
  }

  .pill-morph:hover {
    background: #27272a;
    border-color: #52525b;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
  }

  .pill-morph.expanded {
    background: #000;
    border-color: rgba(255, 255, 255, 0.15);
  }

  .pill-morph.expanded:hover {
    background: #000;
  }

  .pill-content-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
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
    transition: opacity 0.3s ease;
  }

  .expanded .backgrounds {
    opacity: 0;
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

  .pill-info-section {
    position: absolute;
    left: 16px;
    right: 90px;
    top: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    transition:
      opacity 0.2s,
      transform 0.3s;
    transform: translateX(0);
  }

  .expanded .pill-info-section {
    opacity: 0;
    transform: translateX(-20px);
    pointer-events: none;
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
    position: absolute;
    right: 16px;
    top: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .expanded .controls {
    right: 50%;
    transform: translateX(50%);
    gap: 24px;
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
    transition: all 0.2s;
    outline: none;
  }

  .expanded .controls button {
    width: 32px;
    height: 32px;
  }

  .expanded .controls button :global(svg) {
    width: 20px;
    height: 20px;
  }

  .controls button:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: scale(1.18);
  }

  .controls button:active {
    transform: scale(0.9);
  }
</style>
