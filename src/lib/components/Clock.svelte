<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

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
  <span class="time">{time}</span>
  <span class="date">{date}</span>
</div>

<style>
  .clock {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: 0.85rem;
    font-weight: bold;
  }
  .date {
    font-size: 0.75rem;
    color: #a1a1aa;
    font-weight: normal;
  }
</style>