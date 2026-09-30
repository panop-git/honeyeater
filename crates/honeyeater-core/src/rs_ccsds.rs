/// Number of information symbols in the CCSDS RS(255,223) codeword.
pub const CCSDS_RS_DATA_SYMBOLS: usize = 223;

/// Number of check symbols in the CCSDS RS(255,223) codeword.
pub const CCSDS_RS_PARITY_SYMBOLS: usize = 32;

/// Total number of symbols in the CCSDS RS(255,223) codeword.
pub const CCSDS_RS_CODEWORD_SYMBOLS: usize = 255;

// CCSDS 131.0-B-5 section 4.3.3:
//
// F(x) = x^8 + x^7 + x^2 + x + 1
//
// During an 8-bit polynomial multiply, the x^8 term is discarded after
// reduction, leaving x^7 + x^2 + x + 1 = 0x87.
const FIELD_REDUCTION_POLYNOMIAL: u8 = 0x87;

// CCSDS 131.0-B-5 section 4.3.9 / Annex F.
//
// Each byte represents one row of the conventional -> dual basis
// transformation matrix T_alpha_l.
//
// Input bits are interpreted MSB-first as [u7, u6, ... u0].
// Output bits are [z0, z1, ... z7].
const CONVENTIONAL_TO_DUAL_ROWS: [u8; 8] = [
    0x8D, // 10001101
    0xEF, // 11101111
    0xEC, // 11101100
    0x86, // 10000110
    0xFA, // 11111010
    0x99, // 10011001
    0xAF, // 10101111
    0x7B, // 01111011
];

// Inverse transformation T_alpha_l^-1.
//
// Input bits are [z0, z1, ... z7].
// Output bits are [u7, u6, ... u0].
const DUAL_TO_CONVENTIONAL_ROWS: [u8; 8] = [
    0xC5, // 11000101
    0x42, // 01000010
    0x2E, // 00101110
    0xFD, // 11111101
    0xF0, // 11110000
    0x79, // 01111001
    0xAC, // 10101100
    0xCC, // 11001100
];

// CCSDS 131.0-B-5 Annex G, E = 16.
//
// Conventional-basis coefficients of:
//
// g(x) = G0 + G1*x + ... + G32*x^32
//
// The polynomial is self-reciprocal, hence the symmetric coefficients.
const GENERATOR_POLYNOMIAL: [u8; 33] = [
    0x01, // G0
    0x5B, // G1
    0x7F, // G2
    0x56, // G3
    0x10, // G4
    0x1E, // G5
    0x0D, // G6
    0xEB, // G7
    0x61, // G8
    0xA5, // G9
    0x08, // G10
    0x2A, // G11
    0x36, // G12
    0x56, // G13
    0xAB, // G14
    0x20, // G15
    0x71, // G16
    0x20, // G17
    0xAB, // G18
    0x56, // G19
    0x36, // G20
    0x2A, // G21
    0x08, // G22
    0xA5, // G23
    0x61, // G24
    0xEB, // G25
    0x0D, // G26
    0x1E, // G27
    0x10, // G28
    0x56, // G29
    0x7F, // G30
    0x5B, // G31
    0x01, // G32
];

/// Multiplies two conventional-basis symbols in GF(2^8).
///
/// The field is defined by the CCSDS polynomial
/// `x^8 + x^7 + x^2 + x + 1`.
fn gf_multiply(mut lhs: u8, mut rhs: u8) -> u8 {
    let mut product = 0_u8;

    for _ in 0..8 {
        if rhs & 1 != 0 {
            product ^= lhs;
        }

        let carry = lhs & 0x80;
        lhs <<= 1;

        if carry != 0 {
            lhs ^= FIELD_REDUCTION_POLYNOMIAL;
        }

        rhs >>= 1;
    }

    product
}

/// Applies one of the CCSDS eight-bit basis transformation matrices.
///
/// `rows` contains the matrix rows packed MSB-first into bytes.
fn transform_basis(value: u8, rows: &[u8; 8]) -> u8 {
    let mut transformed = 0_u8;

    for (row_index, &row) in rows.iter().enumerate() {
        let input_mask = 0x80_u8 >> row_index;

        if value & input_mask != 0 {
            transformed ^= row;
        }
    }

    transformed
}

/// Converts a CCSDS dual-basis symbol into conventional polynomial basis.
fn dual_to_conventional(value: u8) -> u8 {
    transform_basis(value, &DUAL_TO_CONVENTIONAL_ROWS)
}

/// Converts a conventional-basis symbol into CCSDS dual basis.
fn conventional_to_dual(value: u8) -> u8 {
    transform_basis(value, &CONVENTIONAL_TO_DUAL_ROWS)
}

/// Computes the 32 conventional-basis parity symbols for one RS(255,223)
/// information block.
fn encode_conventional_parity(
    data: &[u8; CCSDS_RS_DATA_SYMBOLS],
) -> [u8; CCSDS_RS_PARITY_SYMBOLS] {
    let mut parity = [0_u8; CCSDS_RS_PARITY_SYMBOLS];

    for &symbol in data {
        let feedback = symbol ^ parity[0];

        for index in 0..(CCSDS_RS_PARITY_SYMBOLS - 1) {
            let generator_index = CCSDS_RS_PARITY_SYMBOLS - 1 - index;

            parity[index] = parity[index + 1]
                ^ gf_multiply(feedback, GENERATOR_POLYNOMIAL[generator_index]);
        }

        parity[CCSDS_RS_PARITY_SYMBOLS - 1] =
            gf_multiply(feedback, GENERATOR_POLYNOMIAL[0]);
    }

    parity
}

