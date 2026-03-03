# Complex High-Compute Stress Tests

This directory contains advanced patches specifically engineered to stress-test the Particelle engine architecture. They push boundaries across routing, parameter computation, allocator endurance, and analysis fetching.

### 1. `max_density_swarm.yaml`
Designed to load test the `GrainPool` lifecycle manager and the 3D Amplitude Panner at extreme throughput.
- **Goal:** Sustains an emission rate of 1000 grains per second across an 8-channel immersive layout.
- **Test:** Ensures atomic lock-free drop performance and confirms `block_size` buffer mixing handles massive overlap without CPU spikes.

### 2. `hyper_mod_matrix.yaml`
Designed to benchmark the `ParamSignal` AST node evaluation logic.
- **Goal:** Heavily nests math operations (`add`, `mul`, `clamp`) mapped to dense LFO shapes (`sine`, `square`, `triangle`) running at high frequency.
- **Test:** Proves that the compiled AST evaluator runs allocations-free within the hot loop and scales correctly even when every parameter dimension is fully automated.

### 3. `multi_extractor_orchestra.yaml`
Designed to load test the `particelle-analysis` offline vector buffer fetching.
- **Goal:** Employs three simultaneous granular clouds, each cross-modulating its properties using three *different* `extractor` vectors (YIN F0 Tracker, Spectral Flux, Spectral Entropy).
- **Test:** Confirms that `AnalysisBuffer` atomic pointer references and inner-loop linear interpolation perform perfectly under heavy realtime load.

### 4. `dxd_384khz_64ch.yaml`
Designed to max out multicore Apple Silicon (M-series) bandwidth.
- **Goal:** Runs the engine at 384,000 Hz (DXD standard) internally, pushing granular synthesis across a massive 64-channel layout.
- **Test:** Simulates extreme memory bandwidth pressure and confirms that `f64` buffer allocations for 64 simultaneous channels do not bottleneck the lock-free audio thread at microsecond deadlines.

### 5. `multi_cloud_saturation.yaml`
Designed to torture the interpolation logic and phase synchronization.
- **Goal:** Spawns 12 entirely independent grain clouds playing simultaneously, each with distinct modulators for density, duration, and position.
- **Test:** Forces the CPU to track 12 disparate stream states and overlap-add them simultaneously into the same shared spatial buffers.

> **Note:** To run these optimally, compile the engine in `--release` mode.
