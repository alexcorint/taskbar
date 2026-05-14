export function marqueeAction(node: HTMLElement, _deps: any) {
  const check = () => {
    const container = node;
    const content = node.querySelector(".marquee-content") as HTMLElement;
    if (!content) return;

    const diff = content.scrollWidth - container.clientWidth;
    if (diff > 0) {
      container.classList.add("is-overflowing");
      const scrollDistance = diff + 20;
      container.style.setProperty("--scroll-distance", `-${scrollDistance}px`);
      const duration = scrollDistance / 15 + 6;
      container.style.setProperty("--scroll-duration", `${duration}s`);
    } else {
      container.classList.remove("is-overflowing");
      container.style.removeProperty("--scroll-distance");
      container.style.removeProperty("--scroll-duration");
    }
  };

  const ro = new ResizeObserver(check);
  ro.observe(node);

  setTimeout(check, 50);

  return {
    update() {
      setTimeout(check, 50);
    },
    destroy() {
      ro.disconnect();
    },
  };
}
