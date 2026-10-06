import init, { initThreadPool, ShredderEngine, get_current_threads } from './pkg/color_shredder_wasm.js';

let wasmModule = null;
let engine = null;
let isRunning = false;
let isFinished = false;
let totalPixels = 0;
let uncapped = true;
let batchSpeed = 500;
let lastTickTime = 0;
let threadPoolInitialized = false;

self.onmessage = async function(e) {
  const msg = e.data;
  if (msg.cmd === 'init') {
    isRunning = false;
    isFinished = false;
    totalPixels = msg.width * msg.height;
    uncapped = msg.uncapped;
    batchSpeed = msg.batchSpeed;

    if (!wasmModule) {
      wasmModule = await init();
    }

    // Initialize Rayon thread pool with detected/requested threads
    let initError = null;
    if (!threadPoolInitialized && msg.threads > 1) {
      try {
        console.log(`[Worker] Initializing Rayon thread pool with ${msg.threads} threads...`);
        await initThreadPool(msg.threads);
        threadPoolInitialized = true;
        console.log(`[Worker] Rayon thread pool initialized successfully!`);
      } catch (err) {
        initError = String(err && err.stack ? err.stack : err);
        console.error('Rayon thread pool initialization failed:', err);
      }
    }

    engine = new ShredderEngine(
      msg.width,
      msg.height,
      msg.startX,
      msg.startY,
      msg.strategy,
      msg.bitDepth,
      msg.colorSpace,
      msg.fixedChannel,
      msg.shuffle,
      msg.seed,
      msg.useRStar
    );

    const activeThreads = get_current_threads();
    console.log(`[Worker] ShredderEngine initialized. Active Rayon threads: ${activeThreads}`);
    self.postMessage({
      cmd: 'ready',
      activeThreads,
      initError
    });

    lastTickTime = performance.now();
    sendFrame(1, 8, false);
  } else if (msg.cmd === 'start') {
    if (!isRunning && !isFinished && engine) {
      isRunning = true;
      runLoop();
    }
  } else if (msg.cmd === 'pause') {
    isRunning = false;
  } else if (msg.cmd === 'setTuning') {
    uncapped = msg.uncapped;
    batchSpeed = msg.batchSpeed;
  }
};

function sendFrame(placed, frontier, finished) {
  if (!engine || !wasmModule) return;
  const ptr = engine.get_rgba_ptr();
  const len = engine.get_rgba_len();
  // Slice out the RGBA bytes from WebAssembly shared memory to transfer safely
  const pixels = new Uint8Array(wasmModule.memory.buffer, ptr, len).slice();
  self.postMessage({
    cmd: 'frame',
    placed,
    frontier,
    finished,
    pixels
  }, [pixels.buffer]);
}

function runLoop() {
  if (!isRunning || isFinished || !engine) return;

  while (isRunning && !isFinished) {
    const steps = uncapped ? 250 : batchSpeed;
    const stats = engine.step(steps);

    if (stats.is_finished || engine.is_finished()) {
      isFinished = true;
      isRunning = false;
      sendFrame(stats.pixels_placed, stats.pixels_available, true);
      return;
    }

    const now = performance.now();
    if (uncapped) {
      // 30 FPS throttle prevents browser GC thrashing with multi-megabyte frame transfers
      if (now - lastTickTime >= 33) {
        sendFrame(stats.pixels_placed, stats.pixels_available, false);
        lastTickTime = now;
        setTimeout(runLoop, 0);
        return;
      }
    } else {
      sendFrame(stats.pixels_placed, stats.pixels_available, false);
      lastTickTime = now;
      setTimeout(runLoop, 16);
      return;
    }
  }
}
