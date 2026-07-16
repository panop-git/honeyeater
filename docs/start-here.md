# Start here

New to honeyeater? This page is the gentle on-ramp. It explains what the library is for and the handful of terms you need to read the rest of the documentation — no prior signal-processing background assumed. Words in **bold** link to the [Glossary](glossary.md), where each is defined in a sentence or two.

## What honeyeater is, in one paragraph

honeyeater is a Rust toolbox of building blocks for processing radio and electrical signals **on a regular computer** — not inside the radio chip itself. A radio or digitiser captures a signal as a stream of **[IQ samples](glossary.md#iq-samples)** (pairs of numbers describing the signal moment by moment); honeyeater gives you the pieces to turn that stream into something useful — filtering it, transforming it, decoding it. Each piece is a **[kernel](glossary.md#kernel)**: one self-contained operation, like a filter or an FFT.

## Who it's for

honeyeater is aimed at people building software that takes a stream of IQ samples — from a radio front-end, a digitiser, or a simulation — and does something with it on a host computer. For example:

- Satellite ground-station software
- Telemetry decoders (turning a spacecraft's or instrument's transmitted data back into readable values)
- Spectrum-analyser back-ends
- Signal-intelligence processing chains (extracting information from captured radio traffic)
- Modem implementations, from workstations down to microcontrollers
- Bench-test and instrumentation tooling

If you work with sampled signals and want memory-safe, well-tested primitives in Rust, you are the target reader.

## A five-minute concept map

A short tour of the ideas the rest of the docs lean on. You do not need to master these — just recognise them.

**Signals and samples.** A radio signal reaches software as **[IQ samples](glossary.md#iq-samples)** produced by an **[ADC](glossary.md#adc)** (the chip that turns the analog signal into numbers). honeyeater works on those numbers; it does not touch the analog hardware or the high-rate logic inside the radio.

**Kernels, generic over sample type.** A **[kernel](glossary.md#kernel)** is one DSP operation. honeyeater's kernels are written once and work across several number formats — high-precision floats for design work, and the compact fixed-point integers (**[Q-format](glossary.md#q-format-fixed-point)**) that real SDR hardware streams — with no runtime penalty, thanks to the compiler specialising each (**[monomorphisation](glossary.md#monomorphisation)**).

**Tested against an oracle.** Every kernel is checked against an **[oracle](glossary.md#oracle)**: a trusted reference (an established library, or correct values published in a standard) that says what the right answer is. This is the project's central discipline — a kernel isn't done until it matches its oracle.

**Tolerance.** Because floating-point results are rarely identical to the last bit, "correct" means "within a stated **[tolerance](glossary.md#tolerance-atol--rtol)**" — close enough, measured precisely. How honeyeater defines and tests that is the subject of the [Testing](testing.md) page.

## Where to go next

- **New to the project?** You're in the right place — then read the [Vision](vision.md) for the fuller statement of what honeyeater is and isn't.
- **Want to test a kernel?** [Testing](testing.md) is the practical guide.
- **Curious why the API looks the way it does?** [Architecture decisions](architecture-planning.md).
- **Hit an unfamiliar term?** The [Glossary](glossary.md) defines them all in one place.

## A note on status

honeyeater is pre-v0.0.1: the foundations and test harness exist, but no DSP kernels are implemented yet. So this page describes ideas rather than runnable code — the first hands-on tutorial will arrive with the first kernel.
