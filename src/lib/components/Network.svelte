<script lang="ts">
    import { network } from "$lib/stores/system";
    import { fade } from "svelte/transition";
    import Icon from "$lib/iconMap";

    const isEthernet = $derived($network.connection_type === "ethernet");

    const iconName = $derived.by(() => {
        if (!$network.is_online) return "fluent:wifi-off-24-filled";
        if (isEthernet) return "fluent:connector-24-filled";
        
        const strength = $network.signal_strength;
        if (strength >= 4) return "fluent:wifi-4-24-filled";
        if (strength >= 3) return "fluent:wifi-3-24-filled";
        if (strength >= 2) return "fluent:wifi-2-24-filled";
        return "fluent:wifi-1-24-filled";
    });
</script>

<div
    class="network-container"
    title={$network.is_online
        ? `Conectado (${isEthernet ? "Ethernet" : "Wi-Fi"})`
        : "Desconectado"}
>
    <div class="icon-wrapper" transition:fade={{ duration: 150 }}>
        <Icon icon={iconName} width="20" height="20" />
    </div>
</div>

<style>
    .network-container {
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
