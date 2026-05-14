<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Icon from '$lib/iconMap';

  let time = '';
  let date = '';
  let interval: ReturnType<typeof setInterval>;

  function updateClock() {
    const now = new Date();
    // Formateamos la hora (ej: 20:11)
    time = now.toLocaleTimeString('es-ES', { hour: '2-digit', minute: '2-digit' });
    // Formateamos la fecha (ej: 03/04/26)
    date = now.toLocaleDateString('es-ES', { day: '2-digit', month: '2-digit', year: '2-digit' });
  }

  onMount(() => {
    updateClock(); // Llamada inicial
    interval = setInterval(updateClock, 1000); // Actualizar cada segundo
  });

  onDestroy(() => {
    clearInterval(interval); // Limpiar memoria si el componente se destruye
  });
</script>

<div class="clock">
  <div class="icon-wrapper">
    <Icon icon="fluent:clock-24-filled" width="20" height="20" />
  </div>
  <div class="text-container">
    <span class="time">{time}</span>
    <span class="date">{date}</span>
  </div>
</div>

<style>
  .clock {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    font-weight: bold;
    line-height: 1.1;
  }
  .icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .text-container {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }
  .date {
    font-size: 0.75rem;
    color: #a1a1aa;
    font-weight: normal;
  }
</style>