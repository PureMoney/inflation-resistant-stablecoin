#![allow(unexpected_cfgs)]
#![feature(trivial_bounds)]
// #[cfg(feature = "idl-build")]

// use anchor_lang::prelude::*;
use anchor_lang::prelude::Account;
use anchor_lang::prelude::AccountInfo;
use anchor_lang::prelude::Context;
use anchor_lang::prelude::msg;
use anchor_lang::prelude::Program;
use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::Rent;
use anchor_lang::prelude::Signer;
use anchor_lang::prelude::SolanaSysvar;
use anchor_lang::prelude::System;
use anchor_lang::prelude::*;
// // use anchor_lang::solana_program::borsh;
use anchor_lang::program;
use anchor_lang::require_gte;
use anchor_lang::require_eq;
use anchor_lang::require;
use anchor_lang::account;
use anchor_lang::Accounts;
use anchor_lang::error;
use anchor_lang::error_code;
use anchor_lang::declare_id;
use anchor_lang::AccountDeserialize;
use anchor_lang::AccountSerialize;
use anchor_lang::AnchorSerialize;
use anchor_lang::AnchorDeserialize;
use anchor_lang::AccountsExit;
use anchor_lang::Discriminator;
// use anchor_lang::IdlBuild;
use anchor_lang::Key;
use anchor_lang::ToAccountInfo;

// // use anchor_lang::prelude::*;
// // use crate::borsh::maybestd::collections::HashMap;
use anchor_lang::prelude::borsh;
// // use borsh::maybestd::collections::BTreeMap;
// use std::collections::Vec;
// use crate::error::Error;

use crate::Stablecoins::*;

declare_id!("aSEXhjDC3inoAK5DviWkw3mPujzRYibQqCRDGN7hg9r");

#[program]
pub mod irma {
    use super::*;

    // All currently existing stablecoins with over $1B in circulation
    // are supported. This list is not exhaustive and will be updated as new
    // stablecoins are added to the market.
    // Initially, we will support only those stablecoins that exist
    // on the Solana blockchain (the first three below). 
    #[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
    pub enum Stablecoins {
        USDT,
        USDC,
        USDS, // from Sky (previously MakerDAO) #19
        USDE, // from Ethena #31
        PYUSD, // from PayPal #98
        USDG, // from Singapore #263
        USDP, // from Paxos #551
        SUSD, // from Solayer, has 4 to 5% yield #839
        ZUSD, // from GMO-Z #1165
        USDR, // from StabIR #1884
        DAI,
        FDUSD,
        USD1,
        INVALID
    }

    impl Stablecoins {
        pub fn from_index(index: usize) -> Option<Self> {
            match index {
                0 => Some(USDT),
                1 => Some(USDC),
                2 => Some(USDS),
                3 => Some(USDE),
                4 => Some(PYUSD),
                5 => Some(USDG),
                6 => Some(USDP),
                7 => Some(SUSD),
                8 => Some(ZUSD),
                9 => Some(USDR),
                10 => Some(DAI),
                11 => Some(FDUSD),
                12 => Some(USD1),
                _ => None,
            }
        }

        pub fn to_index(&self) -> usize {
            match self {
                USDT => 0,
                USDC => 1,
                USDS => 2,
                USDE => 3,
                PYUSD => 4,
                USDG => 5,
                USDP => 6,
                SUSD => 7,
                ZUSD => 8,
                USDR => 9,
                DAI => 10,
                FDUSD => 11,
                USD1 => 12,
                INVALID => INVALID as usize,
            }
        }

        pub fn to_string(&self) -> String {
            match self {
                USDT => "USDT".to_string(),
                USDC => "USDC".to_string(),
                USDS => "USDS".to_string(),
                USDE => "USDE".to_string(),
                PYUSD => "PYUSD".to_string(),
                USDG => "USDG".to_string(),
                USDP => "USDP".to_string(),
                SUSD => "SUSD".to_string(),
                ZUSD => "ZUSD".to_string(),
                USDR => "USDR".to_string(),
                DAI => "DAI".to_string(),
                FDUSD => "FDUSD".to_string(),
                USD1 => "USD1".to_string(),
                INVALID => "INVALID".to_string(),
            }
        }

