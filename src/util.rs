use core::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
pub struct DisplayBool(pub bool);

impl From<bool> for DisplayBool {
    fn from(b: bool) -> Self {
        DisplayBool(b)
    }
}

impl From<DisplayBool> for bool {
    fn from(b: DisplayBool) -> Self {
        b.0
    }
}

impl fmt::Display for DisplayBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 {
            f.write_str("Y")
        } else {
            f.write_str("N")
        }
    }
}

/// Convert degrees celsius (°C) to fahrenheit (°F)
pub fn celsius_to_fahrenheit(c: f32) -> f32 {
    (c * 1.8) + 32.0
}
