<script module>
    const performanceIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M5.6 18.4 A 9 9 0 1 1 18.4 18.4" /><line x1="12" y1="3" x2="12" y2="5" /><line x1="3" y1="12" x2="5" y2="12" /><line x1="21" y1="12" x2="19" y2="12" /><line x1="5.6" y1="5.6" x2="7" y2="7" /><line x1="18.4" y1="5.6" x2="17" y2="7" /><line x1="12" y1="12" x2="17" y2="17" /><circle cx="12" cy="12" r="1.5" /></svg>`;
    const balancedIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="4" x2="12" y2="20" /><line x1="8" y1="20" x2="16" y2="20" /><circle cx="12" cy="5" r="1.5" /><line x1="4" y1="5" x2="20" y2="5" /><polygon points="4,5 1,14 7,14" /><polygon points="20,5 17,14 23,14" /></svg>`;
    const saverIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 3v18M21 9a9 9 0 1 1-18 0"/><path d="M15 9h6M3 9h6"/><path d="M3.75 14.25a6.75 6.75 0 0 1 13.5 0"/></svg>`;
    const ecoIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><g transform="translate(12, 12)"><g transform="rotate(0)"><path d="M 0,-11 C 5,-11 8,-6 5,-1 C 0,-3 -3,-7 0,-11 Z" /><path d="M 0,-11 Q 3,-7 4,-2" /></g><g transform="rotate(120)"><path d="M 0,-11 C 5,-11 8,-6 5,-1 C 0,-3 -3,-7 0,-11 Z" /><path d="M 0,-11 Q 3,-7 4,-2" /></g><g transform="rotate(240)"><path d="M 0,-11 C 5,-11 8,-6 5,-1 C 0,-3 -3,-7 0,-11 Z" /><path d="M 0,-11 Q 3,-7 4,-2" /></g></g></svg>`;
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit, listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";

    // --- PROPS ---
    let { isVisible = false } = $props();

    // --- ESTADO ---
    let batteryStatus = $state({
        percentage: 100,
        is_charging: false,
        battery_saver: false,
    });
    const battPercent = new Tween(100, { duration: 800, easing: cubicOut });

    let brightness = $state(50);
    let volume = $state(0.5);
    let isMuted = $state(false);
    let powerProfiles = $state<any[]>([]);
    let activeProfileGuid = $state("");

    // --- POSICIONAMIENTO ---
    let anchorCenterX = 0;
    let anchorBottom = 0;
    let monitorWidth = 0;

    async function applyWindowPosition() {
        if (anchorCenterX === 0) return;

        const w = 450;
        const h = 420;

        let logicalX = anchorCenterX - w / 2;
        const margin = 10;
        if (logicalX < margin) logicalX = margin;
        if (monitorWidth > 0 && logicalX + w > monitorWidth - margin) {
            logicalX = monitorWidth - w - margin;
        }

        const logicalY = anchorBottom - 48 - h - 10;

        await invoke("manage_window", {
            label: "battery_menu",
            action: {
                type: "updatelogical",
                payload: { x: logicalX, y: logicalY, w, h },
            },
        });
    }

    // --- REACCIÓN A VISIBILIDAD ---
    $effect(() => {
        if (isVisible && anchorCenterX !== 0) {
            applyWindowPosition();
        }
    });

    // --- FETCHING ---
    let isFirstLoad = true;
    async function fetchData() {
        try {
            const batt: any = await invoke("get_battery_status");
            batteryStatus = batt;

            if (isFirstLoad) {
                await battPercent.set(batt.percentage, { duration: 0 });
                isFirstLoad = false;
            } else {
                battPercent.set(batt.percentage);
            }

            brightness = await invoke("get_brightness");

            const vol: any = await invoke("get_volume_status");
            volume = vol.volume;
            isMuted = vol.is_muted;

            powerProfiles = await invoke("get_power_profiles");
            const active = powerProfiles.find((p) => p.active);
            if (active) activeProfileGuid = active.guid;
        } catch (e) {
            console.error("Error fetching control menu data:", e);
        }
    }

    // --- ACCIONES ---
    let brightnessTimer: any;
    async function updateBrightness(val: number) {
        brightness = val;
        if (brightnessTimer) clearTimeout(brightnessTimer);
        brightnessTimer = setTimeout(async () => {
            await invoke("set_brightness", { value: val });
        }, 30);
    }

    let volumeTimer: any;
    async function updateVolume(val: number) {
        volume = val;
        if (volumeTimer) clearTimeout(volumeTimer);
        volumeTimer = setTimeout(async () => {
            await invoke("set_volume", { value: val });
        }, 15);
    }

    async function toggleMute() {
        await invoke("toggle_mute");
        const res: any = await invoke("get_volume_status");
        isMuted = res.is_muted;
    }

    async function setPowerProfile(guid: string) {
        await invoke("set_power_profile", { guidStr: guid });
        activeProfileGuid = guid;
    }

    function openPowerSettings() {
        invoke("interact_app", { hwnd: 0, execPath: "ms-settings:powersleep" });
    }

    onMount(() => {
        fetchData();
        const interval = setInterval(fetchData, 2000);

        let unlistenAnchor: any;
        listen<any>("sync-widget-anchor", (event) => {
            if (event.payload.widgetId !== "battery") return;
            anchorCenterX = event.payload.centerX;
            anchorBottom = event.payload.taskbarBottom;
            monitorWidth = event.payload.monitorWidth;
            if (isVisible) applyWindowPosition();
        }).then((fn) => (unlistenAnchor = fn));

        let unlistenToggle: any;
        listen<boolean>("toggle-battery-menu", (event) => {
            if (event.payload) {
                fetchData();
            }
        }).then((fn) => (unlistenToggle = fn));

        return () => {
            clearInterval(interval);
            if (unlistenAnchor) unlistenAnchor();
            if (unlistenToggle) unlistenToggle();
        };
    });

    function handleMouseEnter() {
        emit("battery-menu-hover", true);
    }

    function handleMouseLeave() {
        emit("battery-menu-hover", false);
    }
</script>

<div
    class="menu-container"
    role="menu"
    tabindex="-1"
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
>
    <!-- Header: Batería -->
    <header class="menu-header">
        <div class="battery-main-info">
            <span class="percentage">{Math.round(battPercent.current)}%</span>
            <span class="status">
                {#if batteryStatus.is_charging}
                    Cargando
                {:else if batteryStatus.battery_saver}
                    Ahorro de batería activo
                {:else}
                    En batería
                {/if}
            </span>
        </div>
        <div class="battery-visual">
            <div class="battery-bar-bg">
                <div
                    class="battery-fill"
                    style="width: {battPercent.current}%; background-color: {batteryStatus.is_charging
                        ? '#22c55e'
                        : batteryStatus.battery_saver
                          ? '#fbbf24'
                          : batteryStatus.percentage <= 20
                            ? '#ef4444'
                            : '#e0e0e0'}"
                ></div>
            </div>
        </div>
    </header>

    <!-- Modos de energía -->
    <div class="power-modes">
        {#each powerProfiles as profile}
            <button
                class="mode-btn"
                class:active={profile.guid === activeProfileGuid}
                onclick={() => setPowerProfile(profile.guid)}
                title={profile.name}
            >
                <span class="icon">
                    {#if profile.name
                        .toLowerCase()
                        .includes("ahorro") || profile.name
                            .toLowerCase()
                            .includes("saver")}
                        {@html saverIcon}
                    {:else if profile.name
                        .toLowerCase()
                        .includes("alto") || profile.name
                            .toLowerCase()
                            .includes("high")}
                        {@html performanceIcon}
                    {:else if profile.name
                        .toLowerCase()
                        .includes("equilibrado") || profile.name
                            .toLowerCase()
                            .includes("balanced")}
                        {@html ecoIcon}
                    {:else if profile.name.toLowerCase().includes("eco")}
                        {@html balancedIcon}
                    {:else}
                        {@html ecoIcon}
                    {/if}
                </span>
            </button>
        {/each}

        <!-- Botón de ajustes si no hay perfiles o como extra -->
        {#if powerProfiles.length === 0}
            <button class="mode-btn wide" onclick={openPowerSettings}>
                <span class="icon">
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
                        <path
                            d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.72V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.17a2 2 0 0 1 1-1.74l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
                        />
                        <circle cx="12" cy="12" r="3" />
                    </svg>
                </span>
                <span class="label">Configuración</span>
            </button>
        {/if}
    </div>

    <!-- Sliders: Brillo y Volumen -->
    <div class="controls-section">
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
                    <circle
                        cx="12"
                        cy="12"
                        r={brightness < 33 ? 3 : brightness < 66 ? 4 : 5}
                    />
                    <g opacity={0.3 + (brightness / 100) * 0.7}>
                        <path d="M12 2v2" /><path d="M12 20v2" />
                        <path d="m4.93 4.93 1.41 1.41" /><path
                            d="m17.66 17.66 1.41 1.41"
                        />
                        <path d="M2 12h2" /><path d="M20 12h2" />
                        <path d="m6.34 17.66-1.41 1.41" /><path
                            d="m19.07 4.93-1.41 1.41"
                        />
                    </g>
                </svg>
            </span>
            <input
                type="range"
                min="0"
                max="100"
                value={brightness}
                oninput={(e) =>
                    updateBrightness(parseInt(e.currentTarget.value))}
                class="control-slider"
                style="--progress: {brightness}%"
            />
            <span class="control-value">{brightness}%</span>
        </div>

        <div class="control-row">
            <button class="control-icon-btn" onclick={toggleMute}>
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
                    {#if isMuted}
                        <path d="M11 4.7L6 9H2v6h4l5 4.3V4.7z" />
                        <line x1="22" y1="9" x2="16" y2="15" />
                        <line x1="16" y1="9" x2="22" y2="15" />
                    {:else}
                        <path d="M11 4.7L6 9H2v6h4l5 4.3V4.7z" />
                        {#if volume >= 0.01}
                            <path
                                d="M15.54 8.46a5 5 0 0 1 0 7.07"
                                opacity={volume < 0.33 ? 0.3 : 1}
                            />
                        {/if}
                        {#if volume >= 0.33}
                            <path
                                d="M19.07 4.93a10 10 0 0 1 0 14.14"
                                opacity={volume < 0.66 ? 0.3 : 1}
                            />
                        {/if}
                    {/if}
                </svg>
            </button>
            <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={volume}
                oninput={(e) => updateVolume(parseFloat(e.currentTarget.value))}
                class="control-slider"
                style="--progress: {volume * 100}%"
            />
            <span class="control-value">{Math.round(volume * 100)}%</span>
        </div>
    </div>

    <!-- Acceso directo a ajustes -->
    <footer class="menu-footer">
        <button class="settings-link" onclick={openPowerSettings}>
            Configuración de energía y batería
        </button>
    </footer>
</div>

<style>
    .menu-container {
        background: rgba(24, 24, 27, 0.85);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 16px;
        overflow: hidden;
        width: 400px;
        padding: 20px;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
        display: flex;
        flex-direction: column;
        gap: 16px;
        color: inherit;
    }

    .menu-header {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .battery-main-info {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
    }

    .percentage {
        font-size: 2rem;
        font-weight: 800;
        letter-spacing: -0.02em;
    }

    .status {
        font-size: 0.85rem;
        opacity: 0.6;
        font-weight: 500;
    }

    .battery-bar-bg {
        width: 100%;
        height: 8px;
        background: rgba(255, 255, 255, 0.08);
        border-radius: 4px;
        overflow: hidden;
    }

    .battery-fill {
        height: 100%;
        transition:
            width 0.8s cubic-bezier(0.4, 0, 0.2, 1),
            background-color 0.6s;
    }

    .power-modes {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
        gap: 8px;
    }

    .mode-btn {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 12px;
        padding: 12px 8px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        color: inherit;
        outline: none;
    }

    .mode-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.15);
        transform: scale(1.06);
    }

    .mode-btn.active {
        background: #3b82f6;
        border-color: #60a5fa;
        color: white;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    }

    .mode-btn .icon {
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .mode-btn .label {
        font-size: 0.7rem;
        font-weight: 600;
        text-align: center;
        max-width: 100%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

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
        transition: all 0.2s;
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
        background: linear-gradient(
            to right,
            #3b82f6 var(--progress),
            rgba(255, 255, 255, 0.1) var(--progress)
        );
        border-radius: 3px;
        outline: none;
        cursor: pointer;
    }

    .control-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 18px;
        height: 18px;
        background: white;
        border-radius: 50%;
        cursor: pointer;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
        transition: transform 0.15s;
    }

    .control-slider::-webkit-slider-thumb:hover {
        transform: scale(1.15);
    }

    .menu-footer {
        margin-top: 4px;
        display: flex;
        justify-content: center;
    }

    .settings-link {
        background: transparent;
        border: none;
        color: #3b82f6;
        font-size: 0.8rem;
        font-weight: 600;
        cursor: pointer;
        padding: 4px 8px;
        border-radius: 4px;
        transition: background 0.2s;
    }

    .settings-link:hover {
        background: rgba(59, 130, 246, 0.1);
        text-decoration: underline;
    }
</style>
