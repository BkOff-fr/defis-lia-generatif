/**
 * Observe an element with IntersectionObserver, call cb the first time it
 * enters the viewport, then disconnect. Used to trigger fade-in animations
 * on scroll without ScrollTrigger overhead.
 */
export function observeOnce(
  el: Element,
  cb: () => void,
  options: IntersectionObserverInit = { threshold: 0.15 },
): () => void {
  if (typeof IntersectionObserver === 'undefined') {
    cb();
    return () => undefined;
  }
  const io = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        cb();
        io.disconnect();
        break;
      }
    }
  }, options);
  io.observe(el);
  return () => io.disconnect();
}
