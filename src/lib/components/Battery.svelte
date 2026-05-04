<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { tweened } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import { fade } from "svelte/transition";
    import { anchor } from "$lib/actions/anchor";
    import { emit, listen } from "@tauri-apps/api/event";

    const battPercent = tweened(100, {
        duration: 800,
        easing: cubicOut,
    });
    let isCharging = false;
    let isBatterySaver = false;
    let showLightning = false;
    let lightningTimeout: ReturnType<typeof setTimeout>;
    let interval: ReturnType<typeof setInterval>;

    // --- LÓGICA DE MENÚ ---
    let showMenu = false;
    let isMouseInMenu = false;
    let menuTimer: ReturnType<typeof setTimeout> | null = null;

    async function updateBattery() {
        try {
            const batt: any = await invoke("get_battery_status");
            battPercent.set(batt.percentage);
            isBatterySaver = batt.battery_saver;

            if (batt.is_charging && !isCharging) {
                showLightning = true;
                clearTimeout(lightningTimeout);
                lightningTimeout = setTimeout(() => {
                    showLightning = false;
                }, 1000);
            }

            isCharging = batt.is_charging;
        } catch (e) {
            console.error("Error leyendo batería:", e);
        }
    }

    // Color del icono basado en el estado
    $: iconColor = isCharging
        ? "#22c55e"
        : isBatterySaver
          ? "#fbbf24" // Amarillo anaranjado
          : $battPercent <= 20
            ? "#ef4444"
            : "currentColor";

    let isMouseOnBattery = false;

    async function toggleMenu(force?: boolean) {
        if (force === false || (force === undefined && showMenu)) {
            showMenu = false;
            await emit("toggle-battery-menu", false);
            await invoke("manage_window", {
                label: "battery_menu",
                action: { type: "hide" },
            });
        } else {
            showMenu = true;
            await invoke("manage_window", {
                label: "battery_menu",
                action: { type: "show" },
            });
            await emit("toggle-battery-menu", true);
        }
    }

    function checkCloseMenu() {
        if (menuTimer) clearTimeout(menuTimer);
        menuTimer = setTimeout(() => {
            if (!isMouseOnBattery && !isMouseInMenu && showMenu) {
                toggleMenu(false);
            }
        }, 500); // Margen generoso para moverse entre barra y menú
    }

    onMount(() => {
        updateBattery();
        interval = setInterval(updateBattery, 2000);

        let unlistenFn: (() => void) | null = null;
        listen<boolean>("battery-menu-hover", (event) => {
            isMouseInMenu = event.payload;
            if (!isMouseInMenu) {
                checkCloseMenu();
            }
        }).then((fn) => (unlistenFn = fn));

        return () => {
            if (unlistenFn) unlistenFn();
            clearInterval(interval);
            clearTimeout(lightningTimeout);
            if (menuTimer) clearTimeout(menuTimer);
        };
    });
</script>

<!-- Contenedor principal que maneja el hover -->
<div
    class="battery-container"
    title={isCharging
        ? "Cargando"
        : isBatterySaver
          ? "Ahorro de batería"
          : "Batería"}
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
                        {Math.round($battPercent)}
                    </text>
                {/if}
            </mask>
            <clipPath id="empty-level">
                <rect
                    x={2 + 21 * ($battPercent / 100)}
                    y="0"
                    width={27 - (2 + 21 * ($battPercent / 100))}
                    height="14"
                />
            </clipPath>
        </defs>

        <rect
            x="1"
            y="1"
            width="23"
            height="12"
            rx="3.5"
            fill="none"
            stroke="white"
            stroke-width="1"
            opacity="0.4"
        />
        <path
            d="M24 4.5 Q25.5 4.5 25.5 7 Q25.5 9.5 24 9.5 Z"
            fill="white"
            opacity="0.4"
        />

        <rect
            x="2"
            y="2"
            width={21 * ($battPercent / 100)}
            height="10"
            rx="2.5"
            fill={iconColor}
            mask="url(#cutout-text)"
            style="transition: fill 0.6s cubic-bezier(0.4, 0, 0.2, 1);"
        />

        {#if showLightning}
            <path
                d="M 13 2.5 L 9.5 7 H 12.5 L 11.5 11.5 L 15.5 6 H 12.5 Z"
                fill="#ffffff"
                stroke="#ffffff"
                stroke-width="1"
                stroke-linejoin="round"
                transition:fade={{ duration: 250 }}
            />
        {:else}
            <text
                x="12.5"
                y="9.5"
                font-size="7.5"
                font-family="system-ui, -apple-system, sans-serif"
                font-weight="700"
                text-anchor="middle"
                fill={iconColor}
                clip-path="url(#empty-level)"
                transition:fade={{ duration: 250 }}
            >
                {Math.round($battPercent)}
            </text>
        {/if}
    </svg>
</div>

<style>
    .battery-container {
        position: relative; /* Para posicionar el menú */
        display: flex;
        align-items: center;
        justify-content: center;
    }
</style>
