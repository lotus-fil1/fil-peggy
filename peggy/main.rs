use app::App;
use clap::Parser;
use anyhow::{anyhow, Error};
use logger;

#[tokio::main]
async fn main() -> Result<(), Error> {
    logger::initialize();

    let app = App::parse();
    let cmd =  match app.cmd.parse() {
        Ok(cmd) => Ok(cmd),
        Err(err) => Err(anyhow!("{}", err)),
    };
    match cmd?.run() {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!("{}", err)),
    }
}
