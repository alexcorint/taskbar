<script lang="ts">
    import { volume, setVolumeImmediate, toggleMuteImmediate } from "$lib/stores/system";
    import Icon from "$lib/iconMap";

    let { brightness = $bindable(50), volumeValue = $bindable(0.5), updateBrightness } = $props();

    const volumeIcon = $derived.by(() => {
        if ($volume.is_muted || $volume.volume === 0) return "fluent:speaker-mute-24-filled";
        if ($volume.volume > 0.66) return "fluent:speaker-2-24-filled";
        if ($volume.volume > 0.33) return "fluent:speaker-1-24-filled";
        return "fluent:speaker-0-24-filled";
    });

    const brightnessIcon = $derived.by(() => {
        if (brightness > 80) return "fluent:brightness-high-24-filled";
        if (brightness > 40) return "fluent:brightness-low-24-filled";
        return "fluent:brightness-low-20-filled"; // Or another variant
    });
</script>

<div class="controls-section">
    <!-- Brillo -->
    <div class="control-row">
        <span class="control-icon">
            <Icon icon={brightnessIcon} width="18" height="18" />
        </span>
        <input
            type="range"
            min="0"
            max="100"
            bind:value={brightness}
            oninput={(e) => updateBrightness(parseInt(e.currentTarget.value))}
            class="control-slider"
            style="--progress: {brightness}%"
        />
        <span class="control-value">{brightness}%</span>
    </div>

    <!-- Volumen -->
    <div class="control-row">
        <button class="control-icon-btn" onclick={toggleMuteImmediate}>
            <Icon icon={volumeIcon} width="18" height="18" />
        </button>
        <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            bind:value={volumeValue}
            oninput={(e) => setVolumeImmediate(parseFloat(e.currentTarget.value))}
            class="control-slider"
            style="--progress: {volumeValue * 100}%"
        />
        <span class="control-value">{Math.round(volumeValue * 100)}%</span>
    </div>
</div>

<style>
    .controls-section {
        padding: 12px 0;
        border-top: 1px solid rgba(255, 255, 255, 0.08);
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    .control-row {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .control-icon,
    .control-icon-btn {
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 8px;
        color: inherit;
        cursor: pointer;
        transition: all 0.4s;
    }

    .control-icon-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.25);
        transform: scale(1.1);
    }

    .control-value {
        font-size: 0.8rem;
        font-weight: 700;
        min-width: 38px;
        text-align: right;
        opacity: 0.7;
    }

    .control-slider {
        flex-grow: 1;
        -webkit-appearance: none;
        appearance: none;
        height: 6px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 3px;
        outline: none;
        background-image: linear-gradient(#3b82f6, #3b82f6);
        background-size: var(--progress) 100%;
        background-repeat: no-repeat;
    }

    .control-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        height: 18px;
        width: 18px;
        border-radius: 50%;
        background: #fff;
        cursor: pointer;
        border: 4px solid #3b82f6;
        box-shadow: 0 0 10px rgba(0, 0, 0, 0.3);
        transition: all 0.2s;
    }

    .control-slider::-webkit-slider-thumb:hover {
        transform: scale(1.2);
        box-shadow: 0 0 15px rgba(59, 130, 246, 0.5);
    }
</style>
