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

> **Note:** To run these optimally, compile the engine in `--release` mode.
