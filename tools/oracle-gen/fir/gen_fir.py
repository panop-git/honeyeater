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
# test. These asymmetric real taps detect incorrect coefficient ordering.
taps_f64 = np.array(
    [
        0.1875,
        -0.078125,
        0.09375,
        0.03125,
        -0.046875,
        0.078125,
        -0.03125,
        0.046875,
        0.015625,
    ],
    dtype=np.float64,
)

taps_f32 = taps_f64.astype(np.float32)

# Complex coefficients detect both reversed tap ordering and accidental
# conjugation. Each component is representable in Q1.7 for fixed-point tests.
taps_complex_f64 = np.array(
    [
        0.1875 + 0.0625j,
        -0.078125 + 0.109375j,
        0.09375 - 0.046875j,
        0.03125 + 0.015625j,
        -0.046875 - 0.078125j,
        0.078125 + 0.03125j,
        -0.03125 + 0.046875j,
        0.046875 - 0.015625j,
        0.015625 + 0.03125j,
    ],
    dtype=np.complex128,
)

taps_complex_f32 = taps_complex_f64.astype(np.complex64)

np.save(ref_vector / "taps_f64.npy", taps_f64)
np.save(ref_vector / "taps_f32.npy", taps_f32)
np.save(ref_vector / "taps_complex_f64.npy", taps_complex_f64)
np.save(ref_vector / "taps_complex_f32.npy", taps_complex_f32)


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

    # Use complex taps directly, without reversing or conjugating them.
    output_complex_f64 = lfilter(
        taps_complex_f64,
        np.array([1.0], dtype=np.complex128),
        input_f64,
    ).astype(np.complex128)

    output_complex_f32 = lfilter(
        taps_complex_f32,
        np.array([1.0], dtype=np.complex64),
        input_f32,
    ).astype(np.complex64)

    # The real-input FIR implementations need their own oracle vectors.
    output_real_f64 = lfilter(
        taps_f64,
        np.array([1.0], dtype=np.float64),
        input_f64.real,
    ).astype(np.float64)

    output_real_f32 = lfilter(
        taps_f32,
        np.array([1.0], dtype=np.float32),
        input_f32.real,
    ).astype(np.float32)

    np.save(ref_vector / f"input_f64_{n}.npy", input_f64)
    np.save(ref_vector / f"output_f64_{n}.npy", output_f64)

    np.save(ref_vector / f"input_f32_{n}.npy", input_f32)
    np.save(ref_vector / f"output_f32_{n}.npy", output_f32)

    np.save(ref_vector / f"output_complex_f64_{n}.npy", output_complex_f64)
    np.save(ref_vector / f"output_complex_f32_{n}.npy", output_complex_f32)

    np.save(ref_vector / f"output_real_f64_{n}.npy", output_real_f64)
    np.save(ref_vector / f"output_real_f32_{n}.npy", output_real_f32)


print(f"Wrote FIR oracle vectors to {ref_vector}")
print(f"scipy {scipy.__version__}, numpy {np.__version__}")
