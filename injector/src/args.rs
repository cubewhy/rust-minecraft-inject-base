use clap::Parser;

#[derive(Parser, Debug)]
pub struct ProgramArgs {
    #[clap(long)]
    pub pid: u32,

    #[clap(long, default_value = "hook_dll.dll")]
    pub dll: String,

    #[clap(long, default_value = "entrypoint.jar")]
    pub jar: String,

    #[clap(long, default_value = "org.cubewhy.TweakEntrypoint.init")]
    pub entrypoint: String,

    #[clap(long, default_value = "")]
    pub args: String,
}
