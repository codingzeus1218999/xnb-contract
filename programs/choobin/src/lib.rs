use anchor_lang::prelude::*;
use solana_program::{
    pubkey::Pubkey,
    system_instruction,
    program::{invoke},
};
// use anchor_spl::token;
// use anchor_spl::{
//     token::{ Transfer, Burn }
// };
use anchor_lang::context::Context;
use pyth_sdk_solana::{load_price_feed_from_account_info};
use crate::{error::ErrorCode};

pub mod contexts;
pub use contexts::*;

pub mod error;

declare_id!("FFcjp1c7oRK6adHWJpjbzdA2X4e7bFxGTGkKReEiGXwT");

#[program]
pub mod choobin {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, price: u64, private_price: u64, end_timestamp: u64) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        let initializer: &Signer = &ctx.accounts.initializer;
        // let mint = &ctx.accounts.mint;
        let treasury = &ctx.accounts.treasury;

        require!(!presale_info.is_initialized, ErrorCode::ErrorInitializedAready);

        presale_info.is_initialized = true;
        presale_info.admin = initializer.to_account_info().key();
        // presale_info.mint = mint.to_account_info().key();
        presale_info.treasury = treasury.to_account_info().key();
        presale_info.usd_amount = 0;
        presale_info.price = price;
        presale_info.private_price = private_price;
        presale_info.end_timestamp = end_timestamp;
        presale_info.private_sale = true;

        Ok(())
    }

    // pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
    //     let presale_info = &mut ctx.accounts.presale_info;

    //     let cpi_accounts = Transfer {
    //         from: ctx.accounts.payer_mint_ata.to_account_info(),
    //         to: ctx.accounts.presale_info_mint_ata.to_account_info(),
    //         authority: ctx.accounts.payer.to_account_info()
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::transfer(cpi_ctx, amount)?;

    //     presale_info.amount += amount;
    //     Ok(())
    // }

    // pub fn burn_token(ctx: Context<BurnToken>) -> Result<()> {
    //     let presale_info = &mut ctx.accounts.presale_info;
    //     let now_ts = Clock::get().unwrap().unix_timestamp as u64;

    //     require!(now_ts > presale_info.end_timestamp, ErrorCode::ErrorInvalidTimestamp);

    //     //--- send token from presale to user pda ---------
    //     // signer -> presale_info
    //     let (_presale_info_pda, presale_info_bump) = Pubkey::find_program_address(
    //         &[
    //             PRESALE_INFO_SEED.as_bytes(),
    //         ],
    //         ctx.program_id
    //     );
    //     let seeds = &[
    //         PRESALE_INFO_SEED.as_bytes(),
    //         &[presale_info_bump]
    //     ];
    //     let signer = &[&seeds[..]];

    //     let cpi_accounts = Burn {
    //         mint: ctx.accounts.mint.to_account_info(),
    //         from: ctx.accounts.presale_info_mint_ata.to_account_info(),
    //         authority: presale_info.to_account_info(),
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    //     token::burn(cpi_ctx, presale_info.amount)?;    

    //     //---- update data -----------
    //     presale_info.amount = 0;

    //     Ok(())
    // }

    pub fn change_admin(ctx: Context<ChangeAdmin>) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        let new_admin = &ctx.accounts.new_admin;
        presale_info.admin = new_admin.to_account_info().key();

        Ok(())
    }

    pub fn change_treasury(ctx: Context<ChangeTreasury>) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        let treasury = &ctx.accounts.treasury;
        presale_info.treasury = treasury.to_account_info().key();
        
        Ok(())
    }

    pub fn set_endtime(ctx: Context<SetEndtime>, endtimestamp: u64) -> Result<()> {
        let now_ts = Clock::get().unwrap().unix_timestamp as u64;

        require!(now_ts < endtimestamp, ErrorCode::ErrorInvalidTimestamp);

        let presale_info = &mut ctx.accounts.presale_info;
        presale_info.end_timestamp = endtimestamp;

        Ok(())
    }

    pub fn set_price(ctx: Context<SetPrice>, price: u64, private_price: u64) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        presale_info.price = price;
        presale_info.private_price = private_price;

        Ok(())
    }

    pub fn change_private_sale(ctx: Context<SetPrice>, private_sale: bool) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        presale_info.private_sale = private_sale;

        Ok(())
    }

    pub fn create_user_info(ctx: Context<CreateUserInfo>) -> Result<()> {
        let user_info = &mut ctx.accounts.user_info;
        let user: &Signer = &ctx.accounts.user;

        if !user_info.is_initialized {
            user_info.is_initialized = true;
            user_info.admin = user.to_account_info().key();
            user_info.amount = 0;
        }

        Ok(())
    }

    pub fn buy_token(ctx: Context<BuyToken>, lamports: u64) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        let user_info = &mut ctx.accounts.user_info;
        let user: &Signer = &ctx.accounts.user;
        let treasury = &ctx.accounts.treasury;
        let price_account_info = &ctx.accounts.price_feed;

        require!(!presale_info.private_sale || lamports >= 5_000_000_000, ErrorCode::InvalidMinimumSol);

        //--- send sol -> treasury ---------
        let sol_ix = system_instruction::transfer(
            user.key,
            treasury.key,
            lamports,
        );
        invoke(
            &sol_ix,
            &[
                user.to_account_info(),
                treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info()
            ],
        )?;

        // //--- send token from presale to user pda ---------
        // // signer -> presale_info
        // let (_presale_info_pda, presale_info_bump) = Pubkey::find_program_address(
        //     &[
        //         PRESALE_INFO_SEED.as_bytes(),
        //     ],
        //     ctx.program_id
        // );
        // let seeds = &[
        //     PRESALE_INFO_SEED.as_bytes(),
        //     &[presale_info_bump]
        // ];
        // let signer = &[&seeds[..]];

        // let cpi_accounts = Transfer {
        //     from: ctx.accounts.presale_info_mint_ata.to_account_info(),
        //     to: ctx.accounts.user_info_mint_ata.to_account_info(),
        //     authority: presale_info.to_account_info(),
        // };
        // let cpi_program = ctx.accounts.token_program.to_account_info();
        // let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        let price_feed = load_price_feed_from_account_info( &price_account_info ).unwrap();
        let current_timestamp = Clock::get()?.unix_timestamp;
        let current_price = price_feed.get_price_no_older_than(current_timestamp, STALENESS_THRESHOLD).unwrap();
        let price;
        if presale_info.private_sale {
            price = presale_info.private_price;
        } else {
            price =presale_info.price;
        }
        let amount = (
            u64::try_from(current_price.price).unwrap() - u64::try_from(current_price.conf).unwrap()
        ) / 10u64.pow(u32::try_from(-current_price.expo).unwrap()) * lamports / price;

        // token::transfer(cpi_ctx, presale_info.amount)?;

        //---- update data -----------
        presale_info.usd_amount += amount * price;
        user_info.amount += amount;

        Ok(())
    }

    // pub fn claim(ctx: Context<Claim>) -> Result<()> {
    //     let presale_info = &mut ctx.accounts.presale_info;
    //     let user_info = &mut ctx.accounts.user_info;
    //     let user: &Signer = &ctx.accounts.user;

    //     let now_ts = Clock::get().unwrap().unix_timestamp as u64;

    //     require!(now_ts > presale_info.end_timestamp, ErrorCode::ErrorInvalidTimestamp);

    //     //--- send token from presale to user pda ---------
    //     // signer -> presale_info
    //     let (_user_info_pda, user_info_bump) = Pubkey::find_program_address(
    //         &[
    //             USER_INFO_SEED.as_bytes(),
    //             &user.to_account_info().key().to_bytes(),
    //         ],
    //         ctx.program_id
    //     );
    //     let seeds = &[
    //         USER_INFO_SEED.as_bytes(),
    //         &user.to_account_info().key().to_bytes(),
    //         &[user_info_bump]
    //     ];
    //     let signer = &[&seeds[..]];

    //     let cpi_accounts = Transfer {
    //         from: ctx.accounts.user_info_mint_ata.to_account_info(),
    //         to: ctx.accounts.user_mint_ata.to_account_info(),
    //         authority: user_info.to_account_info()
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    //     token::transfer(cpi_ctx, user_info.amount)?;    

    //     //---- update data -----------
    //     user_info.amount = 0;

    //     Ok(())
    // }
}
