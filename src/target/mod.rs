use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum Target {
    WindowsX86_64,
    LinuxX86_64,
    Javascript,
    IR,
    Bytecode,
    ObjectFile,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::WindowsX86_64 => f.write_str("windows-x86_64"),
            Target::LinuxX86_64 => f.write_str("linux-x86_64"),
            Target::Javascript => f.write_str("javascript"),
            Target::IR => f.write_str("ir"),
            Target::Bytecode => f.write_str("bytecode"),
            Target::ObjectFile => f.write_str("objectfile"),
        }
    }
}

impl FromStr for Target {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linux-x86_64" => Ok(Self::LinuxX86_64),
            "windows-x86_64" => Ok(Self::WindowsX86_64),
            "ir" => Ok(Self::IR),
            "javascript" => Ok(Self::Javascript),
            "bytecode" => Ok(Self::Bytecode),
            "objectfile" => Ok(Self::ObjectFile),
            _ => Err(()),
        }
    }
}
