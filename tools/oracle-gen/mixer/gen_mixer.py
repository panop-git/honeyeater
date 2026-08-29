"""
Script generates oracle test values for mixer operations from scipy.
"""

from pathlib import Path
import numpy as np
import scipy

# Stores generated oracle vectors into tests/vectors directory
refVector = Path("../../crates/honeyeater-core/tests/vectors/mixer")
refVector.mkdir(parents=True, exist_ok=True)

vecLength = [8, 16, 64]
shift = 10
scale_factor = 1 << 14
round_val = 1 << (shift - 1)

for n in vecLength:
    # Create a time array
    t = np.linspace(0, 1, n, endpoint=False)

    # Generate complex signal
    signal = np.sin(2 * np.pi * 2 * t) + 1j * np.cos(2 * np.pi * 3 * t)

    # Generate a complex NCO output
    nco = np.exp(1j * 2 * np.pi * 1 * t)

    # Floating-Point results
    mix_up_res = signal * nco
    mix_down_res = signal * np.conj(nco)

    # Fixed-Point results (i32 with scaling and half-up rounding)
    s_re = (signal.real * scale_factor).astype(np.int32)
    s_im = (signal.imag * scale_factor).astype(np.int32)
    n_re = (nco.real * scale_factor).astype(np.int32)
    n_im = (nco.imag * scale_factor).astype(np.int32)

    s_re_64 = s_re.astype(np.int64)
    s_im_64 = s_im.astype(np.int64)
    n_re_64 = n_re.astype(np.int64)
    n_im_64 = n_im.astype(np.int64)

    mix_up_fixed_re = (s_re_64 * n_re_64 - s_im_64 * n_im_64 + round_val) >> shift
    mix_up_fixed_im = (s_re_64 * n_im_64 + s_im_64 * n_re_64 + round_val) >> shift

    mix_down_fixed_re = (s_re_64 * n_re_64 + s_im_64 * n_im_64 + round_val) >> shift
    mix_down_fixed_im = (s_im_64 * n_re_64 - s_re_64 * n_im_64 + round_val) >> shift

    # Contiguous layouts
    input_fixed_res = np.column_stack((s_re, s_im)).astype(np.int32)
    nco_fixed_res = np.column_stack((n_re, n_im)).astype(np.int32)
    mix_up_fixed_res = np.column_stack((mix_up_fixed_re, mix_up_fixed_im)).astype(np.int32)
    mix_down_fixed_res = np.column_stack((mix_down_fixed_re, mix_down_fixed_im)).astype(np.int32)

    # Save ALL floating-point and fixed-point vectors to disk
    np.save(refVector / f"input_{n}.npy", signal)
    np.save(refVector / f"nco_{n}.npy", nco)
    np.save(refVector / f"mix_up_{n}.npy", mix_up_res)
    np.save(refVector / f"mix_down_{n}.npy", mix_down_res)
    
    np.save(refVector / f"input_fixed_{n}.npy", input_fixed_res)
    np.save(refVector / f"nco_fixed_{n}.npy", nco_fixed_res)
    np.save(refVector / f"mix_up_fixed_{n}.npy", mix_up_fixed_res)
    np.save(refVector / f"mix_down_fixed_{n}.npy", mix_down_fixed_res)

print(f"Wrote all floating and fixed vectors to {refVector}")