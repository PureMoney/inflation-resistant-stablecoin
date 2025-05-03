#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("aSEXhjDC3inoAK5DviWkw3mPujzRYibQqCRDGN7hg9r");

#[program]
pub mod irma {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