        pub fn from_string(s: &str) -> Self {
            match s {
                "USDT" => USDT,
                "USDC" => USDC,
                "USDS" => USDS,
                "USDE" => USDE,
                "PYUSD" => PYUSD,
                "USDG" => USDG,
                "USDP" => USDP,
                "SUSD" => SUSD,
                "ZUSD" => ZUSD,
                "USDR" => USDR,
                "DAI" => DAI,
                "FDUSD" => FDUSD,
                "USD1" => USD1,
                _ => INVALID,
            }
        }
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        let state = &mut ctx.accounts.state;
        if state.mint_price.len() > 0 {
            return Ok(());
        }
        state.mint_price = Vec::<f64>::with_capacity(INVALID as usize);
        msg!("Vec capacity: {:?}", state.mint_price.capacity());
        state.backing_reserves = Vec::<u64>::with_capacity(INVALID as usize);
        state.irma_in_circulation = Vec::<u64>::with_capacity(INVALID as usize);
        state.mint_price.push(1.0);
        state.mint_price.push(1.0);
        state.mint_price.push(1.0);
        msg!("Vec length: {:?}", state.mint_price.len());
        state.irma_in_circulation.push(1);
        state.irma_in_circulation.push(1);
        state.irma_in_circulation.push(1);
        state.backing_reserves.push(0);
        state.backing_reserves.push(0);
        state.backing_reserves.push(0);
        Ok(())
    }

    pub fn set_mint_price(ctx: Context<SetMintPrice>, quote_token: Stablecoins, mint_price: f64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let curr_price = state.mint_price.get_mut(quote_token as usize).unwrap();
        require!(mint_price > 0.0, CustomError::InvalidAmount);
        *curr_price = mint_price;
        Ok(())
    }

    pub fn mint_irma(ctx: Context<MintIrma>, quote_token: Stablecoins, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);
        require!(state.mint_price > 0, CustomError::MintPriceNotSet);

        let state = &mut ctx.accounts.state;
        let backing_reserve = state.backing_reserves.get_mut(quote_token as usize).unwrap();
        require!(*backing_reserve > 0, CustomError::InsufficientReserve);
        *backing_reserve += amount;

        let curr_price = state.mint_price.get_mut(quote_token as usize).unwrap();
        require!(*curr_price > 0.0, CustomError::MintPriceNotSet);

        let price = (*curr_price).clone();

        let circulation = state.irma_in_circulation.get_mut(quote_token as usize).unwrap();
        require!(*circulation > 0, CustomError::InsufficientCirculation);

        *circulation += (amount as f64 / price).ceil() as u64;

        Ok(())
    }

    pub fn redeem_irma(ctx: Context<RedeemIrma>, quote_token: Stablecoins, irma_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;

        require!(irma_amount > 0, CustomError::InvalidAmount);

        state.reduce_circulations(quote_token, irma_amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
}

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
    pub mint_price: Vec<f64>,
    pub backing_reserves: Vec<u64>,
    pub irma_in_circulation: Vec<u64>,
}

impl State {

