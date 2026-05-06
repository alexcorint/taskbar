<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { emit, listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import { battery, volume, setVolumeImmediate, toggleMuteImmediate } from "$lib/stores/system";
    import type { BatteryInfo, VolumeInfo } from "$lib/types";

    import QuickActions from "./ControlMenu/QuickActions.svelte";
    import ControlSliders from "./ControlMenu/ControlSliders.svelte";

    // --- PROPS ---
    let { isVisible = false } = $props();

    // --- ESTADO ---
    const battPercent = new Tween(100, { duration: 800, easing: cubicOut });
    let brightness = $state(50);
    let wifiActive = $state(true);
    let bluetoothActive = $state(false);

    $effect(() => {
        battPercent.set($battery.percentage);
    });

    let volumeValue = $state($volume.volume);
    $effect(() => {
        volumeValue = $volume.volume;
    });

    // --- POSICIONAMIENTO ---
    let anchorCenterX = 0;
    let anchorBottom = 0;
    let monitorWidth = 0;

    async function applyWindowPosition() {
        if (anchorCenterX === 0) return;
        const w = 450;
        const h = 550;
        let logicalX = anchorCenterX - w / 2;
        const margin = 10;
        if (logicalX < margin) logicalX = margin;
        if (monitorWidth > 0 && logicalX + w > monitorWidth - margin) {
            logicalX = monitorWidth - w - margin;
        }
        const logicalY = anchorBottom - 48 - h - 10;
        await invoke("manage_window", {
            label: "control_menu",
            action: { type: "updatelogical", payload: { x: logicalX, y: logicalY, w, h } },
        });
    }

    $effect(() => {
        if (isVisible && anchorCenterX !== 0) {
            applyWindowPosition();
        }
    });

    async function fetchData() {
        try {
            if (isVisible) {
                brightness = await invoke("get_brightness");
                const radioStates: any = await invoke("get_radio_states");
                wifiActive = radioStates.wifi;
                bluetoothActive = radioStates.bluetooth;

                const b = await invoke<BatteryInfo>("get_battery_status");
                battery.set(b);
                const v = await invoke<VolumeInfo>("get_volume_status");
                volume.set(v);
            }
        } catch (e) {
            console.error("Error fetching control menu data:", e);
        }
    }

    async function updateBrightness(val: number) {
        brightness = val;
        await invoke("set_brightness", { value: val });
    }

    async function openPowerSettings() {
        try {
            await invoke("interact_app", { hwnd: 0, execPath: "ms-settings:powersleep" });
        } catch (e) { console.error(e); }
    }

    onMount(() => {
        fetchData();
        listen<any>("sync-widget-anchor", (event) => {
            if (event.payload.widgetId !== "battery") return;
            anchorCenterX = event.payload.centerX;
            anchorBottom = event.payload.taskbarBottom;
            monitorWidth = event.payload.monitorWidth;
            if (isVisible) applyWindowPosition();
        });
        listen<boolean>("toggle-control-menu", (event) => {
            if (event.payload) fetchData();
        });
    });

    function handleMouseEnter() { emit("control-menu-hover", true); }
    function handleMouseLeave() { emit("control-menu-hover", false); }
</script>

<div
    class="menu-container"
    role="menu"
    tabindex="-1"
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
>
    <header class="menu-header">
        <div class="battery-main-info">
            <span class="percentage">{Math.round(battPercent.current)}%</span>
            <span class="status">
                {#if $battery.is_charging} Cargando
                {:else if $battery.battery_saver} Ahorro de batería activo
                {:else} En batería {/if}
            </span>
        </div>
    </header>

    <QuickActions 
        bind:wifiActive 
        bind:bluetoothActive 
        onOpenSettings={openPowerSettings} 
    />

    <ControlSliders 
        bind:brightness 
        bind:volumeValue 
        {updateBrightness} 
    />

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

    .menu-footer {
        padding-top: 12px;
        border-top: 1px solid rgba(255, 255, 255, 0.08);
        display: flex;
        justify-content: center;
    }

    .settings-link {
        background: transparent;
        border: none;
        color: #3b82f6;
        font-size: 0.85rem;
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
