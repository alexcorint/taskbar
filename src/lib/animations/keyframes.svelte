<style>
    /*
     * Keyframes globales para uso directo en CSS/Estilos.
     * Importa este componente una sola vez en +page o +layout.
     */

    /* Efecto de levitación suave (ideal para iconos) */
    @keyframes -global-float {
        0%, 100% { transform: translateY(0); }
        50% { transform: translateY(-4px); }
    }

    /* Pulsación de opacidad y escala (ideal para indicadores activos) */
    @keyframes -global-pulse-soft {
        0% { transform: scale(1); opacity: 0.8; }
        50% { transform: scale(1.05); opacity: 1; }
        100% { transform: scale(1); opacity: 0.8; }
    }

    /* Efecto de brillo/resplandor (Glassmorphism) */
    @keyframes -global-glow {
        0%, 100% { filter: drop-shadow(0 0 2px rgba(255, 255, 255, 0.1)); }
        50% { filter: drop-shadow(0 0 8px rgba(255, 255, 255, 0.4)); }
    }

    /* Efecto de carga/skeleton (Shimmer) */
    @keyframes -global-shimmer {
        0% { background-position: -200% 0; }
        100% { background-position: 200% 0; }
    }

    /* Entrada suave desde abajo (Slide Up) */
    @keyframes -global-slide-up {
        from { transform: translateY(20px); opacity: 0; }
        to { transform: translateY(0); opacity: 1; }
    }
</style>
