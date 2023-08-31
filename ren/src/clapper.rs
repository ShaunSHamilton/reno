use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Log file to write to
    /// If not provided, will **NOT** save logs to a file - only print to stdout
    #[arg(short, long)]
    pub logs: Option<String>,

    /// Peripheral ID
    /// The ID of the bluetooth device to connect to
    #[arg(short = 'i', long)]
    pub peripheral_id: Option<String>,

    /// Peripheral name
    /// The name of the bluetooth device to connect to
    #[arg(short = 'n', long)]
    pub peripheral_name: Option<String>,

    /// How often to poll the bluetooth device [seconds]
    /// If not provided, defaults to 3
    #[arg(short = 't', long, default_value = "3")]
    pub inverval: u64,
}
