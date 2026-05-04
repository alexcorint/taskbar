<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { anchor } from "$lib/actions/anchor";

  // El componente ahora es mucho más simple ya que la lógica de posición está en la acción
</script>

<!-- EL FANTASMA: Se queda siempre en el flujo de la barra de tareas ocupando 300px -->
<div class="media-ghost-slot" use:anchor={"media"}></div>

<style>
  .media-ghost-slot {
    position: relative;
    width: 300px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
