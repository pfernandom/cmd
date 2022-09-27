use crate::{ Deps, error::CmdError, models::cmd_record::CmdRecord };

pub trait CmdExtension {
    fn handle_if_match(
        self,
        pattern: &Option<CmdRecord>,
        deps: &mut Deps
    ) -> Option<Result<(), CmdError>>;
}