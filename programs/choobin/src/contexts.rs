use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{TokenAccount, Mint, Token}
};
use std::mem::size_of;

pub const DISCRIMINATOR_LENGTH: usize = 8;
pub const PRESALE_INFO_SEED: &str = "presale_info";
pub const USER_INFO_SEED: &str = "user_info";

#[account]
pub struct PresaleInfo {
    pub is_initialized: bool,
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub price: u64,
    pub end_timestamp: u64,
    pub treasury: Pubkey,
}

#[account]
pub struct UserInfo {
    pub is_initialized: bool,
    pub admin: Pubkey,
    pub amount: u64,
}

// --------------------------- Admin Instructions -----------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, seeds = [PRESALE_INFO_SEED.as_bytes()], bump, 
        payer = initializer, space = size_of::<PresaleInfo>() + DISCRIMINATOR_LENGTH)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(
        mint::decimals = 6,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account( constraint = treasury.key() != Pubkey::default())]
    pub treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(
        init_if_needed,
        associated_token::mint = mint,
        associated_token::authority = presale_info,
        payer = payer
    )]    
    pub presale_info_mint_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]    
    pub payer_mint_ata: Account<'info, TokenAccount>,

    #[account(mut, constraint = mint.to_account_info().key() == presale_info.mint)]
    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct BurnToken<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut,
        associated_token::mint = mint,
        associated_token::authority = presale_info,
    )]    
    pub presale_info_mint_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, constraint = mint.to_account_info().key() == presale_info.mint)]
    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ChangeAdmin<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut, constraint = admin.to_account_info().key() == presale_info.admin)]
    pub admin: Signer<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        constraint = new_admin.key() != Pubkey::default(),
        constraint = new_admin.key() != admin.to_account_info().key(),
    )]
    pub new_admin: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct ChangeTreasury<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut, constraint = admin.to_account_info().key() == presale_info.admin)]
    pub admin: Signer<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account( constraint = treasury.key() != Pubkey::default())]
    pub treasury: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct SetEndtime<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut, constraint = admin.to_account_info().key() == presale_info.admin)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut, constraint = admin.to_account_info().key() == presale_info.admin)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateUserInfo<'info> {
    #[account(init_if_needed, seeds = [USER_INFO_SEED.as_bytes(), user.key().as_ref()], bump, 
        payer = user, space = size_of::<UserInfo>() + DISCRIMINATOR_LENGTH)]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut,
        associated_token::mint = mint,
        associated_token::authority = presale_info
    )]    
    pub presale_info_mint_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds = [USER_INFO_SEED.as_bytes(), user.key().as_ref()], bump)]
    pub user_info: Account<'info, UserInfo>,

    #[account(
        init_if_needed,
        associated_token::mint = mint,
        associated_token::authority = user_info,
        payer = user
    )]    
    pub user_info_mint_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, constraint = mint.to_account_info().key() == presale_info.mint)]
    pub mint: Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, constraint = treasury.key() == presale_info.treasury)]
    pub treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut, seeds = [PRESALE_INFO_SEED.as_bytes()], bump)]
    pub presale_info: Account<'info, PresaleInfo>,

    #[account(mut, close = user, seeds = [USER_INFO_SEED.as_bytes(), user.key().as_ref()], bump)]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut,
        associated_token::mint = mint,
        associated_token::authority = user_info,
    )]    
    pub user_info_mint_ata: Account<'info, TokenAccount>,

    #[account(mut, constraint = user.to_account_info().key() == user_info.admin)]
    pub user: Signer<'info>,

    #[account(mut, constraint = mint.to_account_info().key() == presale_info.mint)]
    pub mint: Account<'info, Mint>,

    #[account(init_if_needed,
        associated_token::mint = mint,
        associated_token::authority = user,
        payer = user
    )]    
    pub user_mint_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
