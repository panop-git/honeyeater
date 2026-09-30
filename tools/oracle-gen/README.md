# oracle-gen

`tools/oracle-gen/` hosts the reference-vector generators for honeyeater. Python scripts generate the Phase 1 fixtures; the separate Cargo workspace remains a bootstrap target for future Rust oracle runners. Neither is part of the published library workspace.

## Licence boundary

Architecture decision 10 (`docs/architecture-planning.md`) keeps non-permissively licensed oracles outside the library's dependency graph, including dev-dependencies. KA9Q libfec (LGPL) runs here to generate CCSDS Reed-Solomon fixtures. Only its captured binary outputs enter the library tests; libfec is never linked into a honeyeater crate.

The fixtures live under `crates/honeyeater-core/tests/vectors/`. Ordinary Rust tests read the committed files without running the generators or requiring SciPy or libfec. The optional development-time `honeyeater_test::scipy::run` helper and its tests require Python.

## Python environment and pins

Run these commands from the repository root, using the Python interpreter from your virtual environment:

```sh
python -m pip install -r tools/oracle-gen/requirements.txt
python -c "import numpy, scipy; print('numpy', numpy.__version__, 'scipy', scipy.__version__)"
```

The canonical environment is **NumPy 2.5.1 and SciPy 1.18.0**, pinned in the single `requirements.txt` above. There is no separate windows requirements file. Use this environment for every NumPy/SciPy generator, including the scripts that only import NumPy. `PYTHON` selects the interpreter used by the Rust subprocess helper; it does not install or verify these packages.

## Generators

All commands below run from the repository root. Each script resolves its output directory relative to its own file.

| Fixtures | Oracle | Command |
|---|---|---|
| Hann, Hamming, Blackman-Harris, Kaiser windows | `scipy.signal.windows` | `python tools/oracle-gen/windows/gen_windows.py` |
| RBJ low-pass impulse responses | RBJ formulas + `scipy.signal.sosfilt` | `python tools/oracle-gen/biquad_filters/gen_biquad.py` |
| Complex FFT and inverse FFT | `scipy.fft` | `python tools/oracle-gen/fft/gen_fft.py` |
| Float and fixed-point NCO / DDS | NumPy complex exponential and quantisation model | `python tools/oracle-gen/nco/gen_nco.py` |
| Floating-point FIR execution | `scipy.signal.lfilter` with explicit coefficients | `python tools/oracle-gen/fir/gen_fir.py` |
| Float and fixed-point complex multiply | NumPy arithmetic and widened integer model | `python tools/oracle-gen/mixer/gen_mixer.py` |
| CCSDS RS(255,223) complete codewords | Phil Karn's libfec dual-basis encoder | `python tools/oracle-gen/rs_ccsds/gen_rs_ccsds.py` |

The RS generator pins `quiet/libfec` commit `9750ca0a6d0a786b506e44692776b541f90daa91` in its source. It requires a POSIX build environment with `git`, `cc`, and `make`, plus network access to clone the pinned snapshot. It builds libfec in a temporary directory and writes three 255-byte systematic codewords. It does not require NumPy or SciPy. The library tests also check the CCSDS Annex F basis-transformation examples; those are distinct from the libfec codeword fixtures.

## Updating fixtures

When an oracle pin changes, install the updated environment, regenerate the affected vectors, review the binary changes, and run `cargo test --workspace --all-targets`. Commit the pin and affected vectors together, recording the oracle version or commit. CI validates the kernels against committed vectors and does not regenerate them.

The bootstrap Cargo target can still be checked explicitly:

```sh
cargo build --manifest-path tools/oracle-gen/Cargo.toml
```

It reports where the Python generators live; it does not generate fixtures itself.
