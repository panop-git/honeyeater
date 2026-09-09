"""
Generate oracle vectors for the Honeyeater NCO / DDS implementation.

The oracle models the public DDS behaviour:

- 32-bit wrapping phase accumulator
- frequency expressed in cycles/sample
- unit-amplitude complex exponential
- Q1.15 output for Complex<i16>
- Q1.7 output for Complex<i8>

Fixed-point rounding matches Rust f64::round(): nearest integer with
half-way cases rounded away from zero.
"""

from pathlib import Path
import numpy as np

PHASE_MODULUS = 1 << 32

# Resolve from this file so the generator works regardless of current directory.
repoRoot = Path(__file__).resolve().parents[3]

refVector = (
        repoRoot
        / "crates"
        / "honeyeater-core"
        / "tests"
        / "vectors"
        / "nco"
)

refVector.mkdir(parents=True, exist_ok=True)

vecLength = 64
frequency = 5.0 / 64.0
initial_phase = 0.0


def cycles_to_phase_word(cycles):
    wrapped = cycles % 1.0

    # wrapped is non-negative, so floor(x + 0.5) matches Rust round()
    # for conversion into the unsigned phase word.
    word = int(np.floor(wrapped * PHASE_MODULUS + 0.5))

    return word & 0xFFFFFFFF


def generate_phase_words(length, frequency_cycles, phase_cycles):
    phase = cycles_to_phase_word(phase_cycles)
    increment = cycles_to_phase_word(frequency_cycles)

    words = np.empty(length, dtype=np.uint32)

    for n in range(length):
        words[n] = phase
        phase = (phase + increment) & 0xFFFFFFFF

    return words


def round_away_from_zero(values):
    """
    Match Rust f64::round() rather than numpy.round(), which uses
    round-to-even for halfway cases.
    """
    return np.where(
        values >= 0.0,
        np.floor(values + 0.5),
        np.ceil(values - 0.5),
        )


def quantize_signed(values, bits):
    scale = 1 << (bits - 1)
    minimum = -scale
    maximum = scale - 1

    quantized = round_away_from_zero(values * scale)
    return np.clip(quantized, minimum, maximum)


phase_words = generate_phase_words(
    vecLength,
    frequency,
    initial_phase,
)

phase_cycles = phase_words.astype(np.float64) / float(PHASE_MODULUS)
angles = 2.0 * np.pi * phase_cycles

real = np.cos(angles)
imag = np.sin(angles)

# Floating-point oracle vectors
nco_f64 = (real + 1j * imag).astype(np.complex128)
nco_f32 = (real + 1j * imag).astype(np.complex64)

# Fixed-point oracle vectors
i16_real = quantize_signed(real, 16).astype(np.int16)
i16_imag = quantize_signed(imag, 16).astype(np.int16)
nco_i16 = np.column_stack((i16_real, i16_imag)).astype(np.int16)

i8_real = quantize_signed(real, 8).astype(np.int8)
i8_imag = quantize_signed(imag, 8).astype(np.int8)
nco_i8 = np.column_stack((i8_real, i8_imag)).astype(np.int8)

np.save(refVector / "nco_f64_64.npy", nco_f64)
np.save(refVector / "nco_f32_64.npy", nco_f32)
np.save(refVector / "nco_i16_64.npy", nco_i16)
np.save(refVector / "nco_i8_64.npy", nco_i8)

print(f"Wrote NCO oracle vectors to {refVector}")
print(f"numpy {np.__version__}")
print(f"frequency = {frequency} cycles/sample")
print(f"phase increment = 0x{cycles_to_phase_word(frequency):08X}")