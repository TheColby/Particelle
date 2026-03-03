import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.patches as patches

# Configuration
sample_rate = 44100
duration = 1.0
t = np.linspace(0, duration, int(sample_rate * duration))

# Generate a source waveform (a complex tone/noise mixture)
source = np.sin(2 * np.pi * 110 * t) * np.exp(-t * 2)
source += 0.5 * np.sin(2 * np.pi * 330 * t) * np.exp(-t * 3)
source += 0.2 * np.random.randn(len(t)) * np.exp(-t * 1)
source /= np.max(np.abs(source)) # normalize

# Granular parameters
grain_duration = 0.2  # 200 ms
hop_size = 0.1        # 100 ms (50% overlap)
num_grains = 5

fig, ax = plt.subplots(figsize=(10, 4), facecolor='white')

# Colors
wave_color = '#b0bec5'
window_color = '#3f51b5'
fill_color = '#e8eaf6'
text_color = '#333333'

# Plot the underlying source waveform lightly
ax.plot(t, source, color=wave_color, alpha=0.3, linewidth=1.0)

# Generate and plot grains
for i in range(num_grains):
    start_time = 0.1 + i * hop_size
    end_time = start_time + grain_duration
    
    # Time vector for this grain
    grain_t = np.linspace(start_time, end_time, int(sample_rate * grain_duration))
    
    # Hann window
    window = 0.5 * (1.0 - np.cos(2.0 * np.pi * (grain_t - start_time) / grain_duration))
    
    # Plot window envelope
    ax.plot(grain_t, window, color=window_color, linewidth=2.0)
    ax.fill_between(grain_t, 0, window, color=fill_color, alpha=0.4)
    
    # Plot windowed waveform
    # We need to extract the corresponding source segment
    start_idx = int(start_time * sample_rate)
    end_idx = start_idx + len(grain_t)
    if end_idx <= len(source):
        windowed_source = source[start_idx:end_idx] * window
        ax.plot(grain_t, windowed_source, color=window_color, alpha=0.8, linewidth=1.0)

# Add clear annotations for Hop Size and Grain Duration

# 1. Grain Duration Annotations (Show on the first grain)
g1_start = 0.1
g1_end = 0.1 + grain_duration
ax.annotate('', xy=(g1_start, -0.6), xytext=(g1_end, -0.6),
            arrowprops=dict(arrowstyle='<->', color='#d32f2f', lw=1.5))
ax.text((g1_start + g1_end) / 2, -0.75, 'Grain Duration', ha='center', va='center', 
        color='#d32f2f', fontweight='bold', fontsize=10)

# 2. Hop Size Annotation (Show between first and second grain start times)
g2_start = g1_start + hop_size
ax.annotate('', xy=(g1_start, 1.15), xytext=(g2_start, 1.15),
            arrowprops=dict(arrowstyle='<->', color='#388e3c', lw=1.5))
ax.text((g1_start + g2_start) / 2, 1.30, 'Hop Size', ha='center', va='center', 
        color='#388e3c', fontweight='bold', fontsize=10)

# Vertical dotted lines to lock in the annotations visually
ax.axvline(g1_start, ymin=0.1, ymax=0.9, color='#d32f2f', linestyle=':', alpha=0.5)
ax.axvline(g1_end, ymin=0.1, ymax=0.4, color='#d32f2f', linestyle=':', alpha=0.5)
ax.axvline(g2_start, ymin=0.6, ymax=0.9, color='#388e3c', linestyle=':', alpha=0.5)

# Formatting
ax.set_ylim(-1.0, 1.5)
ax.set_xlim(0.0, 0.8)
ax.set_axis_off()  # Remove axes for clean look

plt.title("Granular Synthesis: Overlapping Windows", fontsize=14, fontweight='bold', color=text_color, pad=20)

plt.tight_layout()
plt.savefig('/Users/cleider/dev/Particelle/docs/grain_hopping_windows.png', dpi=200, bbox_inches='tight')
print("Successfully generated docs/grain_hopping_windows.png")
