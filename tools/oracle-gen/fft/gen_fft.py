"""
Script generates oracle test values for FFT operations from scipy.

NOTE: Ensure Python virtual environment is active, and all required imports have been installed from the requirements.txt file.
"""

from pathlib import Path
import numpy as np
import scipy
from scipy.fft import fft, ifft

# Stores generated oracle vectors into tests/vectors directory
refVector = Path("../../crates/honeyeater-core/tests/vectors/fft")
refVector.mkdir(parents=True, exist_ok=True)

vecLength = [8, 16, 64]

for n in vecLength:
    # Create a time array (defaults to float64)
    t = np.linspace(0, 1, n, endpoint=False)
    
    # Generate complex signal (defaults to complex128 / Complex<f64>)
    signal = np.sin(2 * np.pi * 2 * t) + 1j * np.cos(2 * np.pi * 3 * t)
    
    # Compute FFT and IFFT
    fft_res = fft(signal)
    ifft_res = ifft(fft_res)

    # Save vectors to disk
    np.save(refVector / f"input_{n}.npy", signal)
    np.save(refVector / f"fft_{n}.npy", fft_res)
    np.save(refVector / f"ifft_{n}.npy", ifft_res)

print(f"Wrote {3 * len(vecLength)} vectors to {refVector}")
print(f"scipy {scipy.__version__}, numpy {np.__version__}")