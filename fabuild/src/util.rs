use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemArchitecture {
    X86,
    X64,
    Arm64,
    #[default]
    Auto,
}

impl FromStr for SystemArchitecture {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86" => Ok(Self::X86),
            "x64" => Ok(Self::X64),
            "arm64" => Ok(Self::Arm64),
            "auto" => Ok(Self::Auto),
            _ => anyhow::bail!("invalid value for architecture, got: {s}"),
        }
    }
}

impl SystemArchitecture {
    #[must_use]
    pub fn resolve(&self) -> Self {
        if *self != Self::Auto {
            return *self;
        }

        #[cfg(target_arch = "x86_64")]
        return Self::X64;
        #[cfg(target_arch = "x86")]
        return Self::X86;
        #[cfg(any(target_arch = "arm", target_arch = "arm64ec", target_arch = "aarch64"))]
        return Self::Arm64;
    }
}
