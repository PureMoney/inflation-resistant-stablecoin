use anchor_lang::prelude::*;
use crate::irma::{mint_irma, redeem_irma, set_mint_price, Stablecoins};

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
