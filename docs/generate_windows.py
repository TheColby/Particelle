import os
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import scipy.special

# Mathematical functions matching particelle-dsp/src/window/generator.rs
def w_rectangular(n, N): return np.ones_like(n)
def w_hann(n, N): return 0.5 * (1.0 - np.cos(2.0*np.pi*n/N))
def w_hamming(n, N): return 0.54 - 0.46 * np.cos(2.0*np.pi*n/N)
def w_blackman(n, N): return 0.42 - 0.5*np.cos(2.0*np.pi*n/N) + 0.08*np.cos(4.0*np.pi*n/N)
def w_blackman_harris(n, N): return 0.35875 - 0.48829*np.cos(2.0*np.pi*n/N) + 0.14128*np.cos(4.0*np.pi*n/N) - 0.01168*np.cos(6.0*np.pi*n/N)
def w_nuttall(n, N): return 0.355768 - 0.487396*np.cos(2.0*np.pi*n/N) + 0.144232*np.cos(4.0*np.pi*n/N) - 0.012604*np.cos(6.0*np.pi*n/N)
def w_blackman_nuttall(n, N): return 0.3635819 - 0.4891775*np.cos(2.0*np.pi*n/N) + 0.1365995*np.cos(4.0*np.pi*n/N) - 0.0106411*np.cos(6.0*np.pi*n/N)
def w_flat_top(n, N): return 0.21557895 - 0.41663158*np.cos(2.0*np.pi*n/N) + 0.27726316*np.cos(4.0*np.pi*n/N) - 0.08357895*np.cos(6.0*np.pi*n/N) + 0.00694737*np.cos(8.0*np.pi*n/N)

def w_bartlett(n, N): return 2.0/N * (N/2.0 - np.abs(n - N/2.0))
def w_bartlett_hann(n, N): return 0.62 - 0.48*np.abs(n/N - 0.5) - 0.38*np.cos(2.0*np.pi*n/N)
def w_cosine(n, N): return np.sin(np.pi*n/N)
def w_lanczos(n, N): 
    t = 2.0*n/N - 1.0
    val = np.sin(np.pi*t)/(np.pi*t)
    val[np.isnan(val)] = 1.0
    return val

def w_gaussian(n, N, sigma=0.4): return np.exp(-0.5 * ((n - N/2.0)/(sigma*N/2.0))**2)
def w_cauchy(n, N, alpha=3.0): return 1.0 / (1.0 + (alpha*(n - N/2.0)/(N/2.0))**2)
def w_poisson(n, N, alpha=2.0): return np.exp(-alpha * np.abs(n - N/2.0)/(N/2.0))
def w_hann_poisson(n, N, alpha=2.0): return w_hann(n, N) * w_poisson(n, N, alpha)
def w_welch(n, N): return 1.0 - ((n - N/2.0)/(N/2.0))**2

def w_bohman(n, N):
    t = np.abs(n - N/2.0)/(N/2.0)
    v = (1.0-t)*np.cos(np.pi*t) + (1.0/np.pi)*np.sin(np.pi*t)
    return np.where(t <= 1.0, v, 0.0)

def w_tukey(n, N, alpha=0.5):
    t = n / N
    left = t < alpha/2.0
    right = t > 1.0 - alpha/2.0
    mid = ~(left | right)
    v = np.zeros_like(t)
    v[left] = 0.5*(1.0 + np.cos(np.pi*(2.0*t[left]/alpha - 1.0)))
    v[right] = 0.5*(1.0 + np.cos(np.pi*(2.0*t[right]/alpha - 2.0/alpha + 1.0)))
    v[mid] = 1.0
    return v

def w_parzen(n, N):
    t = np.abs(n - N/2.0)/(N/2.0)
    v = np.zeros_like(t)
    m1 = t <= 0.5
    m2 = (t > 0.5) & (t <= 1.0)
    v[m1] = 1.0 - 6.0*t[m1]**2 + 6.0*t[m1]**3
    v[m2] = 2.0*(1.0 - t[m2])**3
    return v

def besseli0(x): return scipy.special.i0(x)
def w_kaiser(n, N, beta=14.0):
    alpha = np.abs(np.pi * beta)
    m = N/2.0
    k = 1.0 - ((n - m)/m)**2
    k[k<0] = 0
    return besseli0(alpha * np.sqrt(k)) / besseli0(alpha)

windows = [
    ("Rectangular", w_rectangular),
    ("Lanczos", w_lanczos),
    ("Sine / Cosine", w_cosine),
    ("Bartlett (Triangle)", w_bartlett),
    ("Welch", w_welch),
    ("Hann", w_hann),
    ("Hamming", w_hamming),
    ("Blackman", w_blackman),
    ("Bartlett-Hann", w_bartlett_hann),
    ("Bohman", w_bohman),
    ("Nuttall", w_nuttall),
    ("Blackman-Nuttall", w_blackman_nuttall),
    ("Blackman-Harris", w_blackman_harris),
    ("Flat Top", w_flat_top),
    ("Parzen", w_parzen),
    ("Tukey (alpha=0.5)", w_tukey),
    ("Gaussian (sigma=0.4)", w_gaussian),
    ("Kaiser (beta=14.0)", w_kaiser),
    ("Cauchy (alpha=3.0)", w_cauchy),
    ("Poisson (alpha=2.0)", w_poisson),
]

points = 500
t = np.linspace(0, points-1, points)

fig_cols = 5
fig_rows = 4

plt.rcParams.update({'font.family': 'sans-serif', 'font.sans-serif': ['Helvetica', 'Arial']})
fig, axes = plt.subplots(fig_rows, fig_cols, figsize=(15, 10), facecolor='#ffffff')
axes = axes.flatten()

# Muted slate/indigo color system matching the other graphics
curve_color = '#3f51b5'
fill_color = '#e8eaf6'

for i, (name, func) in enumerate(windows):
    ax = axes[i]
    y = func(t, points-1)
    
    # Plot curve
    ax.plot(t, y, color=curve_color, linewidth=2.0)
    # Fill area under curve
    ax.fill_between(t, 0, y, color=fill_color, alpha=0.8)
    
    ax.set_title(name, fontsize=11, fontweight='bold', pad=8, color='#333333')
    
    # Clean up axes
    ax.set_ylim(-0.05, 1.1)
    ax.set_xlim(0, points)
    
    # Remove borders except bottom
    ax.spines['top'].set_visible(False)
    ax.spines['right'].set_visible(False)
    ax.spines['left'].set_visible(False)
    ax.spines['bottom'].set_color('#dddddd')
    
    ax.tick_params(left=False, bottom=False)
    ax.set_yticks([])
    ax.set_xticks([])
    
plt.tight_layout(pad=3.0)
plt.savefig('/Users/cleider/dev/Particelle/docs/windows_grid.png', dpi=200, bbox_inches='tight')
print("Successfully generated docs/windows_grid.png")
