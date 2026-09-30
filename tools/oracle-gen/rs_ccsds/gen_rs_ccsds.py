"""
Generate CCSDS RS(255,223) encoder reference vectors using Phil Karn's libfec.

The resulting .bin files contain complete systematic codewords:

    223 information symbols || 32 check symbols

All symbols use the CCSDS dual-basis representation.

libfec is cloned and built only inside a temporary directory. It never enters
Honeyeater's Cargo dependency graph.
"""

from pathlib import Path
import shutil
import subprocess
import tempfile


LIBFEC_REPOSITORY = "https://github.com/quiet/libfec.git"

# Pinned libfec snapshot containing Phil Karn's CCSDS dual-basis encoder and
# the August 2007 encode_rs_ccsds parity-transform fix.
LIBFEC_COMMIT = "9750ca0a6d0a786b506e44692776b541f90daa91"

DATA_SYMBOLS = 223
CODEWORD_SYMBOLS = 255


repo_root = Path(__file__).resolve().parents[3]

output_dir = (
    repo_root
    / "crates"
    / "honeyeater-core"
    / "tests"
    / "vectors"
    / "rs_ccsds"
)

output_dir.mkdir(parents=True, exist_ok=True)


HELPER_SOURCE = r"""
#include <stdio.h>
#include <stdlib.h>

#include "fec.h"

#define DATA_SYMBOLS 223
#define PARITY_SYMBOLS 32

int main(void)
{
    unsigned char data[DATA_SYMBOLS];
    unsigned char parity[PARITY_SYMBOLS];

    if (fread(data, 1, DATA_SYMBOLS, stdin) != DATA_SYMBOLS) {
        fprintf(stderr, "expected exactly 223 input symbols\n");
        return EXIT_FAILURE;
    }

    encode_rs_ccsds(data, parity, 0);

    if (fwrite(data, 1, DATA_SYMBOLS, stdout) != DATA_SYMBOLS) {
        return EXIT_FAILURE;
    }

    if (fwrite(parity, 1, PARITY_SYMBOLS, stdout) != PARITY_SYMBOLS) {
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
"""


def run(command, cwd=None):
    print("+", " ".join(str(item) for item in command))

    subprocess.run(
        command,
        cwd=cwd,
        check=True,
    )


def encode(helper, data):
    if len(data) != DATA_SYMBOLS:
        raise ValueError("CCSDS RS(255,223) input must contain 223 symbols")

    result = subprocess.run(
        [str(helper)],
        input=data,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )

    if len(result.stdout) != CODEWORD_SYMBOLS:
        raise RuntimeError(
            f"libfec helper returned {len(result.stdout)} bytes, "
            f"expected {CODEWORD_SYMBOLS}"
        )

    return result.stdout


def lcg_vector():
    state = 0x42
    output = bytearray()

    for _ in range(DATA_SYMBOLS):
        state = (73 * state + 41) & 0xFF
        output.append(state)

    return bytes(output)


vectors = {
    "libfec_ramp.bin": bytes(range(DATA_SYMBOLS)),
    "libfec_alternating.bin": bytes(
        0x55 if index % 2 == 0 else 0xAA
        for index in range(DATA_SYMBOLS)
    ),
    "libfec_lcg.bin": lcg_vector(),
}


def main():
    required_tools = ["git", "cc", "make"]

    for tool in required_tools:
        if shutil.which(tool) is None:
            raise RuntimeError(f"required build tool not found: {tool}")

    with tempfile.TemporaryDirectory(prefix="honeyeater-libfec-") as temp:
        temp_path = Path(temp)
        libfec_path = temp_path / "libfec"

        run(
            [
                "git",
                "clone",
                "--quiet",
                LIBFEC_REPOSITORY,
                str(libfec_path),
            ]
        )

        run(
            [
                "git",
                "checkout",
                "--quiet",
                LIBFEC_COMMIT,
            ],
            cwd=libfec_path,
        )

        # libfec ships its configure script in this revision.
        run(
            [
                "./configure",
                "--disable-shared",
            ],
            cwd=libfec_path,
        )

        run(
            [
                "make",
                "libfec.a",
            ],
            cwd=libfec_path,
        )

        helper_source = temp_path / "rs_oracle.c"
        helper_binary = temp_path / "rs_oracle"

        helper_source.write_text(
            HELPER_SOURCE,
            encoding="utf-8",
        )

        run(
            [
                "cc",
                "-O2",
                "-I",
                str(libfec_path),
                str(helper_source),
                str(libfec_path / "libfec.a"),
                "-lm",
                "-o",
                str(helper_binary),
            ]
        )

        for filename, data in vectors.items():
            codeword = encode(helper_binary, data)

            output_path = output_dir / filename
            output_path.write_bytes(codeword)

            print(
                f"Wrote {output_path} "
                f"({len(codeword)} bytes)"
            )

    print(
        "Generated vectors using libfec commit "
        f"{LIBFEC_COMMIT}"
    )


if __name__ == "__main__":
    main()