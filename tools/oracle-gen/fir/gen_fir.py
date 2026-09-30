"""
Generate SciPy reference vectors for Honeyeater FIR execution.

This generator tests FIR execution only, not FIR design. The coefficients are
fixed explicitly and scipy.signal.lfilter is used as the independent execution
oracle.

Generate oracle vectors using the pinned Python environment defined in
tools/oracle-gen/requirements.txt.
"""

from pathlib import Path

import numpy as np
import scipy
from scipy.signal import lfilter


repo_root = Path(__file__).resolve().parents[3]

ref_vector = (
        repo_root
        / "crates"
        / "honeyeater-core"
        / "tests"
        / "vectors"
        / "fir"
)

ref_vector.mkdir(parents=True, exist_ok=True)

vec_lengths = [8, 16, 64]

# Explicit coefficients keep this an execution test rather than a FIR-design
# test. They form a short symmetric low-pass-like FIR with unity DC gain.
taps_f64 = np.array(
    [
        -0.0125,
        0.0,
        0.075,
        0.25,
        0.375,
        0.25,
        0.075,
        0.0,
        -0.0125,
    ],
    dtype=np.float64,
)

taps_f32 = taps_f64.astype(np.float32)

np.save(ref_vector / "taps_f64.npy", taps_f64)
np.save(ref_vector / "taps_f32.npy", taps_f32)


for n in vec_lengths:
    t = np.arange(n, dtype=np.float64) / n

    # Deterministic complex RF-style signal. Peak magnitude remains below
    # full scale so fixed-point tests exercise quantisation rather than
    # intentional clipping.
    input_f64 = (
            0.35 * np.exp(1j * 2.0 * np.pi * 2.0 * t)
            + 0.10 * np.exp(-1j * 2.0 * np.pi * 3.0 * t)
    ).astype(np.complex128)

    output_f64 = lfilter(
        taps_f64,
        np.array([1.0], dtype=np.float64),
        input_f64,
    ).astype(np.complex128)

    input_f32 = input_f64.astype(np.complex64)

    output_f32 = lfilter(
        taps_f32,
        np.array([1.0], dtype=np.float32),
        input_f32,
    ).astype(np.complex64)

    np.save(ref_vector / f"input_f64_{n}.npy", input_f64)
    np.save(ref_vector / f"output_f64_{n}.npy", output_f64)

    np.save(ref_vector / f"input_f32_{n}.npy", input_f32)
    np.save(ref_vector / f"output_f32_{n}.npy", output_f32)


print(f"Wrote FIR oracle vectors to {ref_vector}")
print(f"scipy {scipy.__version__}, numpy {np.__version__}")