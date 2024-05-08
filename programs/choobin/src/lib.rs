use anchor_lang::prelude::*;
use solana_program::{
    instruction::Instruction,
    sysvar::instructions::{load_instruction_at_checked},
    pubkey::Pubkey,
    system_instruction,
    program::{invoke},
};
use anchor_spl::token;
use anchor_spl::{
    token::{ Transfer, Burn }
};
use anchor_lang::context::Context;
use crate::{error::ErrorCode};

pub mod contexts;
pub use contexts::*;

pub mod error;

declare_id!("Ex1dEkyy4hF6ranqa5WcGV2X4hHpdd3JUJ5arocXtBEz");

#[program]
pub mod choobin {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        let initializer: &Signer = &ctx.accounts.initializer;
        let mint = &ctx.accounts.mint;
        let treasury = &ctx.accounts.treasury;

        require!(!presale_info.is_initialized, ErrorCode::ErrorInitializedAready);

        presale_info.is_initialized = true;
        presale_info.admin = initializer.to_account_info().key();
        presale_info.mint = mint.to_account_info().key();
        presale_info.amount = 0;
        presale_info.price = 3500;    // 1 choobin = 0.0000035 SOL
        presale_info.end_timestamp = 0;
        presale_info.treasury = treasury.to_account_info().key();

        Ok(())
    }

    pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;

        let cpi_accounts = Transfer {
            from: ctx.accounts.payer_mint_ata.to_account_info(),
            to: ctx.accounts.presale_info_mint_ata.to_account_info(),
            authority: ctx.accounts.payer.to_account_info()
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        presale_info.amount += amount;
        Ok(())
    }

    pub fn burn_token(ctx: Context<BurnToken>) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        let now_ts = Clock::get().unwrap().unix_timestamp as u64;

        require!(now_ts > presale_info.end_timestamp, ErrorCode::ErrorInvalidTimestamp);

        //--- send token from presale to user pda ---------
        // signer -> presale_info
        let (_presale_info_pda, presale_info_bump) = Pubkey::find_program_address(
            &[
                PRESALE_INFO_SEED.as_bytes(),
            ],
            ctx.program_id
        );
        let seeds = &[
            PRESALE_INFO_SEED.as_bytes(),
            &[presale_info_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.presale_info_mint_ata.to_account_info(),
            authority: presale_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::burn(cpi_ctx, presale_info.amount)?;    

        //---- update data -----------
        presale_info.amount = 0;

        Ok(())
    }

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

    pub fn set_price(ctx: Context<SetPrice>, price: u64) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        presale_info.price = price;
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

        //--- send sol -> treasury ---------
        let sol_ix = system_instruction::transfer(
            user.key,
            treasury.key,
            lamports,
        );
        anchor_lang::solana_program::program::invoke(
            &sol_ix,
            &[
                user.to_account_info(),
                treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info()
            ],
        )?;

        //--- send token from presale to user pda ---------
        // signer -> presale_info
        let (_presale_info_pda, presale_info_bump) = Pubkey::find_program_address(
            &[
                PRESALE_INFO_SEED.as_bytes(),
            ],
            ctx.program_id
        );
        let seeds = &[
            PRESALE_INFO_SEED.as_bytes(),
            &[presale_info_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.presale_info_mint_ata.to_account_info(),
            to: ctx.accounts.user_info_mint_ata.to_account_info(),
            authority: presale_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        let amount = (lamports * 1000000 / presale_info.price) as u64;
        token::transfer(cpi_ctx, amount)?;    

        //---- update data -----------
        presale_info.amount -= amount;
        user_info.amount += amount;

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let presale_info = &mut ctx.accounts.presale_info;
        let user_info = &mut ctx.accounts.user_info;
        let user: &Signer = &ctx.accounts.user;

        let now_ts = Clock::get().unwrap().unix_timestamp as u64;

        require!(now_ts > presale_info.end_timestamp, ErrorCode::ErrorInvalidTimestamp);

        //--- send token from presale to user pda ---------
        // signer -> presale_info
        let (_user_info_pda, user_info_bump) = Pubkey::find_program_address(
            &[
                USER_INFO_SEED.as_bytes(),
                &user.to_account_info().key().to_bytes(),
            ],
            ctx.program_id
        );
        let seeds = &[
            USER_INFO_SEED.as_bytes(),
            &user.to_account_info().key().to_bytes(),
            &[user_info_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_info_mint_ata.to_account_info(),
            to: ctx.accounts.user_mint_ata.to_account_info(),
            authority: user_info.to_account_info()
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, user_info.amount)?;    

        //---- update data -----------
        user_info.amount = 0;

        Ok(())
    }
}
