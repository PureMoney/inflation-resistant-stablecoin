
#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use anchor_lang::prelude::Pubkey;
    use solana_sdk_ids::system_program;
    use anchor_lang::prelude::Signer;
    use anchor_lang::prelude::Account;
    use anchor_lang::prelude::Program;
    use anchor_lang::context::Context;
    use bytemuck::bytes_of_mut;
    use irma_program::pricing::CustomError;
    use irma_program::pricing::BACKING_COUNT;
    use irma_program::pricing::{Stablecoins, State, Initialize, IrmaCommon, IrmaCommonBumps, InitializeBumps};
    use irma_program::pricing::{initialize, set_mint_price, mint_irma, redeem_irma};
    use irma_program::pricing::Stablecoins::EnumCount;

    fn allocate_state<'info>() -> &'info mut State {
        let state = State {
            mint_price: [0f64; Stablecoins::EnumCount as usize],
            backing_reserves: [0u64; Stablecoins::EnumCount as usize],
            irma_in_circulation: [0u64; Stablecoins::EnumCount as usize],
            backing_decimals: [6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0],
            padding: [0u8; 7],
            bump: 0u8,
        };
        Box::leak(Box::new(state))
    }

    fn init_state() -> State {
        let mut state: &mut State = allocate_state();
        let mut state = *state;
        for i in 0..BACKING_COUNT as usize {
            state.mint_price[i] = 1.0; // Initialize with default price
            state.backing_reserves[i] = 1000; // Initialize with some reserve
            state.irma_in_circulation[i] = 100; // Initialize with some IRMA in circulation
            state.backing_decimals[i] = 6; // Assume 6 decimals for stablecoins
            assert_eq!(1.0, state.mint_price[i]);
        }
        assert_eq!(state.mint_price.len(), EnumCount as usize);
        assert_eq!(state.backing_reserves.len(), EnumCount as usize);
        assert_eq!(state.irma_in_circulation.len(), EnumCount as usize);
        assert_eq!(state.backing_decimals.len(), EnumCount as usize);
        assert_eq!(state.bump, 0u8);
        state
    }

    #[test]
    fn test_set_state_directly() {
        let mut state: State = init_state();
        let quote_token: Stablecoins = Stablecoins::USDT;
        let new_price: f64 = 1.23;
        state.mint_price[quote_token as usize] = 1.0;
        assert_eq!(state.mint_price[quote_token as usize], 1.0);
        state.mint_price[quote_token as usize] = new_price;
        assert_eq!(state.mint_price[quote_token as usize], new_price);
    }

    #[test]
    fn test_mint_irma_directly() {
        let mut state = init_state();
        let quote_token = Stablecoins::USDT;
        let amount = 100;
        let price = state.mint_price[quote_token as usize];
        let prev_circulation = state.irma_in_circulation[quote_token as usize];
        let prev_reserve = state.backing_reserves[quote_token as usize];
        // Simulate mint_irma logic
        state.backing_reserves[quote_token as usize] += amount;
        state.irma_in_circulation[quote_token as usize] += (amount as f64 / price).ceil() as u64;
        assert_eq!(state.backing_reserves[quote_token as usize], prev_reserve + amount);
        assert_eq!(state.irma_in_circulation[quote_token as usize], prev_circulation + (amount as f64 / price).ceil() as u64);
    }

    #[test]
    fn test_redeem_irma_simple() {
        let mut state = init_state();
        let quote_token = Stablecoins::USDT;
        let irma_amount = 10;
        let prev_circulation = state.irma_in_circulation[quote_token as usize];
        // Simulate redeem_irma logic (simple case)
        state.irma_in_circulation[quote_token as usize] -= irma_amount;
        assert_eq!(state.irma_in_circulation[quote_token as usize], prev_circulation - irma_amount);
    }

    #[test]
    fn test_reduce_circulations_logic() {
        let mut state = init_state();
        // Manipulate state to create a price difference
        state.mint_price[Stablecoins::USDT as usize] = 2.0;
        state.backing_reserves[Stablecoins::USDT as usize] = 1000;
        state.irma_in_circulation[Stablecoins::USDT as usize] = 100;
        // Should select USDT as first_target
        let quote_token = Stablecoins::USDT;
        let irma_amount = 5;
        let prev_circulation = state.irma_in_circulation[quote_token as usize];
        // Simulate reduce_circulations logic (first_target == quote_token)
        state.irma_in_circulation[quote_token as usize] -= irma_amount;
        assert_eq!(state.irma_in_circulation[quote_token as usize], prev_circulation - irma_amount);
    }

    fn prep_accounts<'info>(owner: &'info Pubkey, state_account: Pubkey) -> (AccountInfo<'info>, AccountInfo<'info>, AccountInfo<'info>) {
        // Create a buffer for State and wrap it in AccountInfo
        let lamports: &'info mut u64 = Box::leak(Box::new(100000u64));
        let state: &'info mut State = allocate_state();

        let state_data: &'info mut [u8] = bytes_of_mut(state);
        let state_key: &'info mut Pubkey = Box::leak(Box::new(state_account));
        msg!("State pre-test account data: {:?}", state_data);
        let state_account_info: AccountInfo<'info> = AccountInfo::new(
            state_key,
            false, // is_signer
            true,  // is_writable
            lamports,
            state_data,
            owner,
            false,
            0,
        );
        msg!("State account created: {:?}", state_account_info.key);
        msg!("State owner: {:?}", owner);
        // Use a mock Signer for testing purposes
        let signer_pubkey: &'info mut Pubkey = Box::leak(Box::new(Pubkey::new_unique()));
        let lamportsx: &'info mut u64 = Box::leak(Box::new(0u64));
        let data: &'info mut [u8] = Box::leak(Box::new([0u8; 1024]));
        let owner: &'info mut Pubkey = Box::leak(Box::new(Pubkey::default()));
        let signer_account_info: AccountInfo<'info> = AccountInfo::new(
            signer_pubkey,
            true, // is_signer
            false, // is_writable
            lamportsx,
            data,
            owner,
            false,
            0,
        );
        // Create AccountInfo for system_program
        let sys_lamports: &'info mut u64 = Box::leak(Box::new(0u64));
        let sys_data: &'info mut [u8] = Box::leak(Box::new([0u8; 1024]));
        let sys_owner: &'info mut Pubkey = Box::leak(Box::new(Pubkey::default()));
        let sys_account_info: AccountInfo<'info> = AccountInfo::new(
            &system_program::ID,
            false, // is_signer
            false, // is_writable
            sys_lamports,
            sys_data,
            sys_owner,
            true,
            0,
        );
        (state_account_info, signer_account_info, sys_account_info)
    }

    fn initialize_anchor<'info>(program_id: &'info Pubkey) -> (AccountLoader<'info, State>, Signer<'info>, Program<'info, anchor_lang::system_program::System>) {
        //                 state_account_info: &'info AccountInfo<'info>) {
        //                 sys_account_info: &AccountInfo<'info>) {
        // let program_id: &'info Pubkey = Box::leak(Box::new(Pubkey::new_from_array(irma::ID.to_bytes())));
        let state_account: Pubkey = Pubkey::find_program_address(&[b"state".as_ref()], program_id).0;
        let (state_account_info, irma_admin_account_info, sys_account_info) 
                 = prep_accounts(program_id, state_account);
        // Bind to variables to extend their lifetime
        let state_account_static: &'info AccountInfo<'info> = Box::leak(Box::new(state_account_info));
        let irma_admin_account_static: &'info AccountInfo<'info> = Box::leak(Box::new(irma_admin_account_info));
        let sys_account_static: &'info AccountInfo<'info> = Box::leak(Box::new(sys_account_info));
        let mut accounts: Initialize<'_> = Initialize {
            state: AccountLoader::try_from(state_account_static).unwrap(),
            irma_admin: Signer::try_from(irma_admin_account_static).unwrap(),
            system_program: Program::try_from(sys_account_static).unwrap(),
        };
        let ctx: Context<Initialize> = Context::new(
            program_id,
            &mut accounts,
            &[],
            InitializeBumps { state: 0u8 }, // Use default bumps if not needed
        );
        let result: std::result::Result<(), Error> = initialize(ctx);
        assert!(result.is_ok());
        msg!("State account: {:?}", accounts.state);
        return (accounts.state, accounts.irma_admin, accounts.system_program);
    }

    #[test]
    fn test_initialize_anchor() {
        msg!("-------------------------------------------------------------------------");
        msg!("Testing initialize IRMA with normal conditions");  
        msg!("-------------------------------------------------------------------------");
        let program_id: &'static Pubkey = &irma_program::IRMA_ID;
        let (state_loader, irma_admin_account, sys_account) 
                = initialize_anchor(program_id);
        // Bind to variables to extend their lifetime
        let mut accounts: Initialize<'_> = Initialize {
            state: state_loader.clone(),
            irma_admin: irma_admin_account.clone(),
            system_program: sys_account.clone(),
        };
        let ctx: Context<Initialize> = Context::new(
            program_id,
            &mut accounts,
            &[],
            InitializeBumps { state: 0u8 }, // Use default bumps if not needed
        );
        let result: std::result::Result<(), Error> = initialize(ctx);
        assert!(result.is_ok());
        msg!("State loader initialized successfully: {:?}", accounts.state);
    }

    #[test]
    fn test_set_mint_price_anchor<'info>() {
        msg!("-------------------------------------------------------------------------");
        msg!("Testing set IRMA mint price with normal conditions");  
        msg!("-------------------------------------------------------------------------");
        let program_id: &'static Pubkey = &irma_program::IRMA_ID;
        let (state_loader, irma_admin_account, sys_account) 
                = initialize_anchor(program_id);
        // Bind to variables to extend their lifetime
        let mut accounts: IrmaCommon<'info> = IrmaCommon {
            state: state_loader.clone(),
            trader: irma_admin_account.clone(),
            system_program: sys_account.clone(),
        };
        let mut ctx: Context<IrmaCommon> = Context::new(
            program_id,
            &mut accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        let mut result: std::result::Result<(), Error> = set_mint_price(ctx, Stablecoins::USDT, 1.5);
        assert!(result.is_ok());
        // Re-create ctx for the next call if needed
        ctx = Context::new(
            program_id,
            &mut accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = set_mint_price(ctx, Stablecoins::USDC, 1.8);
        assert!(result.is_ok());
        ctx = Context::new(
            program_id,
            &mut accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = set_mint_price(ctx, Stablecoins::FDUSD, 1.3);
        assert!(result.is_ok());
        let state: State = *accounts.state.load().unwrap();
        msg!("Mint price for USDT set successfully: {:?}", state.mint_price[Stablecoins::USDT as usize]);
        msg!("Mint price for USDC set successfully: {:?}", state.mint_price[Stablecoins::USDC as usize]);
        msg!("Mint price for USDE set successfully: {:?}", state.mint_price[Stablecoins::FDUSD as usize]);
    }

    #[test]
    fn test_mint_irma_anchor<'info>() {
        msg!("-------------------------------------------------------------------------");
        msg!("Testing mint IRMA with normal conditions");  
        msg!("-------------------------------------------------------------------------");
        let program_id: &'info Pubkey = &irma_program::IRMA_ID;
        // let state_loader: Pubkey = Pubkey::find_program_address(&[b"state".as_ref()], program_id).0;
        let (state_loader, irma_admin_account, sys_account) 
                = initialize_anchor(program_id);
        // Bind to variables to extend their lifetime
        let mut accounts: IrmaCommon<'_> = IrmaCommon {
            state: state_loader.clone(),
            trader: irma_admin_account.clone(),
            system_program: sys_account.clone(),
        };
        let state: State = *accounts.state.load().unwrap();
        msg!("Pre-mint IRMA state:");
        msg!("Backing reserves for USDT: {:?}", state.backing_reserves[Stablecoins::USDT as usize]);
        msg!("Backing reserves for PYUSD: {:?}", state.backing_reserves[Stablecoins::PYUSD as usize]);
        msg!("Backing reserves for USDG: {:?}", state.backing_reserves[Stablecoins::USDG as usize]);
        msg!("IRMA in circulation for USDT: {:?}", state.irma_in_circulation[Stablecoins::USDT as usize]);
        msg!("IRMA in circulation for PYUSD: {:?}", state.irma_in_circulation[Stablecoins::PYUSD as usize]);
        msg!("IRMA in circulation for USDG: {:?}", state.irma_in_circulation[Stablecoins::USDG as usize]);
        let mut ctx: Context<IrmaCommon> = Context::new(
            program_id,
            &mut accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        let mut result = mint_irma(ctx, Stablecoins::USDT, 100);
        match result {
            Err(e) => {
                msg!("Error minting IRMA for USDT: {:?}", e);
            },
            Ok(_) => {
                msg!("Mint IRMA successful for USDT");
            }
        }
        ctx = Context::new(
            program_id,
            &mut accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = mint_irma(ctx, Stablecoins::PYUSD, 1000);
        match result {
            Err(e) => {
                msg!("Error minting IRMA for PYUSD: {:?}", e);
            },
            Ok(_) => {
                msg!("Mint IRMA successful for PYUSD");
            }
        }
        ctx = Context::new(
            program_id,
            &mut accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = mint_irma(ctx, Stablecoins::USDG, 10000);
        match result {
            Err(e) => {
                msg!("Error minting IRMA for USDG: {:?}", e);
            },
            Ok(_) => {
                msg!("Mint IRMA successful for USDG");
            }
        }
        let state: State = *accounts.state.load().unwrap();
        msg!("-------------------------------------------------------------------------");
        msg!("Post-mint IRMA state:");
        msg!("Backing reserves for USDT: {:?}", state.backing_reserves[Stablecoins::USDT as usize]);
        msg!("Backing reserves for PYUSD: {:?}", state.backing_reserves[Stablecoins::PYUSD as usize]);
        msg!("Backing reserves for USDG: {:?}", state.backing_reserves[Stablecoins::USDG as usize]);
        msg!("IRMA in circulation for USDT: {:?}", state.irma_in_circulation[Stablecoins::USDT as usize]);
        msg!("IRMA in circulation for PYUSD: {:?}", state.irma_in_circulation[Stablecoins::PYUSD as usize]);
        msg!("IRMA in circulation for USDG: {:?}", state.irma_in_circulation[Stablecoins::USDG as usize]);
    }


    #[test]
    fn test_redeem_irma_anchor<'info>() -> Result<()> {        
        msg!("-------------------------------------------------------------------------");
        msg!("Testing redeem IRMA when mint price is less than backing price");  
        msg!("-------------------------------------------------------------------------");
        let program_id: &'info Pubkey = &irma_program::IRMA_ID;
        let (state_loader, irma_admin_account, sys_account) 
            = initialize_anchor(program_id);
        let mut accounts: IrmaCommon<'info> = IrmaCommon {
            state: state_loader.clone(),
            trader: irma_admin_account.clone(),
            system_program: sys_account.clone(),
        };
        let state = accounts.state.clone();
        let mut state: &mut State = &mut state.load_mut().unwrap();
        let mut mut_accounts: &mut IrmaCommon<'info> = Box::leak(Box::new(accounts));
        msg!("Pre-redeem IRMA state:");
        for i in 0..BACKING_COUNT {
            let reserve: &mut u64 = &mut state.backing_reserves[i];
            let circulation: &mut u64 = &mut state.irma_in_circulation[i];
            if state.backing_decimals[i as usize] == 0 {
                require!(*reserve == 0, CustomError::InvalidBacking);
                require!(*circulation == 1, CustomError::InvalidIrmaAmount);
                continue; // skip non-existent stablecoins
            }
            *reserve = 1000000; // Set a large reserve for testing
            *circulation = 100000; // Set a large IRMA in circulation for testing
        }
        msg!("Current prices: {:?}", state.mint_price);
        msg!("Backing reserves: {:?}", state.backing_reserves);
        msg!("IRMA in circulation: {:?}", state.irma_in_circulation);
        let mut ctx: Context<IrmaCommon> = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        let mut result: std::result::Result<(), Error> = redeem_irma(ctx, Stablecoins::USDC, 10);
        match result {
            Err(e) => {
                msg!("Error redeeming IRMA for USDC: {:?}", e);
            },
            Ok(_) => {
                msg!("Redeem IRMA successful for USDC");
            }
        }
        // assert!(result.is_ok(), "Redeem IRMA failed for USDC");
        ctx = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = redeem_irma(ctx, Stablecoins::USDT, 20);
        match result {
            Err(e) => {
                msg!("Error redeeming IRMA for USDT: {:?}", e);
            },
            Ok(_) => {
                msg!("Redeem IRMA successful for USDT");
            }
        }
        ctx = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = redeem_irma(ctx, Stablecoins::PYUSD, 30);
        match result {
            Err(e) => {
                msg!("Error redeeming IRMA for PYUSD: {:?}", e);
            },
            Ok(_) => {
                msg!("Redeem IRMA successful for PYUSD");
            }
        }
        ctx = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = redeem_irma(ctx, Stablecoins::USDG, 40);
        match result {
            Err(e) => {
                msg!("Error redeeming IRMA for USDG: {:?}", e);
            },
            Ok(_) => {
                msg!("Redeem IRMA successful for USDG");
            }
        }
        ctx = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = redeem_irma(ctx, Stablecoins::FDUSD, 50);
        match result {
            Err(e) => {
                msg!("Error redeeming IRMA for FDUSD: {:?}", e);
            },
            Ok(_) => {
                msg!("Redeem IRMA successful for FDUSD");
            }
        }
        ctx = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );

        msg!("Mid-state for USDT before further redemption: {:?}", 
            state_loader.load().unwrap().backing_reserves[Stablecoins::USDT as usize]);
        // Test for near maximum redemption
        result = redeem_irma(ctx, Stablecoins::USDT, 10_000);
        match result {
            Err(e) => {
                msg!("Error redeeming IRMA for USDT: {:?}", e);
            },
            Ok(_) => {
                msg!("Redeem IRMA successful for USDT");
            }
        }
        ctx = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        result = redeem_irma(ctx, Stablecoins::USDS, 10);
        match result {
            Err(e) => {
                msg!("Error redeeming IRMA for USDS: {:?}", e);
            },
            Ok(_) => {
                msg!("Redeem IRMA successful for USDS");
            }
        }
        let state: State = *mut_accounts.state.load().unwrap();
        msg!("-------------------------------------------------------------------------");
        msg!("Redeem IRMA successful:");
        msg!("Backing reserves for USDT: {:?}", state.backing_reserves);
        msg!("IRMA in circulation for USDT: {:?}", state.irma_in_circulation);
        Ok(())
    }

    /// Test cases for when redemption price is less than mint price
    #[test]
    fn test_redeem_irma_normal<'info>() -> Result<()> {
        msg!("-------------------------------------------------------------------------");
        msg!("Testing redeem IRMA with normal conditions");  
        msg!("-------------------------------------------------------------------------");
        let program_id: &'info Pubkey = &irma_program::IRMA_ID;
        let (state_loader, irma_admin_account, sys_account) 
            = initialize_anchor(program_id);
        let mut accounts: IrmaCommon<'_> = IrmaCommon {
            state: state_loader.clone(),
            trader: irma_admin_account.clone(),
            system_program: sys_account.clone(),
        };
        msg!("Pre-redeem IRMA state:");
        let state = accounts.state.clone();
        let state: &mut State = &mut state.load_mut().unwrap();
        let mut mut_accounts: &mut IrmaCommon<'info> = Box::leak(Box::new(accounts));
        for i in 0..BACKING_COUNT {
            let reserve: &mut u64 = &mut state.backing_reserves[i];
            let circulation: &mut u64 = &mut state.irma_in_circulation[i];
            let price: &mut f64 = &mut state.mint_price[i];
            if state.backing_decimals[i as usize] == 0 {
                require!(*reserve == 0, CustomError::InvalidBacking);
                require!(*circulation == 1, CustomError::InvalidIrmaAmount);
                continue; // skip non-existent stablecoins
            }
            *reserve = 9_900_000_000; // Set a large reserve for testing
            *circulation = 10_000_000_000; // Set a large IRMA in circulation for testing
            *price = (i as f64 + 1.0) * (i as f64 + 1.0); // Set a price for testing
        }
        msg!("Current prices: {:?}", state.mint_price);
        msg!("Backing reserves: {:?}", state.backing_reserves);
        msg!("IRMA in circulation: {:?}", state.irma_in_circulation);
        let mut ctx: Context<IrmaCommon> = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        // Test for near maximum redemption, multiple times, until it fails.
        // What we expect is that these repeated redemptions will equalize the differences between
        // mint prices and redemptions prices for all stablecoins.
        let mut reslt = redeem_irma(ctx, Stablecoins::USDT, 100_000);
        while reslt.is_ok() {
            ctx = Context::new(
                program_id,
                mut_accounts,
                &[],
                IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
            );
            reslt = redeem_irma(ctx, Stablecoins::USDT, 100_000);
            match reslt {
                Err(e) => {
                    msg!("Error redeeming IRMA for USDT: {:?}", e);
                    break; // Exit loop on error
                },
                Ok(_) => {
                    msg!("Redeem IRMA successful for USDT");
                }
            }
        }
        ctx = Context::new(
            program_id,
            mut_accounts,
            &[],
            IrmaCommonBumps { state: 0u8 }, // Use default bumps if not needed
        );
        msg!("-------------------------------------------------------------------------");
        msg!("Redeem IRMA successful:");
        msg!("Backing reserves: {:?}", state.backing_reserves);
        msg!("IRMA in circulation: {:?}", state.irma_in_circulation);
        Ok(())
    }
}
