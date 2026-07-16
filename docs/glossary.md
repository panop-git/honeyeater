# Glossary

Plain-English definitions of the recurring terms in honeyeater's documentation. Each entry is intentionally brief — just enough to keep reading; the page that uses a term is where its full treatment lives.

## Signals and hardware

### IQ samples
Pairs of numbers (in-phase and quadrature) that together describe a radio signal at an instant — the raw form a radio front-end or digitiser hands to software. A stream of IQ samples is what most honeyeater kernels consume.

### ADC
Analog-to-digital converter: the chip inside a radio or digitiser that turns the continuous analog signal into a stream of numbers. honeyeater starts where the ADC's samples reach the host computer.

### SNR (and dB)
Signal-to-noise ratio: how large the wanted signal is compared to the error or noise riding on it. Measured in **decibels (dB)**, a logarithmic scale where larger is cleaner — every 10 dB is a tenfold ratio.

### PSD (power spectral density)
A description of how a signal's power is spread across frequency — power per hertz. Spectral estimators (Welch, periodogram, multitaper) compute a PSD from a finite chunk of samples.

### Spectral mask
A pair of upper and lower bounds, one per frequency bin, that a signal's spectrum must stay within. Used to check that a filter's response or a transmitter's emissions have the right shape (flat passband, steep transition, attenuated stopband).

### Q-format (fixed-point)
A convention for storing fractional numbers inside plain integers by fixing where the binary point sits (e.g. Q1.15 = a 16-bit integer read as a value in roughly [-1, 1)). It is how the SDR hardware that produces `i16`/`i8` samples represents fractions — a property of *how surrounding code interprets the integer*, not of the integer type itself.

### FFT
Fast Fourier transform: an efficient algorithm that converts a block of samples between the time domain and the frequency domain. A workhorse primitive; honeyeater depends on the `rustfft` crate rather than reimplementing it.

## Coding and modulation

### FEC (forward error correction)
Forward error correction: adding structured redundancy to data so the receiver can detect and repair bit errors without asking for a retransmission. An **encoder** adds the redundancy; a **decoder** uses it to recover the original.

### LDPC, turbo, polar-SCL
Families of modern FEC codes used in real links (satellite, cellular, broadcast). Their decoders are *iterative* and work on soft (probabilistic) inputs, so two correct implementations can produce slightly different outputs — which is why they are tested by error rate, not bit-for-bit.

### BER (bit error rate)
Bit error rate: the fraction of bits a link gets wrong (errors ÷ bits transmitted). The standard measure of how well a decoder performs.

### Eb/N0
A normalised signal-to-noise measure for digital links: energy per bit divided by noise power density. The conventional x-axis for a BER curve — "how many errors at this much signal-to-noise."

### Waterfall
The steep part of a BER curve, where a small improvement in Eb/N0 buys a large drop in error rate. Below it the curve flattens into the **error floor**.

### EVM (error vector magnitude)
A measure of how far received modulation symbols land from their ideal positions — a single number summarising modulation quality, often quoted as a percentage against a standard.

### OFDM
Orthogonal frequency-division multiplexing: a modulation scheme that spreads data across many narrow subcarriers at once. Used in Wi-Fi, LTE, DVB, and many other systems.

### Parks-McClellan
A classic algorithm for designing FIR filters with the best possible (equiripple) response for a given length. Named after its authors.

## Testing and numerics

### Oracle
A trusted reference that honeyeater checks its own results against — either an established implementation (scipy, liquid-dsp, libfec) or a set of correct outputs published in a standard. Every kernel is validated against a *named* oracle.

### Tolerance (atol / rtol)
How close "close enough" is when comparing floating-point results. honeyeater uses a mixed measure: an **absolute tolerance** (`atol`, a floor near zero) plus a **relative tolerance** (`rtol`, scaled to the value's size), combined as `atol + rtol·|b|`.

### Bit-exact
Required to match the reference exactly, byte for byte. The right standard for deterministic outputs like FEC encoders and CRCs; the wrong one for floating-point kernels, whose last bits vary across platforms.

### Parseval's theorem
The fact that a signal's total energy is the same whether you measure it in the time domain or add it up across frequency. honeyeater uses it to check that a spectral estimator's output "adds up" to the right total energy.

### Kolmogorov-Smirnov (KS) test
A statistical test for "do these samples come from the distribution I claim?" It measures the largest gap between the samples' observed distribution and the target one. Used to check PRNGs and noise generators.

### CDF (cumulative distribution function)
A function giving the probability that a random draw falls at or below a given value. The KS test compares a target CDF against what the samples actually produced.

### PRNG
Pseudo-random number generator: an algorithm that produces a repeatable stream of numbers that *look* random. "Pseudo" because a fixed seed always yields the same stream — which is what makes seeded tests deterministic.

### ULP
Unit in the last place: the gap between two adjacent representable floating-point numbers. Results that differ "by a few ULP" differ only in their final bits — the unavoidable noise of floating-point arithmetic.

### FMA (fused multiply-add)
A CPU instruction that computes `a * b + c` in one step, more accurately than doing the multiply and add separately. Because it rounds differently, it can make the same code give slightly different results on different machines.

### libm
The system math library that supplies `sin`, `exp`, `log`, `pow`, and friends. Different implementations (glibc, musl, Apple, Windows) differ by a few ULP, so reference values are baked in once rather than recomputed per platform.

## Rust and project

### Kernel
A single DSP building block — one filter, one transform, one encoder. honeyeater is a library of kernels. (Nothing to do with operating-system kernels.)

### Workspace crate
A crate (Rust package) that is one member of a Cargo *workspace* — a set of related crates built together. honeyeater is split into a few small workspace crates rather than one large one.

### MSRV
Minimum supported Rust version: the oldest Rust toolchain the project promises to compile on.

### Monomorphisation
The compiler's trick of taking generic code written once and emitting a specialised, fully-optimised copy for each concrete type it is used with — so generic kernels pay no runtime cost for being generic.

### Hot path
The code that runs most often and most must be fast — here, the per-sample work in a streaming receiver. APIs on the hot path take plain slices (`&[T]`) to stay allocation-free.

### rustdoc
Rust's built-in API-documentation generator. It produces the reference for what types and functions actually exist; build it with `cargo doc`.

### Tier-1
honeyeater's label for the first batch of foundational RF/electrical primitives (windows, basic filters, FFT wrapper, CRCs, and so on) — the kernels implemented first because later work builds on them. The ordered list lives in the [Roadmap](roadmap.md).