    fn reduce_circulations(&mut self, quote_token: Stablecoins, irma_amount: u64) -> Result<()> {
        let price_differences : Vec<f64> = self.backing_reserves.iter()
            .enumerate()
            .filter_map(|(i, reserve)| {
                let circulation = self.irma_in_circulation[i];
                let redemption_price = *reserve as f64 / circulation as f64;
                let mint_price = self.mint_price[i];
                if mint_price == 0.0 {
                    return None;
                }
                Some(mint_price - redemption_price)
            })
            .collect();

        let mut max_price_diff = 0.0;
        let mut first_target = INVALID;
        for (i, price) in price_differences.iter().enumerate() {
            msg!("{}: {}", i, *price);
            if *price > max_price_diff {
                max_price_diff = *price;
                first_target = Stablecoins::from_index(i).unwrap();
            }
        }
        // msg!("Max token: {}", first_target.to_string());
        msg!("Max price diff: {}", max_price_diff);

        if (first_target == quote_token) || ((max_price_diff <= 0.0) && (first_target == INVALID)) {
            let circulation = self.irma_in_circulation.get_mut(quote_token as usize).unwrap();
            require!(*circulation >= irma_amount, CustomError::InsufficientCirculation);
            *circulation -= irma_amount;
            return Ok(());
        }

        let first_circulation = self.irma_in_circulation[first_target as usize];
        let second_circulation = self.irma_in_circulation[quote_token as usize];

        // if we don't have enough reserve to redeem the irma_amount, just error out;
        // we can't allow redemption from a reserve that is smaller than the irma_amount.
        require!(irma_amount < second_circulation, CustomError::InsufficientCirculation);

        let first_price = self.mint_price[first_target as usize];
        let second_price = self.mint_price[quote_token as usize];
        let first_reserve = self.backing_reserves[first_target as usize];
        let second_reserve = self.backing_reserves[quote_token as usize];

        // we can't subtract from the first_circulation if the first_circulation is less than irma_amount
        // so just subtract from the second_circulation
        if  (first_circulation - irma_amount) <= 0 {
            let second_circulation = self.irma_in_circulation.get_mut(quote_token as usize).unwrap();
            *second_circulation -= irma_amount;
            return Ok(());
        }

        let first_price_diff = first_price as f64 - (first_reserve / first_circulation) as f64;
        let second_price_diff = second_price as f64 - (second_reserve / second_circulation) as f64;

        let post_first_price_diff = first_price as f64 - (first_reserve as f64 / (first_circulation - irma_amount) as f64);

        if post_first_price_diff <= second_price_diff {
            // if irma_amount is such that conditions would remain the same post adjustment
            // we can just subtract from the first_circulation
            let first_circulation = self.irma_in_circulation.get_mut(first_target as usize).unwrap();
            *first_circulation -= irma_amount;
        } else if (second_price as f64 - first_price as f64).abs() < 0.01 {
            // firt and second prices are close enough, need to do linear adjustment
            // of both first and second circulations
            let adjustment_amount = irma_amount as f64 * (first_price_diff - second_price_diff) / (first_price_diff + second_price_diff);
            let first_circulation = self.irma_in_circulation.get_mut(first_target as usize).unwrap();
            *first_circulation -= adjustment_amount.ceil() as u64;
            let second_circulation = self.irma_in_circulation.get_mut(quote_token as usize).unwrap();
            *second_circulation -= irma_amount - adjustment_amount.ceil() as u64;
        } else {
            // firt and second prices are not close, need to do quadratic adjustment
            let p = second_price as f64 - first_price as f64;
            let v = second_circulation as f64 - irma_amount as f64;
            let fc = first_circulation as f64;
            let sc = second_circulation as f64;
            let sr = second_reserve as f64;
            let fr = first_reserve as f64;

            // p is always a much smaller number than the rest of the variables
            let a = -p;
            let b = sr - fr - p * (v - fc);
            let c = p * fc * v - fc * sr + fr * (sc + irma_amount as f64);

            let adjustment_amount = (-b + (b.powf(2.0) - 4.0 * a * c).sqrt()) / (2.0 * a);
            require!(adjustment_amount > 0.0, CustomError::InvalidAmount);
            require!(adjustment_amount < irma_amount as f64, CustomError::InvalidAmount);
            let first_circulation = self.irma_in_circulation.get_mut(first_target as usize).unwrap();
            *first_circulation -= adjustment_amount.ceil() as u64;
            let second_circulation = self.irma_in_circulation.get_mut(quote_token as usize).unwrap();
            *second_circulation -= irma_amount - adjustment_amount.ceil() as u64;
        }

        return Ok(());
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
