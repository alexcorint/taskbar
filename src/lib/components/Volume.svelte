<script lang="ts">
    import { volume, setVolumeImmediate, toggleMuteImmediate } from "$lib/stores/system";
    import { fade } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";

    // Reactivo al store — sin polling propio (era 500ms antes)
    $: volumeLevel = $volume.volume;
    $: isMuted = $volume.is_muted;

    async function volUp() {
        await invoke("volume_control", { action: "Up" });
    }
    async function volDown() {
        await invoke("volume_control", { action: "Down" });
    }

    function handleVolScroll(e: WheelEvent) {
        if (e.deltaY < 0) volUp();
        else volDown();
    }
</script>

<div
    class="volume-btn"
    onwheel={(e) => { e.preventDefault(); handleVolScroll(e); }}
>
    <svg
        viewBox="0 0 24 24"
        width="1.1em"
        height="1.1em"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
    >
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />

        {#if isMuted || volumeLevel === 0}
            <g transition:fade={{ duration: 150 }}>
                <line x1="22" x2="16" y1="9" y2="15" />
                <line x1="16" x2="22" y1="9" y2="15" />
            </g>
        {:else}
            <!-- 1ª onda: 1%–100% -->
            <path d="M 14.83 9.17 a 4 4 0 0 1 0 5.66" transition:fade={{ duration: 150 }} />

            <!-- 2ª onda: > 33% -->
            {#if volumeLevel > 0.33}
                <path d="M 16.95 7.05 a 7 7 0 0 1 0 9.9" transition:fade={{ duration: 150 }} />
            {/if}

            <!-- 3ª onda: > 66% -->
            {#if volumeLevel > 0.66}
                <path d="M 19.07 4.93 a 10 10 0 0 1 0 14.14" transition:fade={{ duration: 150 }} />
            {/if}
        {/if}
    </svg>
</div>

<style>
    .volume-btn {
        display: flex;
        align-items: center;
        justify-content: center;
    }
</style>
