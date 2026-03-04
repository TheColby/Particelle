# Particelle: Catalog of Complexity

The `examples/complex/` directory houses a suite of advanced patches designed to theoretically and practically stress-test the Particelle synthetic engine. As a zero-allocation, lock-free, 64-bit engine, Particelle is capable of scaling to algorithmic extremes that crash traditional audio environments. 

This catalog serves as a technical whitepaper detailing the engineering intent behind each patch.

---

## 🌪️ The Stress Tests

### 1. `dxd_384khz_64ch.yaml`
**Category:** Format & Bandwidth Limit Testing
This patch pushes the `f64` rendering kernel to its physical I/O limit on Apple Silicon architectures. It spins up a simulated 64-channel continuous spherical array and commands the engine to synthesize grains at **384 kHz** (the [DXD standard](https://en.wikipedia.org/wiki/Digital_eXtreme_Definition)). 
- **Theoretical Load:** 50 overlapping grains per second mapped across 64 discrete physical channels equalling $\sim 3,200$ simultaneous voices evaluating at $384,000$ frames per second lock-free. 
- **Purpose:** Verifies thread-safety, memory bandwidth, [SIMD auto-vectorization](https://en.wikipedia.org/wiki/Single_instruction,_multiple_data) boundaries, and [cache-line](https://en.wikipedia.org/wiki/CPU_cache) invalidation under maximum Duplex or File I/O duress.

### 2. `multi_cloud_saturation.yaml`
**Category:** Recursive Instantiation (Memory Footprint)
Spawns 32 entirely independent `Cloud` instances simultaneously. All 32 clouds attempt to read overlapping segments of the exact same stereo audio file while maintaining entirely separate lock-free Panner states, Window Caches, and Grain Pools.
- **Purpose:** Forces the CPU to aggressively context-switch between 32 different spatializer traits and state arrays per frame. Validates the `Arc` reference counting of the central memory-mapped acoustic buffers against memory leaks or locking contention.

### 3. `hyper_mod_matrix.yaml`
**Category:** AST Control Rate Limits
A highly dense modulation graph evaluating mathematical nodes (`add`, `mul`, `osc`, `clamp`) natively on the audio thread.
- **Purpose:** Tests the `ParamSignal` [Abstract Syntax Tree (AST)](https://en.wikipedia.org/wiki/Abstract_syntax_tree) pointer chasing speed. Since parameters are evaluated dynamically, creating deeply nested, non-linear algebraic trees verifies that the engine does not underrun the audio buffer due to [branch-prediction](https://en.wikipedia.org/wiki/Branch_predictor) failures.

### 4. `multi_extractor_orchestra.yaml`
**Category:** Offline Feature Extraction Mapping
Demonstrates Particelle's analytical capability. It extracts offline frequency tracking ($f_0$ via [YIN](https://audition.ens.fr/adc/pdf/2002_JASA_YIN.pdf) or [HPS](https://en.wikipedia.org/wiki/Harmonic_product_spectrum) algorithms) from five separate flute samples and maps those pitch vectors dynamically to the playback head positions of five separate string ensembles.
- **Purpose:** Proves the bridging capabilities between the `particelle-analysis` crate and the real-time continuous rendering engine.

### 5. `directional_shimmer.yaml`
**Category:** Continuous Acoustic Anisotropy
Replaces the traditional granular isotropic point-source model with continuous directivity geometry. Grains are assigned spatial rotation $(\text{az}, \text{el})$ using stochastic models, and their radiation patterns are morphing dynamically between omnidirectional and dipole (figure-8).
- **Purpose:** Computes the cardioid attenuation boundary $G = \max(0, \delta + (1 - \delta) \cos(\theta))$ continuously. Creates mathematically rigorous "shimmering" derived entirely from acoustic physics $\cos(\theta)$ instead of DSP modulation filters.

### 6. `stochastic_brownian.yaml` & `chaos_lorenz.yaml`
**Category:** Deterministic Chaos & Stochastic Math
A demonstration of maintaining stateful recursive algorithms completely lock-free inside the inner audio loop. 
- **Purpose:** The [Lorenz attractor](https://en.wikipedia.org/wiki/Lorenz_system) evaluates three coupled differential equations step-by-step per frame without mutex locking. [Brownian motion](https://en.wikipedia.org/wiki/Brownian_motion) executes a continuous stochastic walk $\mathcal{N}(0, \sigma^2 dt)$ for infinite analog drift.

---

## 🛠️ Verification
All examples in this directory are integrated into the CLI's schema validation tests. To verify the architectural integrity of the engine against these stress tests locally, run:

```bash
cargo run --release -- validate examples/complex/dxd_384khz_64ch.yaml
```
