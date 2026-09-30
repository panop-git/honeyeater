"""
Generate oracle vectors for Honeyeater complex magnitude/power primitives.

Floating-point vectors use NumPy arithmetic.

Fixed-point vectors use widened integer arithmetic:
- Complex<i16> -> u32 power / integer magnitude
- Complex<i8>  -> u32 power / integer magnitude
- magnitude is floor(sqrt(power)), matching Rust u32::isqrt()
"""

from pathlib import Path
import math
import numpy as np

repo_root = Path(__file__).resolve().parents[3]

ref_vector = (
    repo_root
    / "crates"
    / "honeyeater-core"
    / "tests"
    / "vectors"
    / "magnitude"
)

ref_vector.mkdir(parents=True, exist_ok=True)

length = 64


def integer_oracle(samples):
    widened = samples.astype(np.int64)

    power_i64 = (
        widened[:, 0] * widened[:, 0]
        + widened[:, 1] * widened[:, 1]
    )

    power = power_i64.astype(np.uint32)

    magnitude = np.array(
        [math.isqrt(int(value)) for value in power_i64],
        dtype=np.uint32,
    )

    return power, magnitude


# Floating-point input.
t = np.arange(length, dtype=np.float64) / length

signal = (
    0.55 * np.exp(1j * 2.0 * np.pi * 3.0 * t)
    + 0.20 * np.exp(-1j * 2.0 * np.pi * 7.0 * t)
)

input_f64 = signal.astype(np.complex128)
power_f64 = (
    input_f64.real * input_f64.real
    + input_f64.imag * input_f64.imag
)
magnitude_f64 = np.sqrt(power_f64)

input_f32 = signal.astype(np.complex64)
power_f32 = (
    input_f32.real * input_f32.real
    + input_f32.imag * input_f32.imag
).astype(np.float32)
magnitude_f32 = np.sqrt(power_f32).astype(np.float32)

np.save(ref_vector / "input_f64.npy", input_f64)
np.save(ref_vector / "power_f64.npy", power_f64)
np.save(ref_vector / "magnitude_f64.npy", magnitude_f64)

np.save(ref_vector / "input_f32.npy", input_f32)
np.save(ref_vector / "power_f32.npy", power_f32)
np.save(ref_vector / "magnitude_f32.npy", magnitude_f32)


# Fixed-point test vectors.
rng = np.random.default_rng(0x484F4E45)

input_i16 = rng.integers(
    i16_min := -32768,
    32768,
    size=(length, 2),
    dtype=np.int64,
).astype(np.int16)

input_i16[0] = [i16_min, i16_min]
input_i16[1] = [32767, 32767]
input_i16[2] = [3, 4]
input_i16[3] = [0, 0]

power_i16, magnitude_i16 = integer_oracle(input_i16)

np.save(ref_vector / "input_i16.npy", input_i16)
np.save(ref_vector / "power_i16.npy", power_i16)
np.save(ref_vector / "magnitude_i16.npy", magnitude_i16)


input_i8 = rng.integers(
    i8_min := -128,
    128,
    size=(length, 2),
    dtype=np.int64,
).astype(np.int8)

input_i8[0] = [i8_min, i8_min]
input_i8[1] = [127, 127]
input_i8[2] = [3, 4]
input_i8[3] = [0, 0]

power_i8, magnitude_i8 = integer_oracle(input_i8)

np.save(ref_vector / "input_i8.npy", input_i8)
np.save(ref_vector / "power_i8.npy", power_i8)
np.save(ref_vector / "magnitude_i8.npy", magnitude_i8)

print(f"Wrote magnitude oracle vectors to {ref_vector}")
print(f"numpy {np.__version__}")