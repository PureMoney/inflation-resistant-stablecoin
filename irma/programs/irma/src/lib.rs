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

        require!(irma_amount > 0, CustomError::InvalidAmount);

        state.reduce_circulations(&quote_token, irma_amount)?;

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

impl State {
    fn reduce_circulations(&mut self, quote_token: &String, irma_amount: u64) -> Result<()> {
        let mut price_differences: Vec<(&String, i64)> = self.backing_reserves.iter()
            .filter_map(|(token, reserve)| {
                let circulation = self.irma_in_circulation.get(token)?;
                let redemption_price = *reserve as i64 / *circulation as i64;
                let mint_price = self.mint_price as i64;
                Some((token, mint_price - redemption_price))
            })
            .collect();

        price_differences.sort_by(|a, b| b.1.cmp(&a.1));

        let first_target = price_differences.first().ok_or(CustomError::InvalidQuoteToken)?.0;

        if first_target == quote_token {
            let circulation = self.irma_in_circulation.get_mut(first_target).ok_or(CustomError::InvalidQuoteToken)?;
            require!(*circulation >= irma_amount, CustomError::InsufficientCirculation);
            *circulation -= irma_amount;
            return Ok(());
        }

        let first_circulation = self.irma_in_circulation.get_mut(first_target).ok_or(CustomError::InvalidQuoteToken)?;
        let second_circulation = self.irma_in_circulation.get_mut(quote_token).ok_or(CustomError::InvalidQuoteToken)?;

        let first_reserve = self.backing_reserves.get(first_target).ok_or(CustomError::InvalidQuoteToken)?;
        let second_reserve = self.backing_reserves.get(quote_token).ok_or(CustomError::InvalidQuoteToken)?;

        let mut first_price_diff = (*first_reserve as i64 / *first_circulation as i64) - self.mint_price as i64;
        let mut second_price_diff = (*second_reserve as i64 / *second_circulation as i64) - self.mint_price as i64;

        if irma_amount as i64 <= (first_price_diff - second_price_diff).abs() {
            *first_circulation -= irma_amount;
        } else {
            let adjustment_amount = ((first_price_diff - second_price_diff).abs()) as u64;
            *first_circulation -= adjustment_amount;
            *second_circulation -= irma_amount - adjustment_amount;
        }

        Ok(())
    }
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
