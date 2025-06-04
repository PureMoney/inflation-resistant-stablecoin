/// This module is the "genesis" module that sets up all the IRMA markets.
/// It only runs once, after the program is deployed.
/// In the very beginning, there are no reserves, so the redemption prices are all zero.
/// We should mint IRMA for each of the reserve stablecoins, which should immediately
/// set the redemption prices equal to mint prices.
use anchor_lang::prelude::*;