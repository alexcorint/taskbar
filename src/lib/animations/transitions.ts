import { fly, fade, scale, slide } from "svelte/transition";
import { DURATIONS } from "./constants";
import { EASINGS } from "./easings";

/**
 * Configuraciones predeterminadas para transiciones comunes.
 * Úsalas como: transition:fly={TRANSITIONS.FLY_IN}
 */
export const TRANSITIONS = {
    // Entrada suave desde abajo
    FLY_IN: {
        duration: DURATIONS.NORMAL,
        y: 20,
        easing: EASINGS.CUBIC_OUT
    },
    
    // Desvanecimiento suave
    FADE_QUICK: {
        duration: DURATIONS.FAST,
        easing: EASINGS.CUBIC_OUT
    },

    // Escalado tipo "Pop" (ideal para modales o tooltips)
    POP: {
        duration: DURATIONS.NORMAL,
        start: 0.9,
        easing: EASINGS.BACK_OUT
    }
} as const;

/**
 * Transición personalizada avanzada: Desenfoque + Fly
 * Úsala como: in:blurFly={{ y: 20 }}
 */
export function blurFly(node: HTMLElement, { 
    delay = 0, 
    duration = DURATIONS.NORMAL, 
    easing = EASINGS.CUBIC_OUT, 
    x = 0, 
    y = 0, 
    blur = 10 
}) {
    return {
        delay,
        duration,
        easing,
        css: (t: number) => `
            transform: translate(${(1 - t) * x}px, ${(1 - t) * y}px);
            opacity: ${t};
            filter: blur(${(1 - t) * blur}px);
        `
    };
}
