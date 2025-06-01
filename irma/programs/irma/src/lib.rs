#![allow(unexpected_cfgs)]
// #![feature(trivial_bounds)]
// #[cfg(feature = "idl-build")]

use anchor_lang::prelude::*;
use anchor_lang::*;

use crate::Stablecoins::*;

// The number of stablecoins that are currently supported by the IRMA program.
pub const BACKING_COUNT: usize = Stablecoins::USDE as usize;

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
        USDT, // <== from Tether, $2.39 B in circulation
        USDC, // <== from Circle, $8.9 B in circulation
        USDS, // <== from Sky (previously MakerDAO) #19, $82. M in circulation
        PYUSD, // <== from PayPal #98, $224 M in circulation
        USDG, // <== from Singapore #263, $96 M in circulation
        FDUSD, // <== First Digital USD, $104 M in circulation
        USDE, // from Ethena #31, $9.9 M in circulation
        USDP, // from Paxos #551, $1.66 M in circulation
        SUSD, // from Solayer, has 4 to 5% yield #839, $13.9 M in circulation
        ZUSD, // from GMO-Z #1165, $8.9 M in circulation
        USDR, // from StabIR #1884, does not exist on Solana yet
        DAI,  // thru Wormhole, very low liquidity in Solana
        USD1,
        EnumCount
    }

    impl Stablecoins {
        pub fn from_index(index: usize) -> Option<Self> {
            match index {
                0 => Some(Stablecoins::USDT),
                1 => Some(Stablecoins::USDC),
                2 => Some(Stablecoins::USDS),
                3 => Some(Stablecoins::PYUSD),
                4 => Some(Stablecoins::USDG),
                5 => Some(Stablecoins::FDUSD),
                6 => Some(Stablecoins::USDE),
                7 => Some(Stablecoins::USDP),
                8 => Some(Stablecoins::SUSD),
                9 => Some(Stablecoins::ZUSD),
                10 => Some(Stablecoins::USDR),
                11 => Some(Stablecoins::DAI),
                12 => Some(Stablecoins::USD1),
                _ => None,
            }
        }

        pub fn to_index(&self) -> usize {
            match self {
                Stablecoins::USDT => 0,
                Stablecoins::USDC => 1,
                Stablecoins::USDS => 2,
                Stablecoins::PYUSD => 3,
                Stablecoins::USDG => 4,
                Stablecoins::FDUSD => 5,
                Stablecoins::USDE => 6,
                Stablecoins::USDP => 7,
                Stablecoins::SUSD => 8,
                Stablecoins::ZUSD => 9,
                Stablecoins::USDR => 10,
                Stablecoins::DAI => 11,
                Stablecoins::USD1 => 12,
                Stablecoins::EnumCount => Stablecoins::EnumCount as usize,
            }
        }

        pub fn to_string(&self) -> String {
            match self {
                Stablecoins::USDT => "USDT".to_string(),
                Stablecoins::USDC => "USDC".to_string(),
                Stablecoins::USDS => "USDS".to_string(),
                Stablecoins::PYUSD => "PYUSD".to_string(),
                Stablecoins::USDG => "USDG".to_string(),
                Stablecoins::FDUSD => "FDUSD".to_string(),
                Stablecoins::USDE => "USDE".to_string(),
                Stablecoins::USDP => "USDP".to_string(),
                Stablecoins::SUSD => "SUSD".to_string(),
                Stablecoins::ZUSD => "ZUSD".to_string(),
                Stablecoins::USDR => "USDR".to_string(),
                Stablecoins::DAI => "DAI".to_string(),
                Stablecoins::USD1 => "USD1".to_string(),
                Stablecoins::EnumCount => "EnumCount".to_string(),
            }
        }

        pub fn from_string(s: &str) -> Self {
            match s {
                "USDT" => Stablecoins::USDT,
                "USDC" => Stablecoins::USDC,
                "USDS" => Stablecoins::USDS,
                "PYUSD" => Stablecoins::PYUSD,
                "USDG" => Stablecoins::USDG,
                "FDUSD" => Stablecoins::FDUSD,
                "USDE" => Stablecoins::USDE,
                "USDP" => Stablecoins::USDP,
                "SUSD" => Stablecoins::SUSD,
                "ZUSD" => Stablecoins::ZUSD,
                "USDR" => Stablecoins::USDR,
                "DAI" => Stablecoins::DAI,
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
        state.backing_decimals = Vec::<u8>::with_capacity(EnumCount as usize);
        state.mint_price = vec![1.0; BACKING_COUNT];
        msg!("Vec length: {:?}", state.mint_price.len());
        state.irma_in_circulation = vec![1; BACKING_COUNT];
        state.backing_reserves = vec![0; BACKING_COUNT];
        // USDR and USD1 are not yet in Solana, so we set their decimals to 
        // the following are also set to 0 (disabled): USDE, USDP, SUSD, ZUSD, and DAI.
        state.backing_decimals = vec![6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0];
        state.bump = 13u8; // Bump seed for the PDA
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

    /// SetMintPrice of IRMA expressed in terms of a given quote token.
    /// This should be called for every backing stablecoin supported, only once per day
    /// because Truflation updates the inflation data only once per day.
    pub fn set_mint_price(ctx: Context<SetMintPrice>, quote_token: Stablecoins, mint_price: f64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.backing_decimals[quote_token.to_index()] > 0, CustomError::InvalidQuoteToken);
        
        let curr_price = state.mint_price.get_mut(quote_token as usize).unwrap();
        require!(mint_price > 0.0, CustomError::InvalidAmount);
        *curr_price = mint_price;
        Ok(())
    }

    /// Mint IRMA tokens for a given amount of quote token.
    /// FIXME: Currently assumes that decimal point is zero digits for both IRMA and quote token.
    pub fn mint_irma(ctx: Context<MintIrma>, quote_token: Stablecoins, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);

        let state: &mut Account<'_, State> = &mut ctx.accounts.state;
        require!(state.backing_decimals[quote_token.to_index()] > 0, CustomError::InvalidQuoteToken);

        let backing_reserve: &mut u64 = state.backing_reserves.get_mut(quote_token as usize).unwrap();
        // require!(*backing_reserve > 0, CustomError::InsufficientReserve);
        *backing_reserve += amount;

        let curr_price: &mut f64 = state.mint_price.get_mut(quote_token as usize).unwrap();
        require!(*curr_price > 0.0, CustomError::MintPriceNotSet);

        let price: f64 = (*curr_price).clone();

        let circulation: &mut u64 = state.irma_in_circulation.get_mut(quote_token as usize).unwrap();
        require!(*circulation > 0, CustomError::InsufficientCirculation);

        *circulation += (amount as f64 / price).ceil() as u64;

        Ok(())
    }

    /// RedeemIRMA - user surrenders IRMA in irma_amount, expecting to get back quote_token according to redemption price.
    /// FIXME: If resulting redemption price increases by more than 0.0000001, then actual redemption price should be updated.
    pub fn redeem_irma(ctx: Context<RedeemIrma>, quote_token: Stablecoins, irma_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.backing_decimals[quote_token.to_index()] > 0, CustomError::InvalidQuoteToken);

        if irma_amount == 0 { return Ok(()) };

        state.reduce_circulations(quote_token, irma_amount)?;

        Ok(())
    }

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
        pub backing_decimals: Vec<u8>,
        #[max_len(BACKING_COUNT)]
        pub irma_in_circulation: Vec<u64>,
        pub bump: u8,
    }


    /// ReduceCirculation implementation
    /// FIXME: Allow for mint_price being less than redemption_price (a period of deflation).
    impl State {

        fn reduce_circulations(&mut self, quote_token: Stablecoins, irma_amount: u64) -> Result<()> {
            require!(irma_amount > 0, CustomError::InvalidAmount);
            require!((quote_token as usize) < BACKING_COUNT, CustomError::InvalidQuoteToken);
            require!(self.mint_price.len() > 0, CustomError::MintPriceNotSet);
            require!(self.backing_reserves.len() > 0, CustomError::InsufficientReserve);
            require!(self.irma_in_circulation.len() > 0, CustomError::InsufficientCirculation);
            // determine what this redemption does:
            // does it keep the relative spreads even, or does it skew the spreads?
            let mut count: u8 = 0;
            let mut average_diff: f64 = 0.0;
            let price_differences : Vec<f64> = self.backing_reserves.iter()
                .enumerate()
                .filter_map(|(i, reserve)| {
                    let circulation = self.irma_in_circulation[i];
                    let redemption_price = *reserve as f64 / circulation as f64;
                    let mint_price = self.mint_price[i];
                    if mint_price == 0.0 || self.backing_decimals[i] == 0 {
                        msg!("Skipping index {}: mint_price is 0.0 or backing_decimals is 0", i);
                        return Some(0.0);
                    }
                    count += 1;
                    let x: f64 = mint_price - redemption_price;
                    average_diff += x;
                    Some(x)
                })
                .collect();
            if count == 0 {
                msg!("No price differences found, returning early.");
                return Ok(());
            }
            average_diff /= count as f64;
            msg!("Average price difference: {}", average_diff);

            let min_diff = 0.1;
            let mut max_price_diff = average_diff;
            let mut first_target = quote_token;
            for (i, price_diff) in price_differences.iter().enumerate() {
                msg!("{}: {}", i, *price_diff);
                if (*price_diff - max_price_diff).abs() > min_diff {
                    max_price_diff = *price_diff;
                    first_target = Stablecoins::from_index(i).unwrap();
                }
            }
            // max_price_diff = (max_price_diff - average_diff).abs();
            // msg!("Max token: {}", first_target.to_string());
            msg!("Max price diff: {}", max_price_diff);

            // if max price diff does not deviate much from average diff or all inflation-adjusted prices 
            // are less than the redemption prices, then redemption price calc is normal.
            if (first_target == quote_token) || ((max_price_diff - average_diff).abs() < min_diff)
                || (average_diff < 0.0) {
                let circulation: &mut u64 = self.irma_in_circulation.get_mut(quote_token as usize).unwrap();
                if price_differences[quote_token.to_index()] > 0.0 {
                    require!(*circulation >= irma_amount, CustomError::InsufficientCirculation);
                    *circulation -= irma_amount;
                }
                let reserve: &mut u64 = self.backing_reserves.get_mut(quote_token as usize).unwrap();
                let redemption_price: f64 = *reserve as f64 / *circulation as f64;
                let backing_amount: u64 = (irma_amount as f64 * redemption_price) as u64;
                require!(*reserve >= backing_amount, CustomError::InsufficientReserve);
                *reserve -= backing_amount;
                msg!("Redeemed {} IRMA for {} backing tokens.", irma_amount, backing_amount);
                msg!("New reserve for {}: {}", quote_token.to_string(), *reserve);
                msg!("New circulation for {}: {}", quote_token.to_string(), *circulation);
                return Ok(());
            }
            msg!("First target: {}", first_target.to_string());

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
            if first_circulation < irma_amount {
                require!(second_circulation >= irma_amount, CustomError::InsufficientCirculation);
                let second_circulation = self.irma_in_circulation.get_mut(quote_token as usize).unwrap();
                *second_circulation -= irma_amount;
                return Ok(());
            }

            let first_price_diff = first_price - (first_reserve / first_circulation) as f64;
            let second_price_diff = second_price - (second_reserve / second_circulation) as f64;

            let post_first_price_diff = first_price - (first_reserve as f64 / (first_circulation - irma_amount) as f64);

            if post_first_price_diff <= second_price_diff {
                // if irma_amount is such that conditions would remain the same post adjustment
                // we can just subtract from the first_circulation
                let first_circulation = self.irma_in_circulation.get_mut(first_target as usize).unwrap();
                *first_circulation -= irma_amount;
            } else if (second_price - first_price as f64).abs() < 0.01 {
                // firt and second prices are close enough, need to do linear adjustment
                // of both first and second circulations
                let adjustment_amount = irma_amount as f64 * (first_price_diff - second_price_diff) / (first_price_diff + second_price_diff);
                let first_circulation = self.irma_in_circulation.get_mut(first_target as usize).unwrap();
                *first_circulation -= adjustment_amount.ceil() as u64;
                let second_circulation = self.irma_in_circulation.get_mut(quote_token as usize).unwrap();
                *second_circulation -= irma_amount - adjustment_amount.ceil() as u64;
            } else {
                // firt and second prices are not close, need to do quadratic adjustment
                let p = second_price - first_price;
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
        #[msg("Invalid reserve value.")]
        InvalidBacking,
        #[msg("Invalid IRMA amount.")]
        InvalidIrmaAmount,
    }
}
