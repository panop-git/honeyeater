"""Additional Phase 1 contract fixtures from the canonical NumPy/SciPy pins.

RBJ low-pass coefficients follow the W3C Audio EQ Cookbook formula.
SciPy supplies independent filter execution, window, and FFT references.
NumPy supplies magnitude/power references, including every Complex<i8> input.
"""
from pathlib import Path
import numpy as np
from scipy import fft, signal

root = Path(__file__).resolve().parents[3]
out = root / "crates/honeyeater-core/tests/vectors/phase1_audit"
out.mkdir(parents=True, exist_ok=True)

for n in [2, 3, 7, 15, 65]:
    t = np.arange(n, dtype=np.float64) / n
    samples = (0.4*np.exp(2j*np.pi*2*t) + 0.2*np.exp(-2j*np.pi*3*t)).astype(np.complex128)
    np.save(out/f"fft_input_{n}.npy", samples)
    np.save(out/f"fft_output_{n}.npy", fft.fft(samples))
    np.save(out/f"fft_frequency_{n}.npy", fft.fftfreq(n))

for beta in [0, 14, 30, 50, 100, 700]:
    np.save(out/f"kaiser_{beta}.npy", signal.windows.kaiser(65, beta))

for case, (frequency, q) in enumerate([(0.01, 0.5), (0.125, 1.0), (0.45, 10.0)]):
    angle = 2*np.pi*frequency
    cosine = np.cos(angle)
    alpha = np.sin(angle)/(2*q)
    b = np.array([(1-cosine)/2, 1-cosine, (1-cosine)/2])/(1+alpha)
    a = np.array([1, -2*cosine/(1+alpha), (1-alpha)/(1+alpha)])
    sos = np.concatenate([b, a]).reshape(1, 6)
    impulse = np.zeros(256)
    impulse[0] = 1
    np.save(out/f"biquad_coefficients_{case}.npy", sos.reshape(-1))
    np.save(out/f"biquad_output_{case}.npy", signal.sosfilt(sos, impulse))
    np.save(out/f"biquad_output_f32_{case}.npy", signal.sosfilt(sos.astype(np.float32), impulse.astype(np.float32)))

values = np.arange(-128, 128, dtype=np.int16)
real, imag = np.meshgrid(values, values, indexing="ij")
pairs = np.column_stack([real.reshape(-1), imag.reshape(-1)]).astype(np.int8)
np.save(out/"magnitude_input_i8.npy", pairs)

rng = np.random.default_rng(20260930)
pairs16 = rng.integers(-32768, 32768, size=(1024, 2), dtype=np.int16)
limits = np.array([-32768, -32767, -1, 0, 1, 32766, 32767], dtype=np.int16)
real, imag = np.meshgrid(limits, limits, indexing="ij")
pairs16 = np.concatenate([np.column_stack([real.reshape(-1), imag.reshape(-1)]), pairs16])
np.save(out/"magnitude_input_i16.npy", pairs16)
for suffix, inputs in [("i8", pairs), ("i16", pairs16)]:
    widened = inputs.astype(np.int64)
    power = (widened[:, 0]**2 + widened[:, 1]**2).astype(np.uint32)
    magnitude = np.floor(np.sqrt(power.astype(np.float64))).astype(np.uint32)
    np.save(out/f"power_{suffix}.npy", power)
    np.save(out/f"magnitude_{suffix}.npy", magnitude)

samples = np.array([0, 3+4j, -3-4j, 1e-150+1e-150j, 1e150+1e150j, 0.5-0.125j], dtype=np.complex128)
np.save(out/"magnitude_input_f64.npy", samples)
np.save(out/"magnitude_f64.npy", np.abs(samples))
np.save(out/"power_f64.npy", samples.real**2 + samples.imag**2)
print(f"Wrote Phase 1 audit fixtures to {out}")
