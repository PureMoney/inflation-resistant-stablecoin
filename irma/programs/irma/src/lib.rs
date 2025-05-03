#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use std::collections::HashMap;

declare_id!("aSEXhjDC3inoAK5DviWkw3mPujzRYibQqCRDGN7hg9r");

#[program]
pub mod irma {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn set_mint_price(ctx: Context<SetMintPrice>, mint_price: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.mint_price = mint_price;
        Ok(())
    }

    pub fn mint_irma(ctx: Context<MintIrma>, quote_token: String, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let backing_reserve = state.backing_reserves.entry(quote_token.clone()).or_insert(0);
        let circulation = state.irma_in_circulation.entry(quote_token.clone()).or_insert(0);

        require!(amount > 0, CustomError::InvalidAmount);
        require!(state.mint_price > 0, CustomError::MintPriceNotSet);

        *backing_reserve += amount;
        *circulation += amount / state.mint_price;

        Ok(())
    }

    pub fn redeem_irma(ctx: Context<RedeemIrma>, quote_token: String, irma_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let backing_reserve = state.backing_reserves.get_mut(&quote_token).ok_or(CustomError::InvalidQuoteToken)?;
        let circulation = state.irma_in_circulation.get_mut(&quote_token).ok_or(CustomError::InvalidQuoteToken)?;

        require!(irma_amount > 0, CustomError::InvalidAmount);
        require!(*circulation >= irma_amount, CustomError::InsufficientCirculation);

        let redemption_price = *backing_reserve / *circulation;
        let redeemed_amount = irma_amount * redemption_price;

        require!(*backing_reserve >= redeemed_amount, CustomError::InsufficientReserve);

        *backing_reserve -= redeemed_amount;
        *circulation -= irma_amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct SetMintPrice<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
}

#[derive(Accounts)]
pub struct MintIrma<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
}

#[derive(Accounts)]
pub struct RedeemIrma<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
}

#[account]
pub struct State {
    pub mint_price: u64,
    pub backing_reserves: HashMap<String, u64>,
    pub irma_in_circulation: HashMap<String, u64>,
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid amount provided.")]
    InvalidAmount,
    #[msg("Mint price not set.")]
    MintPriceNotSet,
    #[msg("Invalid quote token.")]
    InvalidQuoteToken,
    #[msg("Insufficient circulation.")]
    InsufficientCirculation,
    #[msg("Insufficient reserve.")]
    InsufficientReserve,
}
