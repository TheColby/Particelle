let enginePromise;

async function loadEngine() {
  if (!enginePromise) {
    enginePromise = (async () => {
      const response = await fetch(new URL('./pkg/obelisk_wasm.wasm', import.meta.url));
      if (!response.ok) throw new Error(`WASM download failed (${response.status}). Run scripts/build_wasm.sh first.`);
      const bytes = await response.arrayBuffer();
      const { instance } = await WebAssembly.instantiate(bytes);
      const api = instance.exports;
      if (api.obelisk_abi_version() !== 1) throw new Error('Unsupported M0 WASM ABI. Rebuild the browser package.');
      return api;
    })().catch(error => {
      enginePromise = undefined;
      throw error;
    });
  }
  return enginePromise;
}

async function render(id, settings) {
  const api = await loadEngine();
  const result = api.obelisk_render(48000, settings.root, settings.duration,
    settings.morph, settings.motion, settings.delay, settings.reverb);
  if (result !== 0) throw new Error(result === 1 ? 'Invalid render settings.' : 'M0 rejected these controls.');
  const length = api.obelisk_output_len();
  const pointer = api.obelisk_output_ptr();
  if (!length || length % 2 || pointer + length * 4 > api.memory.buffer.byteLength) {
    api.obelisk_clear();
    throw new Error('M0 returned an invalid stereo buffer.');
  }
  const samples = new Float32Array(api.memory.buffer, pointer, length).slice();
  api.obelisk_clear();
  if (!samples.every(Number.isFinite)) throw new Error('M0 returned non-finite audio.');
  self.postMessage({ type: 'rendered', id, samples }, [samples.buffer]);
}

self.onmessage = async event => {
  const { type, id, settings } = event.data;
  try {
    if (type === 'load') {
      await loadEngine();
      self.postMessage({ type: 'ready' });
    } else if (type === 'render') {
      await render(id, settings);
    }
  } catch (error) {
    self.postMessage({ type: 'error', id, message: error instanceof Error ? error.message : String(error) });
  }
};
