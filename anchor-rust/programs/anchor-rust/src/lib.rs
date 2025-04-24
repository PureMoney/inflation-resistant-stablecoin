#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("HFd983MMdNSdTxur7MKiYrDkFaftNxQnYGra4ypYciSn");

#[program]
pub mod anchor_rust {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
