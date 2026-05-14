<script lang="ts">
    import { battery } from "$lib/stores/system";
    import { anchor } from "$lib/actions/anchor";
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";
    import { fade } from "svelte/transition";
    import Icon from "$lib/iconMap";

    // Usar Svelte 5 Tween para consistencia con el resto de la app
    const battPercent = new Tween(100, { duration: 800, easing: cubicOut });

    let prevCharging = $state(false);

    // Reactivo al store centralizado usando $effect de Svelte 5
    $effect(() => {
        const currentBattery = $battery;
        battPercent.set(currentBattery.percentage);
        prevCharging = currentBattery.is_charging;
    });

    let iconColor = $derived($battery.is_charging
        ? "#22c55e"
        : $battery.battery_saver
          ? "#fbbf24"
          : battPercent.current <= 20
            ? "#ef4444"
            : "currentColor");

    const iconName = $derived.by(() => {
        if ($battery.is_charging) return "fluent:battery-charge-24-filled";
        if ($battery.percentage <= 10) return "fluent:battery-warning-24-filled";
        
        // Mapear 0-100 a 0-10
        const level = Math.floor($battery.percentage / 10);
        return `fluent:battery-${level}-24-filled`;
    });
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
    <div class="icon-wrapper" style="color: {iconColor}" transition:fade={{ duration: 150 }}>
        <Icon icon={iconName} width="20" height="20" />
        <span class="percentage-text">{Math.round(battPercent.current)}%</span>
    </div>
</div>

<style>
    .battery-container {
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .icon-wrapper {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
    }

    .percentage-text {
        font-size: 0.75rem;
        font-weight: 600;
        min-width: 28px;
        text-align: left;
    }
</style>
