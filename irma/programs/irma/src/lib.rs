#![allow(unexpected_cfgs)]
// #![feature(trivial_bounds)]
// #[cfg(feature = "idl-build")]

use anchor_lang::prelude::*;
use anchor_lang::*;

use crate::Stablecoins::*;
const BACKING_COUNT: usize = Stablecoins::EnumCount as usize;

declare_id!("8zs1JbqxqLcCXzBrkMCXyY2wgSW8uk8nxYuMFEfUMQa6");

#[program]
pub mod irmamod {
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
        EnumCount
    }

    impl Stablecoins {
        pub fn from_index(index: usize) -> Option<Self> {
            match index {
                0 => Some(Stablecoins::USDT),
                1 => Some(Stablecoins::USDC),
                2 => Some(Stablecoins::USDS),
                3 => Some(Stablecoins::USDE),
                4 => Some(Stablecoins::PYUSD),
                5 => Some(Stablecoins::USDG),
                6 => Some(Stablecoins::USDP),
                7 => Some(Stablecoins::SUSD),
                8 => Some(Stablecoins::ZUSD),
                9 => Some(Stablecoins::USDR),
                10 => Some(Stablecoins::DAI),
                11 => Some(Stablecoins::FDUSD),
                12 => Some(Stablecoins::USD1),
                _ => None,
            }
        }

        pub fn to_index(&self) -> usize {
            match self {
                Stablecoins::USDT => 0,
                Stablecoins::USDC => 1,
                Stablecoins::USDS => 2,
                Stablecoins::USDE => 3,
                Stablecoins::PYUSD => 4,
                Stablecoins::USDG => 5,
                Stablecoins::USDP => 6,
                Stablecoins::SUSD => 7,
                Stablecoins::ZUSD => 8,
                Stablecoins::USDR => 9,
                Stablecoins::DAI => 10,
                Stablecoins::FDUSD => 11,
                Stablecoins::USD1 => 12,
                Stablecoins::EnumCount => Stablecoins::EnumCount as usize,
            }
        }

        pub fn to_string(&self) -> String {
            match self {
                Stablecoins::USDT => "USDT".to_string(),
                Stablecoins::USDC => "USDC".to_string(),
                Stablecoins::USDS => "USDS".to_string(),
                Stablecoins::USDE => "USDE".to_string(),
                Stablecoins::PYUSD => "PYUSD".to_string(),
                Stablecoins::USDG => "USDG".to_string(),
                Stablecoins::USDP => "USDP".to_string(),
                Stablecoins::SUSD => "SUSD".to_string(),
                Stablecoins::ZUSD => "ZUSD".to_string(),
                Stablecoins::USDR => "USDR".to_string(),
                Stablecoins::DAI => "DAI".to_string(),
                Stablecoins::FDUSD => "FDUSD".to_string(),
                Stablecoins::USD1 => "USD1".to_string(),
                Stablecoins::EnumCount => "EnumCount".to_string(),
            }
        }

        pub fn from_string(s: &str) -> Self {
            match s {
                "USDT" => Stablecoins::USDT,
                "USDC" => Stablecoins::USDC,
                "USDS" => Stablecoins::USDS,
                "USDE" => Stablecoins::USDE,
                "PYUSD" => Stablecoins::PYUSD,
                "USDG" => Stablecoins::USDG,
                "USDP" => Stablecoins::USDP,
                "SUSD" => Stablecoins::SUSD,
                "ZUSD" => Stablecoins::ZUSD,
                "USDR" => Stablecoins::USDR,
                "DAI" => Stablecoins::DAI,
                "FDUSD" => Stablecoins::FDUSD,
                "USD1" => Stablecoins::USD1,
                _ => Stablecoins::EnumCount,
            }
        }
    }


    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        let state = &mut ctx.accounts.state;
        if state.mint_price.len() > 0 {
            return Ok(());
        }
        state.mint_price = Vec::<f64>::with_capacity(EnumCount as usize);
        msg!("Vec capacity: {:?}", state.mint_price.capacity());
        state.backing_reserves = Vec::<u64>::with_capacity(EnumCount as usize);
        state.irma_in_circulation = Vec::<u64>::with_capacity(EnumCount as usize);
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

    pub fn hello(ctx: Context<SetMintPrice>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        if state.mint_price.len() == 0 {
            state.mint_price = vec![1.0; BACKING_COUNT];
            state.backing_reserves = vec![0; BACKING_COUNT];
            state.irma_in_circulation = vec![0; BACKING_COUNT];
        }
        msg!("State initialized with mint prices: {:?}", state.mint_price);
        msg!("Backing reserves: {:?}", state.backing_reserves);
        msg!("Irma in circulation: {:?}", state.irma_in_circulation);
        msg!("Program ID: {:?}", ctx.program_id);
        msg!("Hello world...");
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

    /// There are three arrays: irma price, which has f64 (8 bytes) values, backing reserves, which has u64 (8 bytes) each, and 
    /// irma in circulation, which has u64 (8 bytes) each. There's also one byte for decimal points and another byte for the bump.
    /// The total comes out to 3 x 8 + 2 = 26 bytes per backing reserve stablecoin.
    #[derive(Accounts)]
    pub struct Initialize<'info> {
        #[account(init, space=26*BACKING_COUNT, payer=irma_admin, seeds=[b"state".as_ref()], bump)]
        pub state: Account<'info, State>,
        #[account(mut)]
        pub irma_admin: Signer<'info>,
        #[account(address = system_program::ID)]
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct SetMintPrice<'info> {
        #[account(mut, seeds=[b"state".as_ref()], bump)]
        pub state: Account<'info, State>,
        #[account(mut)]
        pub trader: Signer<'info>,
        #[account(address = system_program::ID)]
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct MintIrma<'info> {
        #[account(mut, seeds=[b"state".as_ref()], bump)]
        pub state: Account<'info, State>,
        #[account(mut)]
        pub trader: Signer<'info>,
        #[account(address = system_program::ID)]
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct RedeemIrma<'info> {
        #[account(mut, seeds=[b"state".as_ref()], bump)]
        pub state: Account<'info, State>,
        #[account(mut)]
        pub trader: Signer<'info>,
        #[account(address = system_program::ID)]
        pub system_program: Program<'info, System>,
    }

    #[account]
    #[derive(InitSpace)]
    #[derive(Debug)]
    pub struct State {
        #[max_len(BACKING_COUNT)]
        pub mint_price: Vec<f64>,
        #[max_len(BACKING_COUNT)]
        pub backing_reserves: Vec<u64>,
        #[max_len(BACKING_COUNT)]
        pub irma_in_circulation: Vec<u64>,
        pub bump: u8,
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
            let mut first_target = EnumCount;
            for (i, price) in price_differences.iter().enumerate() {
                msg!("{}: {}", i, *price);
                if *price > max_price_diff {
                    max_price_diff = *price;
                    first_target = Stablecoins::from_index(i).unwrap();
                }
            }
            // msg!("Max token: {}", first_target.to_string());
            msg!("Max price diff: {}", max_price_diff);

            if (first_target == quote_token) || ((max_price_diff <= 0.0) && (first_target == EnumCount)) {
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
}
