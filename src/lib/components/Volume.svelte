<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { fade } from "svelte/transition";
    import { anchor } from "$lib/actions/anchor";
    import { onMount } from "svelte";

    let isMuted = false;
    let volumeLevel = 0.5;
    let interval: ReturnType<typeof setInterval>;

    async function updateVolume() {
        try {
            const vol: any = await invoke("get_volume_status");
            volumeLevel = vol.volume;
            isMuted = vol.is_muted;
        } catch (e) {
            console.error("Error leyendo volumen:", e);
        }
    }

    onMount(() => {
        updateVolume();
        interval = setInterval(updateVolume, 500);

        return () => {
            clearInterval(interval);
        };
    });

    async function volMute() {
        await invoke("volume_control", { action: "Mute" });
        setTimeout(updateVolume, 50);
    }
    async function volUp() {
        await invoke("volume_control", { action: "Up" });
        setTimeout(updateVolume, 50);
    }
    async function volDown() {
        await invoke("volume_control", { action: "Down" });
        setTimeout(updateVolume, 50);
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
            <!-- 1st wave: 1% to 100% -->
            <path
                d="M 14.83 9.17 a 4 4 0 0 1 0 5.66"
                transition:fade={{ duration: 150 }}
            />

            <!-- 2nd wave: > 33% -->
            {#if volumeLevel > 0.33}
                <path
                    d="M 16.95 7.05 a 7 7 0 0 1 0 9.9"
                    transition:fade={{ duration: 150 }}
                />
            {/if}

            <!-- 3rd wave: > 66% -->
            {#if volumeLevel > 0.66}
                <path
                    d="M 19.07 4.93 a 10 10 0 0 1 0 14.14"
                    transition:fade={{ duration: 150 }}
                />
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