/// Encodes one CCSDS Reed-Solomon (255,223) information block and returns the
/// 32 check symbols.
///
/// Input and output symbols use the CCSDS-required dual-basis representation.
///
/// This implements the E=16 code specified by CCSDS 131.0-B-5:
///
/// - 8 bits per symbol
/// - 223 information symbols
/// - 32 check symbols
/// - 255 symbols per codeword
/// - GF(2^8) field polynomial `x^8 + x^7 + x^2 + x + 1`
/// - dual-basis symbol representation
#[must_use]
pub fn ccsds_rs_255_223_parity(
    data: &[u8; CCSDS_RS_DATA_SYMBOLS],
) -> [u8; CCSDS_RS_PARITY_SYMBOLS] {
    let mut conventional_data = [0_u8; CCSDS_RS_DATA_SYMBOLS];

    for (output, &input) in conventional_data.iter_mut().zip(data.iter()) {
        *output = dual_to_conventional(input);
    }

    let mut parity = encode_conventional_parity(&conventional_data);

    for symbol in &mut parity {
        *symbol = conventional_to_dual(*symbol);
    }

    parity
}

/// Encodes one complete CCSDS Reed-Solomon (255,223) codeword.
///
/// The encoder is systematic: the first 223 symbols are the information
/// symbols unchanged, followed by 32 Reed-Solomon check symbols.
///
/// Input and output symbols use the CCSDS dual-basis representation.
#[must_use]
pub fn ccsds_rs_255_223_encode(
    data: &[u8; CCSDS_RS_DATA_SYMBOLS],
) -> [u8; CCSDS_RS_CODEWORD_SYMBOLS] {
    let parity = ccsds_rs_255_223_parity(data);

    let mut codeword = [0_u8; CCSDS_RS_CODEWORD_SYMBOLS];

    codeword[..CCSDS_RS_DATA_SYMBOLS].copy_from_slice(data);
    codeword[CCSDS_RS_DATA_SYMBOLS..].copy_from_slice(&parity);

    codeword
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::assert_bit_exact;

    const LIBFEC_RAMP: &[u8; CCSDS_RS_CODEWORD_SYMBOLS] =
        include_bytes!("../tests/vectors/rs_ccsds/libfec_ramp.bin");

    const LIBFEC_ALTERNATING: &[u8; CCSDS_RS_CODEWORD_SYMBOLS] =
        include_bytes!("../tests/vectors/rs_ccsds/libfec_alternating.bin");

    const LIBFEC_LCG: &[u8; CCSDS_RS_CODEWORD_SYMBOLS] =
        include_bytes!("../tests/vectors/rs_ccsds/libfec_lcg.bin");

    fn assert_matches_libfec(
        expected: &[u8; CCSDS_RS_CODEWORD_SYMBOLS],
    ) {
        let mut data = [0_u8; CCSDS_RS_DATA_SYMBOLS];
        data.copy_from_slice(&expected[..CCSDS_RS_DATA_SYMBOLS]);

        let actual = ccsds_rs_255_223_encode(&data);

        assert_bit_exact!(actual, expected);
    }

    // CCSDS 131.0-B-5 Annex F, Example 1:
    //
    // dual basis:
    //     10111001 = 0xB9
    //
    // conventional basis:
    //     00101010 = 0x2A
    #[test]
    fn test_dual_to_conventional_matches_ccsds_annex_f_example_1() {
        let actual = [dual_to_conventional(0xB9)];
        let expected = [0x2A];

        assert_bit_exact!(actual, expected);
    }

    // CCSDS 131.0-B-5 Annex F, Example 2:
    //
    // conventional basis:
    //     01011001 = 0x59
    //
    // dual basis:
    //     11101000 = 0xE8
    #[test]
    fn test_conventional_to_dual_matches_ccsds_annex_f_example_2() {
        let actual = [conventional_to_dual(0x59)];
        let expected = [0xE8];

        assert_bit_exact!(actual, expected);
    }

    #[test]
    fn test_basis_transform_round_trip_all_symbols() {
        for symbol in u8::MIN..=u8::MAX {
            assert_eq!(
                conventional_to_dual(dual_to_conventional(symbol)),
                symbol
            );

            assert_eq!(
                dual_to_conventional(conventional_to_dual(symbol)),
                symbol
            );
        }
    }

    #[test]
    fn test_zero_information_produces_zero_parity() {
        let data = [0_u8; CCSDS_RS_DATA_SYMBOLS];

        let actual = ccsds_rs_255_223_parity(&data);
        let expected = [0_u8; CCSDS_RS_PARITY_SYMBOLS];

        assert_bit_exact!(actual, expected);
    }

    #[test]
    fn test_codeword_is_systematic() {
        let mut data = [0_u8; CCSDS_RS_DATA_SYMBOLS];

        for (index, value) in data.iter_mut().enumerate() {
            *value = u8::try_from(index).expect("223-symbol index fits in u8");
        }

        let codeword = ccsds_rs_255_223_encode(&data);

        assert_bit_exact!(
            &codeword[..CCSDS_RS_DATA_SYMBOLS],
            data
        );
    }

    #[test]
    fn test_ccsds_rs_matches_libfec_ramp_vector() {
        assert_matches_libfec(LIBFEC_RAMP);
    }

    #[test]
    fn test_ccsds_rs_matches_libfec_alternating_vector() {
        assert_matches_libfec(LIBFEC_ALTERNATING);
    }

    #[test]
    fn test_ccsds_rs_matches_libfec_lcg_vector() {
        assert_matches_libfec(LIBFEC_LCG);
    }
}