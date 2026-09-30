//! Named Q-format and SDR sample-scaling constants.
//!
//! Q-format is deliberately not encoded in the Rust sample type. A
//! `Complex<i16>` may represent USRP Q1.15 samples, BladeRF Q1.11 samples,
//! Pluto Q1.11 samples, or another fixed-point convention.
//!
//! Boundary conversion functions therefore accept a [`QFormat`] describing
//! the interpretation of the integer container.

/// Describes the number of fractional bits in a signed fixed-point sample.
///
/// Honeyeater currently supports `i8` and `i16` kernel samples, so the largest
/// useful value is 15 fractional bits.
///
/// For example:
///
/// - Q1.15 -> scale 32768
/// - Q1.11 -> scale 2048
/// - Q1.7  -> scale 128
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QFormat {
    fractional_bits: u32,
}

impl QFormat {
    /// Creates a Q-format with the supplied number of fractional bits.
    ///
    /// # Panics
    ///
    /// Panics if `fractional_bits > 15`.
    #[must_use]
    pub const fn new(fractional_bits: u32) -> Self {
        assert!(
            fractional_bits <= 15,
            "Q-format fractional bits must be <= 15"
        );

        Self { fractional_bits }
    }

    /// Returns the number of fractional bits.
    #[must_use]
    pub const fn fractional_bits(self) -> u32 {
        self.fractional_bits
    }

    /// Returns the integer scale factor `2^fractional_bits`.
    #[must_use]
    pub const fn scale_u16(self) -> u16 {
        1_u16 << self.fractional_bits
    }

    /// Returns the scale factor as `f32`.
    #[must_use]
    pub fn scale_f32(self) -> f32 {
        f32::from(self.scale_u16())
    }

    /// Returns the scale factor as `f64`.
    #[must_use]
    pub fn scale_f64(self) -> f64 {
        f64::from(self.scale_u16())
    }
}

/// Q1.15 scaling: ±1.0 corresponds to approximately ±32768.
pub const Q1_15: QFormat = QFormat::new(15);

/// Q1.11 scaling: ±1.0 corresponds to approximately ±2048.
pub const Q1_11: QFormat = QFormat::new(11);

/// Q1.7 scaling: ±1.0 corresponds to approximately ±128.
pub const Q1_7: QFormat = QFormat::new(7);

/// Describes an unsigned transport format with a non-zero DC midpoint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnsignedMidpointFormat {
    midpoint: f32,
    scale: f32,
}

impl UnsignedMidpointFormat {
    /// Returns the unsigned DC midpoint.
    #[must_use]
    pub const fn midpoint(self) -> f32 {
        self.midpoint
    }

    /// Returns the normalising scale used after midpoint subtraction.
    #[must_use]
    pub const fn scale(self) -> f32 {
        self.scale
    }
}

/// Describes why a radio or shim does not expose one static Q-format constant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QFormatAvailability {
    /// The stream is floating-point, so fixed-point scaling is not applicable.
    NotApplicable,

    /// The scale depends on the underlying device or runtime configuration.
    DriverDependent,

    /// The available driver documentation does not establish a safe scale.
    Unverified,
}

// ---------------------------------------------------------------------------
// Ettus / NI USRP
// ---------------------------------------------------------------------------

/// Default UHD/SoapyUHD `sc16` scaling.
pub const USRP_SC16: QFormat = Q1_15;

/// UHD/SoapyUHD `sc8` scaling.
pub const USRP_SC8: QFormat = Q1_7;

// ---------------------------------------------------------------------------
// BladeRF
// ---------------------------------------------------------------------------

/// BladeRF `SC16_Q11`.
pub const BLADERF_SC16_Q11: QFormat = Q1_11;

/// BladeRF packed `SC16_Q11_PACKED`.
///
/// Packing changes the wire representation, not the numeric Q-format.
pub const BLADERF_SC16_Q11_PACKED: QFormat = Q1_11;

/// BladeRF `SC8_Q7` high-rate/oversample mode.
pub const BLADERF_SC8_Q7: QFormat = Q1_7;

// ---------------------------------------------------------------------------
// HackRF
// ---------------------------------------------------------------------------

/// HackRF signed 8-bit IQ scaling.
pub const HACKRF_SC8: QFormat = Q1_7;

// ---------------------------------------------------------------------------
// RTL-SDR
// ---------------------------------------------------------------------------

/// Raw RTL-SDR unsigned 8-bit sample representation.
///
/// Raw samples have a DC midpoint of 127.5. Conversion to floating point uses
/// `(sample - 127.5) / 128`.
pub const RTL_SDR_U8: UnsignedMidpointFormat = UnsignedMidpointFormat {
    midpoint: 127.5,
    scale: 128.0,
};

/// RTL-SDR signed 8-bit representation after integer debiasing.
pub const RTL_SDR_SC8: QFormat = Q1_7;

// ---------------------------------------------------------------------------
// Airspy
// ---------------------------------------------------------------------------

