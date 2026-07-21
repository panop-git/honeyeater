"""
Script generates oracle test values for hanning windows from scipy.

NOTE: Ensure Python virtual environment is active, and all required imports have been installed from the requirements.txt file.
"""

from pathlib import Path
import numpy as np
import scipy
from scipy.signal.windows import hann
from scipy.signal.windows import hamming

# Stores generated oracle vectors into tests/vectors directory for later use.
refVector = Path("../../../crates/honeyeater-core/tests/vectors/windows")
refVector.mkdir(parents=True, exist_ok=True)

# Create the specific sub-directories for different window types
(refVector / "hann").mkdir(parents=True, exist_ok=True)
(refVector / "hamming").mkdir(parents=True, exist_ok=True)

# Various floating point values to test against.
vecLength = [8, 16, 64];

# Following for loop runs for each vecLength value, calls scipy hann and hamming functions (for both symmetric and periodic), then saves to disk in .npy format.
for n in vecLength:
    np.save(refVector / "hann" / f"hann_{n}.npy", hann(n, sym=True))
    np.save(refVector / "hann" / f"hann_periodic_{n}.npy", hann(n, sym=False))
    np.save(refVector / "hamming" / f"hamming_{n}.npy", hamming(n, sym=True))
    np.save(refVector / "hamming" / f"hamming_periodic_{n}.npy", hamming(n, sym=False))

print(f"wrote {2 * len(vecLength)} vectors to {refVector}")
print(f"scipy {scipy.__version__}, numpy {np.__version__}")