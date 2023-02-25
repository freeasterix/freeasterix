//! List of all units of measurements used in ASTERIX protocol.

/// Unit of measurement
#[derive(Debug)]
pub enum Unit {
    /// Unitless measure
    Unitless,
    /// Percent, 1 part per 100
    Percent,
    /// Second, unit of time
    Second,
    /// Milliseconds, 1**10-3 second
    Millisecond,
    /// Nanoseconds, 1**10-9 second
    NanoSecond,
    /// Femtoseconds, 1**10-15 second
    FemtoSecond,
    /// Degree (angle)
    Degree,
    /// Degree per second (angular speed)
    DegreePerSecond,
    /// Degree Celsius (temperature)
    Celsius,
    /// Flight Level, unit of altitude (expressed in 100's of feet)
    FlightLevel,
    /// MegaHertz
    MHz,
    /// Meter
    Meter,
    /// Kilometer
    KiloMeter,
    /// Meter per second
    MeterPerSecond,
    /// Meter per second squared, acceleration
    MeterPerSecondSq,
    /// Mach number
    Mach,
    /// Nautical Mile, 1852 meters, or 6076 feet
    NauticalMile,
    /// Knot, 1 nautical mile/hour, or 1.852 kilometers / hour
    Knot,
    /// Nautical mile per second, or 1852 meters / second
    NauticalMilePerSecond,
    /// Feet, 0.3048 meters
    Feet,
    /// Feet per minute
    FeetPerMinute,
    /// Hectopascal, 100 Pascal
    HectoPascal,
    /// Millibar, 1*10-3 Bar, 100 Pascal
    Millibar,
    /// The dB is the unit of relative power
    #[allow(non_camel_case_types)]
    dB,
    /// The dBm is the unit of absolute power related to 1 milliwatt
    #[allow(non_camel_case_types)]
    dBm,
    /// The dBuV is the unit of absolute power related to 1 microvolt.
    /// `P_{dBm} = E_{dBuV} - 95.2dB`
    /// Only met in XML specs.
    #[allow(non_camel_case_types)]
    dBuV,
    /// Minute, XML-spec specific
    Minute,
    /// Kilobytes per second, XML-spec specific
    KBytesPerSecond,
}

use std::str::FromStr;
impl FromStr for Unit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Unit::*;
        // Note: cat008 contains notes inside units ¯\_(ツ)_/¯
        let s = s.split(" /Note:").next().unwrap();
        let rv = match s {
            "" => Unitless,
            "%" => Percent,
            "s" => Second,
            "ms" => Millisecond,
            "ns" => NanoSecond,
            "fs" => FemtoSecond,
            "°" => Degree,
            "°/s" => DegreePerSecond,
            "°C" => Celsius,
            "FL" => FlightLevel,
            "MHz" => MHz,
            "m" => Meter,
            "km" => KiloMeter,
            "m/s" => MeterPerSecond,
            "m/s2" => MeterPerSecondSq,
            "Mach" => Mach,
            "NM" => NauticalMile,
            "kt" => Knot,
            "NM/s" => NauticalMilePerSecond,
            "ft" => Feet,
            "ft/min" => FeetPerMinute,
            "hPa" => HectoPascal,
            "mb" => Millibar,
            "dB" => dB,
            "dBm" => dBm,
            // Note: special case for XML spec, which has scaling factor
            // embedded in the unit for some reason.
            "*2^(-6+f)" => NauticalMile,
            "* 2^(-6+f)" => NauticalMile,
            "* 2^(-7+f)" => NauticalMile,
            // Units from obsolete specs
            // "/1000" => Unitless,
            // XML-specific unit names, because XML spec isn't consistent
            // TODO: what's up with these gain units in XML spec?
            "*10E-5" => Unitless,
            "dBuV" => dBuV,
            "min" => Minute,
            "KBytes/s" => KBytesPerSecond,
            "sec" => Second,
            "deg" => Degree,
            "deg." => Degree,
            "deg/s" => DegreePerSecond,
            "deg/sec" => DegreePerSecond,
            "C" => Celsius,
            "deg C" => Celsius,
            "m/sec" => MeterPerSecond,
            "m/s^2" => MeterPerSecondSq,
            "m/s²" => MeterPerSecondSq,
            "Nm" => NauticalMile,
            "knot" => Knot,
            "feet/minute" => FeetPerMinute,
            "mbar" => Millibar,
            _ => return Err(format!("unknown unit `{s}`")),
        };
        Ok(rv)
    }
}

impl Unit {
    fn to_str(&self) -> &'static str {
        use Unit::*;
        match self {
            Unitless => "",
            Percent => "%",
            Second => "s",
            Millisecond => "ms",
            NanoSecond => "ns",
            FemtoSecond => "fs",
            Degree => "°",
            DegreePerSecond => "°/s",
            Celsius => "°C",
            FlightLevel => "FL",
            MHz => "MHz",
            Meter => "m",
            KiloMeter => "km",
            MeterPerSecond => "m/s",
            MeterPerSecondSq => "m/s2",
            Mach => "Mach",
            NauticalMile => "NM",
            NauticalMilePerSecond => "NM/s",
            Feet => "ft",
            FeetPerMinute => "ft/min",
            Knot => "kt",
            HectoPascal => "hPa",
            Millibar => "mb",
            dB => "dB",
            dBm => "dBm",
            // XML specific
            Minute => "min",
            dBuV => "dBuV",
            KBytesPerSecond => "KBytes/s",
        }
    }
}

use std::fmt;
impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
impl Serialize for Unit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_str())
    }
}

impl<'de> Deserialize<'de> for Unit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let val = Unit::from_str(&s).map_err(de::Error::custom)?;
        Ok(val)
    }
}
