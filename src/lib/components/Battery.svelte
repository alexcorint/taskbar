<script lang="ts">
    import { battery } from "$lib/stores/system";
    import { anchor } from "$lib/actions/anchor";
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import { fade } from "svelte/transition";

    // Usar Svelte 5 Tween para consistencia con el resto de la app
    const battPercent = new Tween(100, { duration: 800, easing: cubicOut });

    let showLightning = $state(false);
    let lightningTimeout: ReturnType<typeof setTimeout>;
    let prevCharging = $state(false);

    // Reactivo al store centralizado usando $effect de Svelte 5
    $effect(() => {
        const currentBattery = $battery;
        battPercent.set(currentBattery.percentage);

        if (currentBattery.is_charging && !prevCharging) {
            showLightning = true;
            clearTimeout(lightningTimeout);
            lightningTimeout = setTimeout(() => {
                showLightning = false;
            }, 1000);
        }
        prevCharging = currentBattery.is_charging;
    });

    let iconColor = $derived($battery.is_charging
        ? "#22c55e"
        : $battery.battery_saver
          ? "#fbbf24"
          : battPercent.current <= 20
            ? "#ef4444"
            : "currentColor");
</script>

<div
    class="battery-container"
    title={$battery.is_charging
        ? "Cargando"
        : $battery.battery_saver
          ? "Ahorro de batería"
          : "Batería"}
    use:anchor={"battery"}
>
    <svg viewBox="0 0 27 14" width="1.8em" height="1.1em" fill="currentColor">
        <defs>
            <mask id="cutout-text">
                <rect width="100%" height="100%" fill="white" />
                {#if !showLightning}
                    <text
                        x="12.5"
                        y="9.5"
                        font-size="7.5"
                        font-family="system-ui, -apple-system, sans-serif"
                        font-weight="700"
                        text-anchor="middle"
                        fill="black"
                        transition:fade={{ duration: 250 }}
                    >
                        {Math.round(battPercent.current)}
                    </text>
                {/if}
            </mask>
            <clipPath id="empty-level">
                <rect
                    x={2 + 21 * (battPercent.current / 100)}
                    y="0"
                    width={27 - (2 + 21 * (battPercent.current / 100))}
                    height="14"
                />
            </clipPath>
        </defs>

        <rect
            x="1" y="1" width="23" height="12" rx="3.5"
            fill="none" stroke="white" stroke-width="1" opacity="0.4"
        />
        <path d="M24 4.5 Q25.5 4.5 25.5 7 Q25.5 9.5 24 9.5 Z" fill="white" opacity="0.4" />

        <rect
            x="2" y="2"
            width={21 * (battPercent.current / 100)}
            height="10" rx="2.5"
            fill={iconColor}
            mask="url(#cutout-text)"
            style="transition: fill 0.6s cubic-bezier(0.4, 0, 0.2, 1);"
        />

        {#if showLightning}
            <path
                d="M 13 2.5 L 9.5 7 H 12.5 L 11.5 11.5 L 15.5 6 H 12.5 Z"
                fill="#ffffff" stroke="#ffffff" stroke-width="1" stroke-linejoin="round"
                transition:fade={{ duration: 250 }}
            />
        {:else}
            <text
                x="12.5" y="9.5"
                font-size="7.5"
                font-family="system-ui, -apple-system, sans-serif"
                font-weight="700"
                text-anchor="middle"
                fill={iconColor}
                clip-path="url(#empty-level)"
                transition:fade={{ duration: 250 }}
            >
                {Math.round(battPercent.current)}
            </text>
        {/if}
    </svg>
</div>

<style>
    .battery-container {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 4px;
        border-radius: 6px;
        transition: background 0.2s;
    }

    .battery-container:hover {
        background: rgba(255, 255, 255, 0.1);
    }
</style>