/// Airspy R2 / Mini signed-16 host representation.
pub const AIRSPY_R2_SC16: QFormat = Q1_15;

/// Airspy HF+ / Discovery native floating-point stream.
pub const AIRSPYHF_CF32: QFormatAvailability =
    QFormatAvailability::NotApplicable;

// ---------------------------------------------------------------------------
// SDRplay
// ---------------------------------------------------------------------------

/// SDRplay signed-16 host representation.
pub const SDRPLAY_SC16: QFormat = Q1_15;

// ---------------------------------------------------------------------------
// ADALM-Pluto
// ---------------------------------------------------------------------------

/// ADALM-Pluto receive scaling.
pub const PLUTO_RX_SC16_Q11: QFormat = Q1_11;

/// ADALM-Pluto transmit scaling.
pub const PLUTO_TX_SC16_Q15: QFormat = Q1_15;

// ---------------------------------------------------------------------------
// LimeSDR
// ---------------------------------------------------------------------------

/// LimeSDR signed-16 host representation.
pub const LIMESDR_SC16: QFormat = Q1_15;

/// LimeSDR packed `CS12` transport.
///
/// The packed link format transports the same numeric samples represented by
/// the normal signed-16 Q1.15 host convention.
pub const LIMESDR_CS12: QFormat = Q1_15;

// ---------------------------------------------------------------------------
// FUNcube Dongle Pro+
// ---------------------------------------------------------------------------

/// FUNcube Dongle Pro+ signed 16-bit audio-path representation.
pub const FCDPP_SC16: QFormat = Q1_15;

// ---------------------------------------------------------------------------
// Sidekiq
// ---------------------------------------------------------------------------

/// Epiq Sidekiq native signed-16 format.
///
/// SoapySidekiq determines the scale from the specific card's ADC resolution,
/// so there is no single static Q-format valid for every Sidekiq model.
pub const SIDEKIQ_SC16: QFormatAvailability =
    QFormatAvailability::DriverDependent;

// ---------------------------------------------------------------------------
// Mirics
// ---------------------------------------------------------------------------

/// Mirics / MSi2500 signed-16 format.
///
/// The current driver does not establish a trustworthy normalised Q-format,
/// so no numeric scale is exposed.
pub const MIRI_SC16: QFormatAvailability =
    QFormatAvailability::Unverified;

// ---------------------------------------------------------------------------
// Red Pitaya
// ---------------------------------------------------------------------------

/// Red Pitaya STEMlab signed-16 representation.
pub const REDPITAYA_SC16: QFormat = Q1_15;

// ---------------------------------------------------------------------------
// XTRX
// ---------------------------------------------------------------------------

/// Fairwaves XTRX signed-16 representation.
pub const XTRX_SC16: QFormat = Q1_15;

// ---------------------------------------------------------------------------
// Iris
// ---------------------------------------------------------------------------

/// Skylark Iris / Faros signed-16 representation.
pub const IRIS_SC16: QFormat = Q1_15;

// ---------------------------------------------------------------------------
// NetSDR / Afedri
// ---------------------------------------------------------------------------

/// RFSpace NetSDR signed-16 representation.
pub const NETSDR_SC16: QFormat = Q1_15;

/// Afedri signed-16 representation.
pub const AFEDRI_SC16: QFormat = NETSDR_SC16;

// ---------------------------------------------------------------------------
// Soapy shims
// ---------------------------------------------------------------------------

/// SoapyOsmo delegates sample format and scaling to its underlying driver.
pub const SOAPY_OSMO: QFormatAvailability =
    QFormatAvailability::DriverDependent;

/// SoapyAudio supplies floating-point host samples.
pub const SOAPY_AUDIO_CF32: QFormatAvailability =
    QFormatAvailability::NotApplicable;

/// SoapyRemote passes through the remote device's format and scaling.
pub const SOAPY_REMOTE: QFormatAvailability =
    QFormatAvailability::DriverDependent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_format_scales_are_correct() {
        assert_eq!(Q1_15.scale_u16(), 32_768);
        assert_eq!(Q1_11.scale_u16(), 2_048);
        assert_eq!(Q1_7.scale_u16(), 128);
    }

    #[test]
    fn radio_aliases_have_expected_scales() {
        assert_eq!(USRP_SC16, Q1_15);
        assert_eq!(BLADERF_SC16_Q11, Q1_11);
        assert_eq!(BLADERF_SC16_Q11_PACKED, Q1_11);
        assert_eq!(BLADERF_SC8_Q7, Q1_7);
        assert_eq!(PLUTO_RX_SC16_Q11, Q1_11);
        assert_eq!(PLUTO_TX_SC16_Q15, Q1_15);
        assert_eq!(AFEDRI_SC16, NETSDR_SC16);
    }

    #[test]
    fn rtl_sdr_raw_format_has_exact_midpoint() {
        assert_eq!(RTL_SDR_U8.midpoint(), 127.5);
        assert_eq!(RTL_SDR_U8.scale(), 128.0);
    }
}