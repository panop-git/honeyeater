from pathlib import Path
import numpy as np
import scipy
from scipy.signal import sosfilt

# Stores generated oracle vectors into tests/vectors directory
refVector = Path(__file__).resolve().parents[3] / "crates/honeyeater-core/tests/vectors/biquad_filters"
refVector.mkdir(parents=True, exist_ok=True)
(refVector / "lpf").mkdir(parents=True, exist_ok=True)

vecLength = [8, 16, 64]
sample_rate = 44100.0
cutoff_freq = 1000.0
q = 1.0

# Define the RBJ lowpass formula in Python to generate the exact same coefficients
def compute_rbj_lpf_sos(sample_rate, cutoff_freq, q):
    w0 = 2.0 * np.pi * cutoff_freq / sample_rate
    alpha = np.sin(w0) / (2.0 * q)
    cos_w0 = np.cos(w0)
    
    b0 = (1.0 - cos_w0) / 2.0
    b1 = 1.0 - cos_w0
    b2 = (1.0 - cos_w0) / 2.0
    a0 = 1.0 + alpha
    a1 = -2.0 * cos_w0
    a2 = 1.0 - alpha
    
    # Format required by scipy.signal.sosfilt: [[b0/a0, b1/a0, b2/a0, 1.0, a1/a0, a2/a0]]
    return np.array([[b0/a0, b1/a0, b2/a0, 1.0, a1/a0, a2/a0]])

sos = compute_rbj_lpf_sos(sample_rate, cutoff_freq, q)

# Loop through each vector length, run impulse response through sosfilt, and save
for n in vecLength:
    x = np.zeros(n)
    x[0] = 1.0 # Impulse input matching Rust test
    y = sosfilt(sos, x)
    np.save(refVector / "lpf" / f"lpf_{n}.npy", y)

print(f"Wrote {len(vecLength)} vectors to {refVector / 'lpf'}")
print(f"scipy {scipy.__version__}, numpy {np.__version__}")