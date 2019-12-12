/// Command line interface
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose:   u8,
    /// Use minimal format instead of full format
    #[structopt(short, long)]
    pub minimal:   bool,
    /// Prefix for output
    #[structopt(long, env = "VCP_PREFIX", default_value = " ")]
    pub prefix:    String,
    /// VCS name or symbol
    #[structopt(long, env = "VCP_NAME", default_value = "{symbol}")]
    pub name:      String,
    /// Current branch
    #[structopt(long, env = "VCP_BRANCH", default_value = "{blue}{value}{reset}")]
    pub branch:    String,
    /// Current operation
    #[structopt(long, env = "VCP_OPERATION", default_value = "{red}{value}{reset}")]
    pub operation: String,
}

impl Opt {
    pub fn parse_args() -> Self {
        Opt::from_args()
    }
}
