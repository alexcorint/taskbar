<script lang="ts">
    import { volume, setVolumeImmediate, toggleMuteImmediate } from "$lib/stores/system";
    import { fade } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";
    import Icon from "$lib/iconMap";

    // Reactivo al store — sin polling propio (era 500ms antes)
    let volumeLevel = $derived($volume.volume);
    let isMuted = $derived($volume.is_muted);

    const iconName = $derived.by(() => {
        if (isMuted || volumeLevel === 0) return "fluent:speaker-mute-24-filled";
        if (volumeLevel > 0.66) return "fluent:speaker-2-24-filled";
        if (volumeLevel > 0.33) return "fluent:speaker-1-24-filled";
        return "fluent:speaker-0-24-filled";
    });

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
    <div class="icon-wrapper" transition:fade={{ duration: 150 }}>
        <Icon icon={iconName} width="20" height="20" />
    </div>
</div>

<style>
    .volume-btn {
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .icon-wrapper {
        display: flex;
        align-items: center;
        justify-content: center;
    }
</style>
