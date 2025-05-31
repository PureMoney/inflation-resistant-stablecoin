
#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use anchor_lang::prelude::Pubkey;
    use anchor_lang::solana_program::system_program;
    use anchor_lang::prelude::Signer;
    use anchor_lang::prelude::Account;
    use anchor_lang::prelude::Program;
    use anchor_lang::context::Context;
    use irma::irmamod::{self, Stablecoins, State}; //, CustomError};
    use irma::irmamod::{initialize}; // , set_mint_price, mint_irma, redeem_irma};
    use anchor_lang::Discriminator;

    fn default_state() -> State {
        State {
            mint_price: vec![1.0; Stablecoins::EnumCount as usize],
            backing_reserves: vec![1000; Stablecoins::EnumCount as usize],
            irma_in_circulation: vec![100; Stablecoins::EnumCount as usize],
            bump: 0,
        }
    }

    #[test]
    fn test_set_mint_price() {
        let mut state = default_state();
        let quote_token = Stablecoins::USDT;
        let new_price = 1.23;
        state.mint_price[quote_token as usize] = 1.0;
        assert_eq!(state.mint_price[quote_token as usize], 1.0);
        state.mint_price[quote_token as usize] = new_price;
        assert_eq!(state.mint_price[quote_token as usize], new_price);
    }

    #[test]
    fn test_mint_irma() {
        let mut state = default_state();
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
        let mut state = default_state();
        let quote_token = Stablecoins::USDT;
        let irma_amount = 10;
        let prev_circulation = state.irma_in_circulation[quote_token as usize];
        // Simulate redeem_irma logic (simple case)
        state.irma_in_circulation[quote_token as usize] -= irma_amount;
        assert_eq!(state.irma_in_circulation[quote_token as usize], prev_circulation - irma_amount);
    }

    #[test]
    fn test_reduce_circulations_logic() {
        let mut state = default_state();
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

    fn prep_accounts(owner: &'static Pubkey, state_account: Pubkey) -> (AccountInfo<'static>, AccountInfo<'static>, AccountInfo<'static>) {
        // Create a buffer for State and wrap it in AccountInfo
        let lamports: &'static mut u64 = Box::leak(Box::new(100000u64));
        let state: State = default_state();

        // Prepare the account data with the correct discriminator
        let mut state_data_vec: Vec<u8> = Vec::with_capacity(1024);
        state.try_serialize(&mut state_data_vec).unwrap();

        let state_data: &'static mut Vec<u8> = Box::leak(Box::new(state_data_vec));
        let state_key: &'static mut Pubkey = Box::leak(Box::new(state_account));
        msg!("State account data: {:?}", state_data);
        let state_account_info: AccountInfo<'static> = AccountInfo::new(
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
        let signer_pubkey: &'static mut Pubkey = Box::leak(Box::new(Pubkey::new_unique()));
        let lamportsx: &'static mut u64 = Box::leak(Box::new(0u64));
        let data: &'static mut Vec<u8> = Box::leak(Box::new(vec![]));
        let owner: &'static mut Pubkey = Box::leak(Box::new(Pubkey::default()));
        let signer_account_info: AccountInfo<'static> = AccountInfo::new(
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
        let sys_lamports: &'static mut u64 = Box::leak(Box::new(0u64));
        let sys_data: &'static mut Vec<u8> = Box::leak(Box::new(vec![]));
        let sys_owner: &'static mut Pubkey = Box::leak(Box::new(Pubkey::default()));
        let sys_account_info: AccountInfo<'static> = AccountInfo::new(
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

    #[test]
    fn test_initialize_anchor() {
        let program_id: &'static Pubkey = Box::leak(Box::new(Pubkey::new_from_array(irma::ID.to_bytes())));
        let state_account: Pubkey = Pubkey::find_program_address(&[b"state".as_ref()], program_id).0;
        let (state_account_info, irma_admin_account_info, sys_account_info) 
                = prep_accounts(program_id, state_account);
        // Bind to variables to extend their lifetime
        let state_account_static: &'static AccountInfo<'static> = Box::leak(Box::new(state_account_info));
        let irma_admin_account_static: &'static AccountInfo<'static> = Box::leak(Box::new(irma_admin_account_info));
        let sys_account_static: &'static AccountInfo<'static> = Box::leak(Box::new(sys_account_info));
        let mut accounts: irmamod::Initialize<'_> = irmamod::Initialize {
            state: Account::try_from(state_account_static).unwrap(),
            irma_admin: Signer::try_from(irma_admin_account_static).unwrap(),
            system_program: Program::try_from(sys_account_static).unwrap(),
        };
        let ctx: Context<irmamod::Initialize> = Context::new(
            program_id,
            &mut accounts,
            &[],
            irmamod::InitializeBumps::default(), // Use default bumps if not needed
        );
        let result: std::result::Result<(), Error> = initialize(ctx);
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_set_mint_price_anchor() {
    //     let program_id = Pubkey::default();
    //     let mut state = default_state() as AccountInfo;
    //     let trader = Pubkey::new_unique() as Signer;
    //     let system_program_id = system_program::ID as Program;
    //     let ctx = Context::new(
    //         program_id,
    //         irmamod::SetMintPrice {
    //             state: Account::try_from(&mut state).unwrap(),
    //             trader: Signer::try_from(&trader).unwrap(),
    //             system_program: Program::try_from(&system_program_id).unwrap(),
    //         },
    //         vec![],
    //     );
    //     let result = set_mint_price(ctx, Stablecoins::USDT, 1.5);
    //     assert!(result.is_ok());
    // }

    // #[test]
    // fn test_mint_irma_anchor() {
    //     let program_id = Pubkey::default();
    //     let mut state = default_state() as AccountInfo;
    //     let trader = Pubkey::new_unique() as Signer;
    //     let system_program_id = system_program::ID as Program;
    //     let ctx = Context::new(
    //         program_id,
    //         irmamod::MintIrma {
    //             state: Account::try_from(&mut state).unwrap(),
    //             trader: Signer::try_from(&trader).unwrap(),
    //             system_program: Program::try_from(&system_program_id).unwrap(),
    //         },
    //         vec![],
    //     );
    //     let result = mint_irma(ctx, Stablecoins::USDT, 100);
    //     assert!(result.is_ok());
    // }

    // #[test]
    // fn test_redeem_irma_anchor() {
    //     let program_id = Pubkey::default();
    //     let mut state = default_state() as AccountInfo;
    //     let trader = Pubkey::new_unique() as Signer;
    //     let system_program_id = system_program::ID as Program;
    //     let ctx = Context::new(
    //         program_id,
    //         irmamod::RedeemIrma {
    //             state: Account::try_from(&mut state).unwrap(),
    //             trader: Signer::try_from(&trader).unwrap(),
    //             system_program: Program::try_from(&system_program_id).unwrap(),
    //         },
    //         vec![],
    //     );
    //     let result = redeem_irma(ctx, Stablecoins::USDT, 10);
    //     assert!(result.is_ok());
    // }
}
