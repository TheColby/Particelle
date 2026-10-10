#!/usr/bin/env python3
"""Verify the book's examples against a fresh offline CLI build (standard library only)."""
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
import wave

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'target/userguide-qa'
BIN = ROOT / 'target/debug/particelle'


def run(*args, stdin=None):
    return subprocess.run([str(x) for x in args], cwd=ROOT, input=stdin,
                          text=True, check=True, capture_output=True).stdout


def measure(path, expected_channels, seconds):
    with wave.open(str(path), 'rb') as audio:
        channels, frames, rate, width = (audio.getnchannels(), audio.getnframes(),
                                        audio.getframerate(), audio.getsampwidth())
        data = audio.readframes(frames)
    assert channels == expected_channels, (path, channels)
    assert frames == int(rate * seconds), (path, frames)
    assert width == 3, (path, width)
    peak = [0.0] * channels
    sums = [0.0] * channels
    for i, pos in enumerate(range(0, len(data), width)):
        sample = int.from_bytes(data[pos:pos + width], 'little', signed=True) / 8388608.0
        assert math.isfinite(sample)
        ch = i % channels
        peak[ch] = max(peak[ch], abs(sample))
        sums[ch] += sample * sample
    assert max(peak) > 1e-6, f'Silent output: {path}'
    return {'file': str(path.relative_to(ROOT)), 'frames': frames, 'sample_rate': rate,
            'channels': channels, 'peak': peak,
            'rms': [math.sqrt(value / frames) for value in sums],
            'sha256': hashlib.sha256(data).hexdigest()}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    text = (ROOT / 'docs/USERGUIDE.md').read_text()
    for name, content in re.findall(r'```(?:yaml|json) file=(examples/userguide/[\w.-]+)\n(.*?)\n```', text, re.S):
        assert (ROOT / name).read_text() == content + '\n', f'Book/example mismatch: {name}'
    subprocess.run(['cargo', 'build', '-p', 'particelle-cli', '--no-default-features'], cwd=ROOT, check=True)
    metrics = []
    for name in ['basic', 'moving', 'scan', 'follower', 'quad', 'control']:
        patch = ROOT / 'examples/userguide' / f'{name}.yaml'
        output = OUT / f'{name}.wav'
        run(BIN, 'validate', patch)
        run(BIN, 'render', patch, '-o', output, '--duration', '2', '--format', 'pcm24')
        result = measure(output, 4 if name == 'quad' else 2, 2)
        assert all(value > 1e-6 for value in result['peak']), f'Inactive teaching channel: {name}'
        metrics.append(result)
    repeated = OUT / 'basic-repeat.wav'
    run(BIN, 'render', 'examples/userguide/basic.yaml', '-o', repeated,
        '--duration', '2', '--format', 'pcm24')
    assert repeated.read_bytes() == (OUT / 'basic.wav').read_bytes(), 'Repeated render changed'
    starter = run(BIN, 'init')
    run(BIN, 'validate', '-', stdin=starter)
    output = OUT / 'starter.wav'
    run(BIN, 'render', '-', '-o', output, '--duration', '2', '--format', 'pcm24', stdin=starter)
    metrics.append(measure(output, 2, 2))
    for channels in [1, 2, 8]:
        patch = run(BIN, 'preset', 'dronoify', 'audio/music_example.wav',
                    '--channels', channels, '--amplitude', '0.06')
        run(BIN, 'validate', '-', stdin=patch)
        output = OUT / f'dronoify-{channels}.wav'
        run(BIN, 'render', '-', '-o', output, '--duration', '2', '--format', 'pcm24', stdin=patch)
        metrics.append(measure(output, channels, 2))
    curve = run(BIN, 'curve', 'examples/userguide/scan.json', '--resolution', '17')
    (OUT / 'scan.tsv').write_text(curve)
    (OUT / 'report.json').write_text(json.dumps(metrics, indent=2) + '\n')
    print(f'Passed: {len(metrics)} measured renders, repeatability, stdin, curve, and book/example consistency.')
    print(f'Report: {OUT / "report.json"}')


if __name__ == '__main__':
    main()
