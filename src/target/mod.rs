#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Target {
    X86_64Windows,
    X86_64Linux,
    JavaScript,
    IR,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::X86_64Windows => f.write_str("windows-x86_64"),
            Target::X86_64Linux => f.write_str("linux-x86_64"),
            Target::JavaScript => f.write_str("javaScript"),
            Target::IR => f.write_str("ir"),
        }
    }
}
