<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { slide, fade } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import Icon from "$lib/iconMap";

    // --- PROPS ---
    let { 
        wifiActive = $bindable(true), 
        bluetoothActive = $bindable(false),
        onOpenSettings
    } = $props();

    // --- ICONS ---
    const wifiIcon = "fluent:wifi-1-24-filled";
    const bluetoothIcon = "fluent:bluetooth-24-filled";
    const settingsIcon = "fluent:settings-24-filled";

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
        }, 500);
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

    const quickActions = $derived([
        {
            id: "wifi",
            label: "Wi-Fi",
            icon: wifiIcon,
            isActive: wifiActive,
            onClick: async () => {
                wifiActive = !wifiActive;
                try {
                    await invoke("toggle_radio", { kind: "wifi", enable: wifiActive });
                } catch (e) { console.error(e); }
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
                    await invoke("toggle_radio", { kind: "bluetooth", enable: bluetoothActive });
                } catch (e) { console.error(e); }
            },
        },
        {
            id: "settings",
            label: "Configuración",
            icon: settingsIcon,
            isActive: false,
            onClick: onOpenSettings,
        },
    ]);
</script>

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
                <Icon icon={action.icon} />
            </span>
        </button>
    {/each}
</div>

{#if hoveredAction === "wifi" || hoveredAction === "bluetooth"}
    <div class="hover-info-wrapper" transition:slide={{ duration: 400, easing: cubicOut }}>
        <div
            class="hover-info-panel"
            role="tooltip"
            onmouseenter={cancelLeave}
            onmouseleave={handleActionLeave}
        >
            {#key hoveredAction}
                <div in:fade={{ duration: 300 }} out:fade={{ duration: 150 }} class="fade-container">
                    {#if hoveredAction === "wifi"}
                        <div class="hover-list">
                            <div class="list-title">
                                Redes Wi-Fi
                                {#if isLoadingHover}<span class="scanning-text">(Escaneando...)</span>{/if}
                            </div>
                            {#each wifiNetworks as net}
                                <div class="list-item">
                                    <span class="item-icon"><Icon icon={wifiIcon} /></span>
                                    <span class="item-name">{net.ssid}</span>
                                    <span class="item-status">
                                        {#if net.signal_bars >= 4}Excelente{:else if net.signal_bars >= 3}Buena{:else if net.signal_bars >= 2}Media{:else}Débil{/if}
                                    </span>
                                </div>
                            {/each}
                            {#if wifiNetworks.length === 0 && !isLoadingHover}
                                <div class="empty-msg">No se encontraron redes</div>
                            {/if}
                        </div>
                    {:else if hoveredAction === "bluetooth"}
                        <div class="hover-list">
                            <div class="list-title">
                                Dispositivos Bluetooth
                                {#if isLoadingHover}<span class="scanning-text">(Escaneando...)</span>{/if}
                            </div>
                            {#each bluetoothDevices as dev}
                                <div class="list-item">
                                    <span class="item-icon"><Icon icon={bluetoothIcon} /></span>
                                    <span class="item-name">{dev.name}</span>
                                    <span class="item-status" class:connected={dev.is_connected}>
                                        {dev.is_connected ? "Conectado" : "Emparejado"}
                                    </span>
                                </div>
                            {/each}
                            {#if bluetoothDevices.length === 0 && !isLoadingHover}
                                <div class="empty-msg">No hay dispositivos emparejados</div>
                            {/if}
                        </div>
                    {/if}
                </div>
            {/key}
        </div>
    </div>
{/if}

<style>
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
        0%, 100% { opacity: 0.4; }
        50% { opacity: 1; }
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

    .empty-msg {
        font-size: 0.85rem;
        opacity: 0.6;
        text-align: center;
        padding: 20px 0;
    }
</style>
