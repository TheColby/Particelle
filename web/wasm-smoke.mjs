import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const bytes = await readFile(new URL('./pkg/obelisk_wasm.wasm', import.meta.url));
const { instance } = await WebAssembly.instantiate(bytes);
const api = instance.exports;
assert.equal(api.obelisk_abi_version(), 1);
assert.equal(api.obelisk_render(48000, 60, 0.25, 0.7, 0.2, 0.1, 0.2), 0);

const length = api.obelisk_output_len();
const pointer = api.obelisk_output_ptr();
assert.equal(length, 108000 * 2);
assert.ok(pointer + length * 4 <= api.memory.buffer.byteLength);

const audio = new Float32Array(api.memory.buffer, pointer, length);
assert.ok(audio.every(Number.isFinite));
assert.ok(audio.some(sample => Math.abs(sample) > 0.001));
api.obelisk_clear();
assert.equal(api.obelisk_output_len(), 0);
console.log('WASM M0 runtime produced finite, nonzero stereo audio.');
