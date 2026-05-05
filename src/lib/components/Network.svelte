<script lang="ts">
    import { onMount } from "svelte";
    import { fade } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";

    let isOnline = $state(true);
    let connectionInfo = $state({
        type: "unknown",
        effectiveType: "unknown",
        signalStrength: 0,
    });

    async function updateNetworkStatus() {
        try {
            const status: any = await invoke("get_network_status");
            isOnline = status.is_online;
            connectionInfo = {
                type: status.connection_type,
                effectiveType: "unknown", // No disponible directamente así
                signalStrength: status.signal_strength,
            };
        } catch (e) {
            console.error("Error fetching native network status:", e);
            isOnline = navigator.onLine;
            const conn = (navigator as any).connection;
            if (conn) {
                connectionInfo = {
                    type: conn.type || "unknown",
                    effectiveType: conn.effectiveType || "unknown",
                    signalStrength: 0,
                };
            }
        }
    }

    onMount(() => {
        updateNetworkStatus();
        const interval = setInterval(updateNetworkStatus, 5000);

        return () => {
            clearInterval(interval);
        };
    });

    const isEthernet = $derived(connectionInfo.type === "ethernet");
</script>

<div
    class="network-container"
    title={isOnline
        ? `Conectado (${isEthernet ? "Ethernet" : "Wi-Fi"})`
        : "Desconectado"}
>
    <svg
        viewBox="0 0 24 24"
        width="1.2em"
        height="1.2em"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
    >
        {#if !isOnline}
            <!-- Icono de red desconectada -->
            <g transition:fade={{ duration: 150 }}>
                <path d="M1 1l22 22M16.72 11.06A10.94 10.94 0 0 1 19 12.55" />
                <path d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39" />
                <path d="M10.71 5.05A16 16 0 0 1 22.58 9" />
                <path d="M1.42 9a15.91 15.91 0 0 1 4.7-2.88" />
                <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
                <line x1="12" y1="20" x2="12.01" y2="20" />
            </g>
        {:else if isEthernet}
            <!-- Icono de Ethernet -->
            <g transition:fade={{ duration: 150 }}>
                <!-- Pantalla del monitor -->
                <path d="M9 11v5h12V4H9" />
                <!-- Base del monitor -->
                <path d="M15 16v5 M11 21h8" />
                <!-- Caja del puerto Ethernet conectada al monitor -->
                <rect x="3" y="4" width="6" height="7" />
                <!-- Detalle interno del conector (RJ45) -->
                <path d="M4.5 4v2.5h3V4" />
                <!-- Cable de red bajando -->
                <path d="M6 11v10" />
            </g>
        {:else}
            <!-- Icono de Wi-Fi dinámico -->
            <g transition:fade={{ duration: 150 }}>
                <!-- Punto de origen (Wi-Fi) -->
                <circle
                    cx="19"
                    cy="19"
                    r="2"
                    fill="currentColor"
                    stroke="none"
                    opacity={connectionInfo.signalStrength >= 1 ? 1 : 0.2}
                />

                <!-- Ondas de señal adaptadas a la intensidad -->
                <path
                    d="M14 19 A 5 5 0 0 1 19 14"
                    opacity={connectionInfo.signalStrength >= 2 ? 1 : 0.2}
                />
                <path
                    d="M9 19 A 10 10 0 0 1 19 9"
                    opacity={connectionInfo.signalStrength >= 3 ? 1 : 0.2}
                />
                <path
                    d="M4 19 A 15 15 0 0 1 19 4"
                    opacity={connectionInfo.signalStrength >= 4 ? 1 : 0.2}
                />
            </g>
        {/if}
    </svg>
</div>

<style>
    .network-container {
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
    }
</style>
