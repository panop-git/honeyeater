"""
Generate oracle vectors for Honeyeater complex multiply / mixer primitives.

Floating-point vectors use NumPy complex arithmetic.

Fixed-point vectors model Honeyeater's widened integer arithmetic:
- Complex<i16>: Q1.15 NCO, post-multiply shift 15
- Complex<i8>:  Q1.7 NCO, post-multiply shift 7
- round-to-nearest, ties away from zero
- saturation to the destination integer type
"""

from pathlib import Path
import numpy as np

repo_root = Path(__file__).resolve().parents[3]

ref_vector = (
    repo_root
    / "crates"
    / "honeyeater-core"
    / "tests"
    / "vectors"
    / "mixer"
)

ref_vector.mkdir(parents=True, exist_ok=True)

vec_lengths = [8, 16, 64]


def round_away_from_zero(values):
    return np.where(
        values >= 0,
        np.floor(values + 0.5),
        np.ceil(values - 0.5),
    )


def quantize_signed(values, bits):
    scale = 1 << (bits - 1)
    minimum = -scale
    maximum = scale - 1

    quantized = round_away_from_zero(values * scale)

    return np.clip(quantized, minimum, maximum)


def quantize_complex(values, bits, dtype):
    real = quantize_signed(values.real, bits).astype(dtype)
    imag = quantize_signed(values.imag, bits).astype(dtype)

    return np.column_stack((real, imag)).astype(dtype)


def round_shift(values, shift):
    if shift == 0:
        return values

    rounding = 1 << (shift - 1)
    magnitude = np.abs(values)

    rounded = (magnitude + rounding) >> shift

    return np.where(values < 0, -rounded, rounded)


def fixed_complex_multiply(lhs, rhs, shift, bits, conjugate_rhs=False):
    lhs = lhs.astype(np.int64)
    rhs = rhs.astype(np.int64)

    lhs_re = lhs[:, 0]
    lhs_im = lhs[:, 1]
    rhs_re = rhs[:, 0]
    rhs_im = rhs[:, 1]

    if conjugate_rhs:
        result_re = lhs_re * rhs_re + lhs_im * rhs_im
        result_im = lhs_im * rhs_re - lhs_re * rhs_im
    else:
        result_re = lhs_re * rhs_re - lhs_im * rhs_im
        result_im = lhs_re * rhs_im + lhs_im * rhs_re

    result_re = round_shift(result_re, shift)
    result_im = round_shift(result_im, shift)

    minimum = -(1 << (bits - 1))
    maximum = (1 << (bits - 1)) - 1

    result_re = np.clip(result_re, minimum, maximum)
    result_im = np.clip(result_im, minimum, maximum)

    dtype = np.int16 if bits == 16 else np.int8

    return np.column_stack((result_re, result_im)).astype(dtype)


for n in vec_lengths:
    t = np.arange(n, dtype=np.float64) / n

    # A deterministic multi-tone complex signal below full scale.
    signal = (
        0.45 * np.exp(1j * 2.0 * np.pi * 2.0 * t)
        + 0.15 * np.exp(-1j * 2.0 * np.pi * 3.0 * t)
    )

    # Unit-amplitude NCO.
    nco = np.exp(1j * 2.0 * np.pi * t)

    # Floating-point f64
    signal_f64 = signal.astype(np.complex128)
    nco_f64 = nco.astype(np.complex128)

    multiply_f64 = signal_f64 * nco_f64
    multiply_conj_f64 = signal_f64 * np.conj(nco_f64)

    np.save(ref_vector / f"input_f64_{n}.npy", signal_f64)
    np.save(ref_vector / f"nco_f64_{n}.npy", nco_f64)
    np.save(ref_vector / f"multiply_f64_{n}.npy", multiply_f64)
    np.save(ref_vector / f"multiply_conj_f64_{n}.npy", multiply_conj_f64)

    # Floating-point f32
    signal_f32 = signal.astype(np.complex64)
    nco_f32 = nco.astype(np.complex64)

    multiply_f32 = signal_f32 * nco_f32

    np.save(ref_vector / f"input_f32_{n}.npy", signal_f32)
    np.save(ref_vector / f"nco_f32_{n}.npy", nco_f32)
    np.save(ref_vector / f"multiply_f32_{n}.npy", multiply_f32)

    # Fixed-point i16 / Q1.15
    signal_i16 = quantize_complex(signal, 16, np.int16)
    nco_i16 = quantize_complex(nco, 16, np.int16)

    multiply_i16 = fixed_complex_multiply(
        signal_i16,
        nco_i16,
        shift=15,
        bits=16,
    )

    multiply_conj_i16 = fixed_complex_multiply(
        signal_i16,
        nco_i16,
        shift=15,
        bits=16,
        conjugate_rhs=True,
    )

    np.save(ref_vector / f"input_i16_{n}.npy", signal_i16)
    np.save(ref_vector / f"nco_i16_{n}.npy", nco_i16)
    np.save(ref_vector / f"multiply_i16_{n}.npy", multiply_i16)
    np.save(ref_vector / f"multiply_conj_i16_{n}.npy", multiply_conj_i16)

    # Fixed-point i8 / Q1.7
    signal_i8 = quantize_complex(signal, 8, np.int8)
    nco_i8 = quantize_complex(nco, 8, np.int8)

    multiply_i8 = fixed_complex_multiply(
        signal_i8,
        nco_i8,
        shift=7,
        bits=8,
    )

    multiply_conj_i8 = fixed_complex_multiply(
        signal_i8,
        nco_i8,
        shift=7,
        bits=8,
        conjugate_rhs=True,
    )

    np.save(ref_vector / f"input_i8_{n}.npy", signal_i8)
    np.save(ref_vector / f"nco_i8_{n}.npy", nco_i8)
    np.save(ref_vector / f"multiply_i8_{n}.npy", multiply_i8)
    np.save(ref_vector / f"multiply_conj_i8_{n}.npy", multiply_conj_i8)


print(f"Wrote mixer oracle vectors to {ref_vector}")
print(f"numpy {np.__version__}")