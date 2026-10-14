"""
Generate SciPy reference vectors for Honeyeater FIR coefficient design.

This generator tests window-method FIR design, not FIR execution. SciPy's
scipy.signal.firwin is used as the independent coefficient-design oracle.
Parks-McClellan design is checked separately against a spectral mask in Rust.

Generate oracle vectors using the pinned Python environment defined in
tools/oracle-gen/requirements.txt.
"""

from pathlib import Path

import numpy as np
import scipy
from scipy.signal import firwin


repo_root = Path(__file__).resolve().parents[3]

ref_vector = (
        repo_root
        / "crates"
        / "honeyeater-core"
        / "tests"
        / "vectors"
        / "fir_design"
)

ref_vector.mkdir(parents=True, exist_ok=True)

sample_rate = 1_000_000.0

# Reference taps for the four window-method designs exercised by fir_design.rs.
lowpass_hamming_63 = firwin(
    63,
    100_000.0,
    window="hamming",
    pass_zero="lowpass",
    scale=True,
    fs=sample_rate,
).astype(np.float64)

highpass_kaiser_63 = firwin(
    63,
    150_000.0,
    window=("kaiser", 7.5),
    pass_zero="highpass",
    scale=True,
    fs=sample_rate,
).astype(np.float64)

bandpass_hann_64 = firwin(
    64,
    [100_000.0, 200_000.0],
    window="hann",
    pass_zero="bandpass",
    scale=True,
    fs=sample_rate,
).astype(np.float64)

bandstop_blackmanharris_63 = firwin(
    63,
    [100_000.0, 200_000.0],
    window="blackmanharris",
    pass_zero="bandstop",
    scale=True,
    fs=sample_rate,
).astype(np.float64)

np.save(ref_vector / "lowpass_hamming_63.npy", lowpass_hamming_63)
np.save(ref_vector / "highpass_kaiser_63.npy", highpass_kaiser_63)
np.save(ref_vector / "bandpass_hann_64.npy", bandpass_hann_64)
np.save(ref_vector / "bandstop_blackmanharris_63.npy", bandstop_blackmanharris_63)


print(f"Wrote FIR-design oracle vectors to {ref_vector}")
print(f"scipy {scipy.__version__}, numpy {np.__version__}")
