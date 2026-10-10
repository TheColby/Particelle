import assert from 'node:assert/strict';
import test from 'node:test';
import { encodeWav, formatTime } from './audio.mjs';

test('encodes a valid little-endian stereo PCM16 WAV', () => {
  const view = new DataView(encodeWav(new Float32Array([-1, 1, 0, .5])));
  const tag = offset => String.fromCharCode(...new Uint8Array(view.buffer, offset, 4));
  assert.equal(tag(0), 'RIFF');
  assert.equal(tag(8), 'WAVE');
  assert.equal(tag(12), 'fmt ');
  assert.equal(tag(36), 'data');
  assert.equal(view.getUint32(4, true), 44);
  assert.equal(view.getUint16(20, true), 1);
  assert.equal(view.getUint16(22, true), 2);
  assert.equal(view.getUint32(24, true), 48000);
  assert.equal(view.getUint32(28, true), 192000);
  assert.equal(view.getUint16(32, true), 4);
  assert.equal(view.getUint16(34, true), 16);
  assert.equal(view.getUint32(40, true), 8);
  assert.equal(view.getInt16(44, true), -32768);
  assert.equal(view.getInt16(46, true), 32767);
  assert.equal(view.getInt16(48, true), 0);
  assert.equal(view.getInt16(50, true), 16384);
});

test('rejects malformed sample buffers', () => {
  assert.throws(() => encodeWav(new Float32Array(0)), TypeError);
  assert.throws(() => encodeWav(new Float32Array(1)), TypeError);
});

test('formats elapsed seconds', () => {
  assert.equal(formatTime(65.9), '1:05');
});
