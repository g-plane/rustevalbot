use futures::{Future, IntoFuture};

use super::{CommandImpl, ExecutionContext};

pub struct AboutCommand;

impl CommandImpl for AboutCommand {
    type Flags = ();

    fn run(
        _ctx: &ExecutionContext,
        _flags: &(),
        _arg: &str,
    ) -> Box<dyn Future<Item = String, Error = &'static str>> {
        Box::new(
            Ok(format!(
                "{} {}\n{}",
                env!("CARGO_PKG_NAME"),
                ::VERSION,
                env!("CARGO_PKG_HOMEPAGE")
            )).into_future(),
        )
    }
}
