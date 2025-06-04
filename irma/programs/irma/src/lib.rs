use anchor_lang::prelude::*;
use openbook_v2::prelude::*;
use crate::irma::{mint_irma, redeem_irma, set_mint_price, Stablecoins};

// Function to consume events from OpenBook V2 and update IRMA mint prices
use openbook_v2::accounts::ConsumeEvents;
use openbook_v2::instructions::consume_events;
use anchor_lang::CpiContext;
use anchor_lang::Result;

declare_id!("8zs1JbqxqLcCXzBrkMCXyY2wgSW8uk8nxYuMFEfUMQa6");

/// There are several markets, one for each stablecoin paired with IRMA.
/// Events are specific to a market, so the number of types of events should match
/// the number of stablecoins in the reserve backing.
#[program]
pub mod irma_program {
    use super::*;

    /// This module listens to cranks from main client running off-chain
    /// and updates IRMA mint prices based on OpenBook V2 market data.
    /// It processes buy and sell events for IRMA tokens and updates the mint prices accordingly.
    /// It also provides a function to handle inflation inputs for the IRMA mint price.
    /// This consumes event data from OpenBook V2 started by the off-chain client.
    /// NOTE: When there is a spread between mint and redemption prices, we
    /// are only interested in traddes that pertain to minting and redemptions of IRMA.
    /// All other trade events are events that happen between other market makers and traders
    /// with price points in between the IRMA mint and redemption prices. 
    /// These are not relevant to the IRMA program.
    pub fn crank(ctx: Context<CrankIrma>) -> Result<()> {
        // First, we enumerate all the markets that we are managing.

        // Second, we collect all the eventHeap data accounts from each of the markets.

        // Third, we determine the latest slot and fetch the relevant events for this slot.

        // Finally, we process the events and update the IRMA mint prices accordingly.
        
        CpiContext::new(
            openbook_v2::ID,
            ConsumeEvents {
                event_queue: Pubkey::default(), // Replace with actual event queue pubkey
                market: Pubkey::default(), // Replace with actual market pubkey
                openbook_program: Pubkey::default(), // Replace with actual OpenBook program pubkey
            },
        );
        Ok(())
    }

    pub fn consume_openbook_events<'info>(
        ctx: CpiContext<'_, '_, 'c, 'info, ConsumeEvents>,
        slots: Vec<usize>,
    ) -> Result<()> {
        // process each event in the OpenBook V2 event queue
        consume_given_events(ctx, slots)?;
        Ok(())
    }


    // Event types for OpenBook V2 market executions
    pub enum OpenBookEvent {
        BuyIRMA {
            trader: Pubkey,
            quote_token: Stablecoins,
            amount: u64, // amount of quote_token paid
        },
        SellIRMA {
            trader: Pubkey,
            quote_token: Stablecoins,
            irma_amount: u64, // amount of IRMA sold
        },
    }

    // Listen to OpenBook V2 events and call the appropriate IRMA program functions
    pub fn handle_openbook_event<'info>(
        ctx: Context<'_, '_, '_, 'info, crate::irma::MintIrma<'info>>, // or RedeemIrma as needed
        event: OpenBookEvent,
    ) -> Result<()> {
        match event {
            OpenBookEvent::BuyIRMA { trader: _, quote_token, amount } => {
                // Call mint_irma for buy executions
                mint_irma(ctx, quote_token, amount)?;
            }
            OpenBookEvent::SellIRMA { trader: _, quote_token, irma_amount } => {
                // Call redeem_irma for sell executions
                redeem_irma(ctx, quote_token, irma_amount)?;
            }
        }
        Ok(())
    }

    // Oracle inflation input function
    pub fn oracle_inflation_input<'info>(
        ctx: Context<'_, '_, '_, 'info, crate::irma::SetMintPrice<'info>>,
        inflation_percent: f64,
        stablecoin: Stablecoins,
        stablecoin_price_usd: f64,
    ) -> Result<()> {
        let mint_price = if inflation_percent < 2.0 {
            1.0
        } else {
            stablecoin_price_usd * (1.0 + inflation_percent / 100.0)
        };
        set_mint_price(ctx, stablecoin, mint_price)?;
        Ok(())
    }
}
