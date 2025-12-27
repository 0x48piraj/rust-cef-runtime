window.addEventListener("DOMContentLoaded", () => {
  const fps = document.getElementById("fps") as HTMLElement;
  const box = document.getElementById("box") as HTMLElement;

  const boxSize = 120;
  let x = 100;
  let y = 100;
  let vx = 3;
  let vy = 2;

  let lastFps = performance.now();
  let frames = 0;

  function animate(time: number) {
    frames++;

    if (time - lastFps >= 1000) {
      fps.textContent = `FPS: ${frames}`;
      frames = 0;
      lastFps = time;
    }

    x += vx;
    y += vy;

    // Bounce off edges
    if (x <= 0 || x + boxSize >= window.innerWidth) vx *= -1;
    if (y <= 0 || y + boxSize >= window.innerHeight) vy *= -1;

    box.style.transform = `translate(${x}px, ${y}px)`;

    requestAnimationFrame(animate);
  }

  requestAnimationFrame(animate);
});
