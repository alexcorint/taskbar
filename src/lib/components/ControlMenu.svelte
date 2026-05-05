<script module>
    const performanceIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M5.6 18.4 A 9 9 0 1 1 18.4 18.4" /><line x1="12" y1="3" x2="12" y2="5" /><line x1="3" y1="12" x2="5" y2="12" /><line x1="21" y1="12" x2="19" y2="12" /><line x1="5.6" y1="5.6" x2="7" y2="7" /><line x1="18.4" y1="5.6" x2="17" y2="7" /><line x1="12" y1="12" x2="17" y2="17" /><circle cx="12" cy="12" r="1.5" /></svg>`;
    const balancedIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="4" x2="12" y2="20" /><line x1="8" y1="20" x2="16" y2="20" /><circle cx="12" cy="5" r="1.5" /><line x1="4" y1="5" x2="20" y2="5" /><polygon points="4,5 1,14 7,14" /><polygon points="20,5 17,14 23,14" /></svg>`;
    const saverIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 3v18M21 9a9 9 0 1 1-18 0"/><path d="M15 9h6M3 9h6"/><path d="M3.75 14.25a6.75 6.75 0 0 1 13.5 0"/></svg>`;
    const ecoIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><g transform="translate(12, 12)"><g transform="rotate(0)"><path d="M 0,-11 C 5,-11 8,-6 5,-1 C 0,-3 -3,-7 0,-11 Z" /><path d="M 0,-11 Q 3,-7 4,-2" /></g><g transform="rotate(120)"><path d="M 0,-11 C 5,-11 8,-6 5,-1 C 0,-3 -3,-7 0,-11 Z" /><path d="M 0,-11 Q 3,-7 4,-2" /></g><g transform="rotate(240)"><path d="M 0,-11 C 5,-11 8,-6 5,-1 C 0,-3 -3,-7 0,-11 Z" /><path d="M 0,-11 Q 3,-7 4,-2" /></g></g></svg>`;
    const wifiIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/></svg>`;
    const bluetoothIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6.5 6.5 11 11L12 23V1l5.5 5.5-11 11"/></svg>`;
    const settingsIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`;
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit, listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import { slide, fade } from "svelte/transition";
    import { battery, volume, setVolumeImmediate, toggleMuteImmediate } from "$lib/stores/system";
    import type { PowerProfile, BatteryInfo, VolumeInfo } from "$lib/types";

    // --- PROPS ---
    let { isVisible = false } = $props();

    // --- ESTADO ---
    // battery y volume vienen del store centralizado — sin polling propio
    const battPercent = new Tween(100, { duration: 800, easing: cubicOut });

    let brightness = $state(50);
    let powerProfiles = $state<PowerProfile[]>([]);
    let activeProfileGuid = $state("");

    // Sincronizar Tween con el store de batería
    $effect(() => {
        battPercent.set($battery.percentage);
    });

    // Variable local para el slider de volumen — necesaria para que Svelte 5
    // actualice el <input range> cuando el store cambia externamente
    let volumeValue = $state($volume.volume);
    $effect(() => {
        volumeValue = $volume.volume;
    });

    // --- ACCIONES RÁPIDAS (Quick Settings) ---
    let wifiActive = $state(true);
    let bluetoothActive = $state(false);

    let quickActions = $derived([
        {
            id: "wifi",
            label: "Wi-Fi",
            icon: wifiIcon,
            isActive: wifiActive,
            onClick: async () => {
                wifiActive = !wifiActive;
                try {
                    await invoke("toggle_radio", {
                        kind: "wifi",
                        enable: wifiActive,
                    });
                } catch (e) {
                    console.error(e);
                }
            },
        },
        {
            id: "bluetooth",
            label: "Bluetooth",
            icon: bluetoothIcon,
            isActive: bluetoothActive,
            onClick: async () => {
                bluetoothActive = !bluetoothActive;
                try {
                    await invoke("toggle_radio", {
                        kind: "bluetooth",
                        enable: bluetoothActive,
                    });
                } catch (e) {
                    console.error(e);
                }
            },
        },
        {
            id: "settings",
            label: "Configuración",
            icon: settingsIcon,
            isActive: false,
            onClick: openPowerSettings,
        },
    ]);

    let hoveredAction = $state<string | null>(null);
    let wifiNetworks = $state<any[]>([]);
    let bluetoothDevices = $state<any[]>([]);
    let isLoadingHover = $state(false);
    let hoverTimeout: any;
    let enterTimeout: any;

    async function handleActionEnter(id: string) {
        if (enterTimeout) clearTimeout(enterTimeout);

        if (id !== "wifi" && id !== "bluetooth") {
            hoveredAction = null;
            return;
        }

        if (hoverTimeout) clearTimeout(hoverTimeout);

        enterTimeout = setTimeout(async () => {
            hoveredAction = id;
            isLoadingHover = true;

            try {
                if (id === "wifi") {
                    wifiNetworks = await invoke("get_wifi_networks");
                } else if (id === "bluetooth") {
                    bluetoothDevices = await invoke("get_bluetooth_devices");
                }
            } catch (e) {
                console.error("Error fetching " + id, e);
            } finally {
                isLoadingHover = false;
            }
        }, 500); // Pequeño timeout (200ms) para evitar aperturas accidentales
    }

    function handleActionLeave() {
        if (enterTimeout) clearTimeout(enterTimeout);
        if (hoverTimeout) clearTimeout(hoverTimeout);
        hoverTimeout = setTimeout(() => {
            hoveredAction = null;
        }, 300);
    }

    function cancelLeave() {
        if (hoverTimeout) clearTimeout(hoverTimeout);
    }

    // --- POSICIONAMIENTO ---
    let anchorCenterX = 0;
    let anchorBottom = 0;
    let monitorWidth = 0;

    async function applyWindowPosition() {
        if (anchorCenterX === 0) return;

        const w = 450;
        const h = 550; // Altura aumentada para acomodar el panel desplegable sin recortes

        let logicalX = anchorCenterX - w / 2;
        const margin = 10;
        if (logicalX < margin) logicalX = margin;
        if (monitorWidth > 0 && logicalX + w > monitorWidth - margin) {
            logicalX = monitorWidth - w - margin;
        }

        const logicalY = anchorBottom - 48 - h - 10;

        await invoke("manage_window", {
            label: "control_menu",
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
    // Brillo, radios y perfiles se fetchean manualmente. 
    // Batería y volumen vienen del store (sincronizado), pero forzamos un fetch inicial para evitar desincronización.
    async function fetchData() {
        try {
            if (isVisible) {
                // Brillo (local al componente)
                brightness = await invoke("get_brightness");

                // Radios
                const radioStates: any = await invoke("get_radio_states");
                wifiActive = radioStates.wifi;
                bluetoothActive = radioStates.bluetooth;

                // Perfiles de energía
                powerProfiles = await invoke<PowerProfile[]>("get_power_profiles");
                const active = powerProfiles.find((p) => p.active);
                if (active) activeProfileGuid = active.guid;

                // Forzar actualización del store centralizado (esto disparará el evento de sync a otras ventanas también)
                // Usamos invoke directamente para actualizar el store local
                const b = await invoke<BatteryInfo>("get_battery_status");
                battery.set(b);
                
                const v = await invoke<VolumeInfo>("get_volume_status");
                volume.set(v);
            }
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

    // Volumen y mute usan las acciones del store (feedback optimista inmediato)
    async function updateVolume(val: number) {
        await setVolumeImmediate(val);
    }

    async function toggleMute() {
        await toggleMuteImmediate();
    }

    async function setPowerProfile(guid: string) {
        await invoke("set_power_profile", { guidStr: guid });
        activeProfileGuid = guid;
    }

    async function openPowerSettings() {
        try {
            await invoke("interact_app", { hwnd: 0, execPath: "ms-settings:powersleep" });
        } catch (e) {
            console.error(e);
        }
    }

    onMount(() => {
        // Carga inicial — brillo y radios cuando el componente monta
        fetchData();

        let unlistenAnchor: (() => void) | undefined;
        listen<any>("sync-widget-anchor", (event) => {
            if (event.payload.widgetId !== "battery") return;
            anchorCenterX = event.payload.centerX;
            anchorBottom = event.payload.taskbarBottom;
            monitorWidth = event.payload.monitorWidth;
            if (isVisible) applyWindowPosition();
        }).then((fn) => (unlistenAnchor = fn));

        let unlistenToggle: (() => void) | undefined;
        listen<boolean>("toggle-control-menu", (event) => {
            if (event.payload) fetchData(); // Re-fetch al abrir el menú
        }).then((fn) => (unlistenToggle = fn));

        return () => {
            unlistenAnchor?.();
            unlistenToggle?.();
        };
    });

    function handleMouseEnter() {
        emit("control-menu-hover", true);
    }

    function handleMouseLeave() {
        emit("control-menu-hover", false);
    }
</script>

<div
    class="menu-container"
    role="menu"
    tabindex="-1"
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
>
    <!-- Header: Batería (desde store centralizado) -->
    <header class="menu-header">
        <div class="battery-main-info">
            <span class="percentage">{Math.round(battPercent.current)}%</span>
            <span class="status">
                {#if $battery.is_charging}
                    Cargando
                {:else if $battery.battery_saver}
                    Ahorro de batería activo
                {:else}
                    En batería
                {/if}
            </span>
        </div>
    </header>

    <!-- Modos de energía -->
    <!-- Acciones Rápidas (Grid dinámico) -->
    <div class="quick-actions-grid">
        {#each quickActions as action}
            <button
                class="action-btn"
                class:active={action.isActive}
                onclick={action.onClick}
                onmouseenter={() => handleActionEnter(action.id)}
                onmouseleave={handleActionLeave}
                title={action.label}
            >
                <span class="icon">
                    {@html action.icon}
                </span>
            </button>
        {/each}
    </div>

    <!-- Panel de información al hacer hover -->
    <!-- Panel de información al hacer hover -->
    {#if hoveredAction === "wifi" || hoveredAction === "bluetooth"}
        <div
            class="hover-info-wrapper"
            transition:slide={{ duration: 400, easing: cubicOut }}
        >
            <div
                class="hover-info-panel"
                role="tooltip"
                onmouseenter={cancelLeave}
                onmouseleave={handleActionLeave}
            >
                {#key hoveredAction}
                    <div
                        in:fade={{ duration: 300 }}
                        out:fade={{ duration: 150 }}
                        class="fade-container"
                    >
                        {#if hoveredAction === "wifi"}
                            <div class="hover-list">
                                <div class="list-title">
                                    Redes Wi-Fi
                                    {#if isLoadingHover}<span
                                            class="scanning-text"
                                            >(Escaneando...)</span
                                        >{/if}
                                </div>
                                {#each wifiNetworks as net}
                                    <div class="list-item">
                                        <span class="item-icon"
                                            >{@html wifiIcon}</span
                                        >
                                        <span class="item-name">{net.ssid}</span
                                        >
                                        <span class="item-status">
                                            {#if net.signal_bars >= 4}Excelente{:else if net.signal_bars >= 3}Buena{:else if net.signal_bars >= 2}Media{:else}Débil{/if}
                                        </span>
                                    </div>
                                {/each}
                                {#if wifiNetworks.length === 0 && !isLoadingHover}
                                    <div class="empty-msg">
                                        No se encontraron redes
                                    </div>
                                {/if}
                                {#if wifiNetworks.length === 0 && isLoadingHover}
                                    <div class="loading-msg">
                                        Buscando redes...
                                    </div>
                                {/if}
                            </div>
                        {:else if hoveredAction === "bluetooth"}
                            <div class="hover-list">
                                <div class="list-title">
                                    Dispositivos Bluetooth
                                    {#if isLoadingHover}<span
                                            class="scanning-text"
                                            >(Escaneando...)</span
                                        >{/if}
                                </div>
                                {#each bluetoothDevices as dev}
                                    <div class="list-item">
                                        <span class="item-icon"
                                            >{@html bluetoothIcon}</span
                                        >
                                        <span class="item-name">{dev.name}</span
                                        >
                                        <span
                                            class="item-status"
                                            class:connected={dev.is_connected}
                                        >
                                            {dev.is_connected
                                                ? "Conectado"
                                                : "Emparejado"}
                                        </span>
                                    </div>
                                {/each}
                                {#if bluetoothDevices.length === 0 && !isLoadingHover}
                                    <div class="empty-msg">
                                        No hay dispositivos emparejados
                                    </div>
                                {/if}
                                {#if bluetoothDevices.length === 0 && isLoadingHover}
                                    <div class="loading-msg">
                                        Buscando dispositivos...
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    </div>
                {/key}
            </div>
        </div>
    {/if}

    <!-- Sliders: Brillo y Volumen -->
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
                bind:value={brightness}
                oninput={(e) =>
                    updateBrightness(parseInt(e.currentTarget.value))}
                class="control-slider"
                style="--progress: {brightness}%"
            />
            <span class="control-value">{brightness}%</span>
        </div>

        <!-- Volumen: variable local sincronizada con el store para reactividad correcta -->
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
                    {#if $volume.is_muted}
                        <path d="M11 4.7L6 9H2v6h4l5 4.3V4.7z" />
                        <line x1="22" y1="9" x2="16" y2="15" />
                        <line x1="16" y1="9" x2="22" y2="15" />
                    {:else}
                        <path d="M11 4.7L6 9H2v6h4l5 4.3V4.7z" />
                        {#if $volume.volume >= 0.01}
                            <path
                                d="M15.54 8.46a5 5 0 0 1 0 7.07"
                                opacity={$volume.volume < 0.33 ? 0.3 : 1}
                            />
                        {/if}
                        {#if $volume.volume >= 0.33}
                            <path
                                d="M19.07 4.93a10 10 0 0 1 0 14.14"
                                opacity={$volume.volume < 0.66 ? 0.3 : 1}
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
                bind:value={volumeValue}
                oninput={(e) => updateVolume(parseFloat(e.currentTarget.value))}
                class="control-slider"
                style="--progress: {volumeValue * 100}%"
            />
            <span class="control-value">{Math.round(volumeValue * 100)}%</span>
        </div>
    </div>

    <!-- Perfiles de energía -->
    {#if powerProfiles.length > 0}
        <div class="power-profiles">
            <div class="section-title">Modo de energía</div>
            <div class="profiles-grid">
                {#each powerProfiles as profile}
                    <button
                        class="profile-btn"
                        class:active={profile.guid === activeProfileGuid}
                        onclick={() => setPowerProfile(profile.guid)}
                        title={profile.name}
                    >
                        <span class="profile-icon">
                            {#if profile.name.toLowerCase().includes('alto rendimiento') || profile.name.toLowerCase().includes('high')}
                                {@html performanceIcon}
                            {:else if profile.name.toLowerCase().includes('ahorro') || profile.name.toLowerCase().includes('saver') || profile.name.toLowerCase().includes('power saver')}
                                {@html saverIcon}
                            {:else if profile.name.toLowerCase().includes('eco')}
                                {@html ecoIcon}
                            {:else}
                                {@html balancedIcon}
                            {/if}
                        </span>
                        <span class="profile-name">{profile.name}</span>
                    </button>
                {/each}
            </div>
        </div>
    {/if}

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
        position: absolute;
        bottom: 0;
        left: 50%;
        transform: translateX(-50%);
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

    .quick-actions-grid {
        display: grid;
        grid-template-columns: repeat(5, 1fr);
        gap: 10px;
        margin-bottom: 12px;
    }

    .action-btn {
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        aspect-ratio: 1 / 1;
        cursor: pointer;
        transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
        color: inherit;
        outline: none;
    }

    .action-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(255, 255, 255, 0.15);
        transform: scale(1.05);
    }

    .action-btn.active {
        background: #3b82f6;
        border-color: #60a5fa;
        color: white;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    }

    .action-btn .icon {
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .action-btn .icon :global(svg) {
        width: 100%;
        height: 100%;
    }

    .hover-info-wrapper {
        overflow: hidden;
    }

    .hover-info-panel {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        padding: 12px;
        margin-bottom: 2px;
        height: 220px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
    }

    .hover-info-panel::-webkit-scrollbar {
        width: 4px;
    }

    .hover-info-panel::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.2);
        border-radius: 2px;
    }

    .hover-list {
        display: flex;
        flex-direction: column;
        flex-grow: 1;
    }

    .fade-container {
        display: grid;
        grid-template-areas: "stack";
        flex-grow: 1;
        width: 100%;
    }

    .fade-container > div {
        grid-area: stack;
    }

    .scanning-text {
        font-size: 0.75rem;
        text-transform: none;
        opacity: 0.7;
        font-weight: normal;
        animation: pulse 1.5s infinite;
        margin-left: 4px;
    }

    @keyframes pulse {
        0%,
        100% {
            opacity: 0.4;
        }
        50% {
            opacity: 1;
        }
    }

    .list-title {
        font-size: 0.8rem;
        font-weight: 700;
        opacity: 0.6;
        margin-bottom: 8px;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        padding-left: 4px;
    }

    .list-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 8px 10px;
        border-radius: 8px;
        transition: background 0.2s;
        cursor: pointer;
    }

    .list-item:hover {
        background: rgba(255, 255, 255, 0.08);
    }

    .item-icon {
        width: 16px;
        height: 16px;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0.8;
    }

    .item-icon :global(svg) {
        width: 100%;
        height: 100%;
    }

    .item-name {
        font-size: 0.9rem;
        font-weight: 500;
        flex-grow: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .item-status {
        font-size: 0.75rem;
        opacity: 0.5;
        font-weight: 500;
    }

    .item-status.connected {
        color: #60a5fa;
        opacity: 1;
        font-weight: 700;
    }

    .loading-msg,
    .empty-msg {
        font-size: 0.85rem;
        opacity: 0.6;
        text-align: center;
        padding: 20px 0;
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

    /* Perfiles de energía */
    .power-profiles {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .section-title {
        font-size: 0.75rem;
        font-weight: 600;
        opacity: 0.5;
        text-transform: uppercase;
        letter-spacing: 0.08em;
    }

    .profiles-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 8px;
    }

    .profile-btn {
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 6px;
        padding: 10px 6px;
        cursor: pointer;
        color: inherit;
        transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
        outline: none;
    }

    .profile-btn:hover {
        background: rgba(255, 255, 255, 0.09);
        border-color: rgba(255, 255, 255, 0.12);
    }

    .profile-btn.active {
        background: rgba(59, 130, 246, 0.2);
        border-color: #3b82f6;
        color: #60a5fa;
    }

    .profile-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 20px;
        height: 20px;
    }

    .profile-icon :global(svg) {
        width: 20px;
        height: 20px;
    }

    .profile-name {
        font-size: 0.68rem;
        font-weight: 500;
        text-align: center;
        line-height: 1.2;
        max-width: 100%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
</style>
