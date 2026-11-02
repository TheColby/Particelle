import { encodeWav, formatTime, SAMPLE_RATE } from './audio.mjs';

const $ = id => document.getElementById(id);
const worker = new Worker(new URL('./render-worker.mjs', import.meta.url), { type: 'module' });
const form = $('render-form');
const renderButton = $('render');
const playButton = $('play');
const downloadButton = $('download');
const canvas = $('waveform');
const status = $('status');
const sliders = ['morph', 'motion', 'delay', 'reverb'];
let ready = false;
let pending = false;
let currentId = 0;
let samples = null;
let audioContext = null;
let source = null;
let startedAt = 0;
let pausedAt = 0;
let animationFrame = 0;

function setStatus(message, error = false) {
  status.textContent = message;
  status.classList.toggle('error', error);
}

function updateControls() {
  renderButton.disabled = !ready || pending;
  renderButton.setAttribute('aria-busy', pending ? 'true' : 'false');
  renderButton.firstElementChild.textContent = pending ? 'Rendering…' : 'Render passage';
  playButton.disabled = !samples || pending;
  downloadButton.disabled = !samples || pending;
}

function duration() { return samples ? samples.length / 2 / SAMPLE_RATE : 0; }
function updateProgress() {
  const position = source && audioContext ? Math.min(duration(), audioContext.currentTime - startedAt) : pausedAt;
  $('time').textContent = `${formatTime(position)} / ${formatTime(duration())}`;
  $('progress').style.width = duration() ? `${position / duration() * 100}%` : '0%';
  if (source) animationFrame = requestAnimationFrame(updateProgress);
}

function stopPlayback(reset = true) {
  if (source) {
    const old = source;
    source = null;
    old.onended = null;
    old.stop();
    old.disconnect();
  }
  cancelAnimationFrame(animationFrame);
  pausedAt = reset ? 0 : pausedAt;
  $('play-icon').textContent = '▶';
  playButton.setAttribute('aria-label', 'Play rendered passage');
  updateProgress();
}

async function togglePlayback() {
  if (!samples) return;
  if (source) {
    pausedAt = Math.min(duration(), audioContext.currentTime - startedAt);
    stopPlayback(false);
    return;
  }
  try {
    audioContext ??= new AudioContext();
    await audioContext.resume();
    if (pausedAt >= duration()) pausedAt = 0;
    const frames = samples.length / 2;
    const buffer = audioContext.createBuffer(2, frames, SAMPLE_RATE);
    const left = buffer.getChannelData(0);
    const right = buffer.getChannelData(1);
    for (let i = 0; i < frames; i++) {
      left[i] = samples[i * 2];
      right[i] = samples[i * 2 + 1];
    }
    const node = audioContext.createBufferSource();
    node.buffer = buffer;
    node.connect(audioContext.destination);
    node.onended = () => {
      if (source !== node) return;
      source = null;
      node.disconnect();
      pausedAt = 0;
      cancelAnimationFrame(animationFrame);
      $('play-icon').textContent = '▶';
      playButton.setAttribute('aria-label', 'Play rendered passage');
      updateProgress();
    };
    source = node;
    startedAt = audioContext.currentTime - pausedAt;
    node.start(0, pausedAt);
    $('play-icon').textContent = 'Ⅱ';
    playButton.setAttribute('aria-label', 'Pause rendered passage');
    updateProgress();
  } catch (error) {
    stopPlayback();
    setStatus(`Playback unavailable: ${error.message}`, true);
  }
}

function drawWaveform() {
  const bounds = canvas.getBoundingClientRect();
  if (!bounds.width || !bounds.height) return;
  const dpr = Math.min(devicePixelRatio || 1, 2);
  canvas.width = Math.round(bounds.width * dpr);
  canvas.height = Math.round(bounds.height * dpr);
  const context = canvas.getContext('2d');
  if (!samples) return;
  const width = canvas.width;
  const height = canvas.height;
  const center = height / 2;
  const usable = height * .37;
  context.strokeStyle = '#d7b789';
  context.lineWidth = Math.max(1, dpr);
  context.beginPath();
  for (let x = 0; x < width; x++) {
    const start = Math.floor(x / width * samples.length / 2);
    const end = Math.max(start + 1, Math.floor((x + 1) / width * samples.length / 2));
    let peak = 0;
    for (let frame = start; frame < end; frame++) {
      peak = Math.max(peak, Math.abs(samples[frame * 2]), Math.abs(samples[frame * 2 + 1]));
    }
    const amplitude = Math.max(1, Math.min(1, peak) * usable);
    context.moveTo(x + .5, center - amplitude);
    context.lineTo(x + .5, center + amplitude);
  }
  context.stroke();
}

for (const id of sliders) {
  const input = $(id);
  input.addEventListener('input', () => {
    $(`${id}-value`).textContent = Number(input.value).toFixed(2) + (id === 'motion' ? ' Hz' : '');
  });
}

form.addEventListener('submit', event => {
  event.preventDefault();
  if (!ready || pending) return;
  stopPlayback();
  samples = null;
  drawWaveform();
  $('visual-overlay').hidden = false;
  $('visual-overlay').textContent = 'Rendering your passage…';
  $('transport-title').textContent = 'Rendering passage';
  $('transport-detail').textContent = 'The M0 engine is working locally.';
  pending = true;
  updateControls();
  setStatus('Synthesizing audio in the background…');
  worker.postMessage({ type: 'render', id: ++currentId, settings: {
    root: Number($('root').value), duration: Number($('duration').value),
    morph: Number($('morph').value), motion: Number($('motion').value),
    delay: Number($('delay').value), reverb: Number($('reverb').value),
  } });
});

worker.onmessage = ({ data }) => {
  if (data.type === 'ready') {
    ready = true;
    setStatus('Engine ready. Adjust the controls and render.');
  } else if (data.type === 'rendered' && data.id === currentId) {
    samples = data.samples;
    pending = false;
    $('visual-overlay').hidden = true;
    $('transport-title').textContent = 'M0 passage';
    $('transport-detail').textContent = `${formatTime(duration())} · stereo · 48 kHz`;
    setStatus('Render complete. Listen here or download a WAV.');
    drawWaveform();
    updateProgress();
  } else if (data.type === 'error' && (!data.id || data.id === currentId)) {
    pending = false;
    $('visual-overlay').textContent = 'Render unavailable';
    $('transport-title').textContent = 'No render available';
    $('transport-detail').textContent = 'Check the status below.';
    setStatus(data.message, true);
  }
  updateControls();
};
worker.onerror = () => {
  pending = false;
  ready = false;
  setStatus('The render worker failed. Reload the page and try again.', true);
  updateControls();
};

playButton.addEventListener('click', togglePlayback);
downloadButton.addEventListener('click', () => {
  if (!samples) return;
  try {
    const url = URL.createObjectURL(new Blob([encodeWav(samples)], { type: 'audio/wav' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = `obelisk-m0-${$('root').selectedOptions[0].textContent.split(' ')[0].toLowerCase()}.wav`;
    document.body.append(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 30000);
  } catch (error) {
    setStatus(`WAV export failed: ${error.message}`, true);
  }
});
new ResizeObserver(drawWaveform).observe(canvas);
updateControls();
worker.postMessage({ type: 'load' });
