<script lang="ts">
    import { volume, setVolumeImmediate, toggleMuteImmediate } from "$lib/stores/system";

    let { brightness = $bindable(50), volumeValue = $bindable(0.5), updateBrightness } = $props();
</script>

<div class="controls-section">
    <!-- Brillo -->
    <div class="control-row">
        <span class="control-icon">
            <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="18"
                height="18"
            >
                <circle cx="12" cy="12" r={brightness < 33 ? 3 : brightness < 66 ? 4 : 5} />
                <g opacity={0.3 + (brightness / 100) * 0.7}>
                    <path d="M12 2v2" /><path d="M12 20v2" />
                    <path d="m4.93 4.93 1.41 1.41" /><path d="m17.66 17.66 1.41 1.41" />
                    <path d="M2 12h2" /><path d="M20 12h2" />
                    <path d="m6.34 17.66-1.41 1.41" /><path d="m19.07 4.93-1.41 1.41" />
                </g>
            </svg>
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
            <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="18"
                height="18"
            >
                {#if $volume.is_muted}
                    <path d="M11 4.7L6 9H2v6h4l5 4.3V4.7z" />
                    <line x1="22" y1="9" x2="16" y2="15" />
                    <line x1="16" y1="9" x2="22" y2="15" />
                {:else}
                    <path d="M11 4.7L6 9H2v6h4l5 4.3V4.7z" />
                    {#if $volume.volume >= 0.01}
                        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" opacity={$volume.volume < 0.33 ? 0.3 : 1} />
                    {/if}
                    {#if $volume.volume >= 0.33}
                        <path d="M19.07 4.93a10 10 0 0 1 0 14.14" opacity={$volume.volume < 0.66 ? 0.3 : 1} />
                    {/if}
                {/if}
            </svg>
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
