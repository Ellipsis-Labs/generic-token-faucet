use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod devnet_token_faucet {
    use anchor_spl::token::MintTo;

    use super::*;

    pub fn create_mint(ctx: Context<CreateMint>, _ticker: String, _decimals: u8) -> Result<()> {
        let mint_authority = &mut ctx.accounts.mint_authority;

        // Assert mint_authority is not already initialized
        assert!(!mint_authority.is_initialized);

        //Store the mint information in the PDA
        mint_authority.mint = ctx.accounts.mint.key();
        mint_authority.is_initialized = true;
        mint_authority.bump = *ctx.bumps.get("mint_authority").unwrap();

        Ok(())
    }

    pub fn airdrop_spl(ctx: Context<AirdropSpl>, ticker: String, amount: u64) -> Result<()> {
        let destination_token_account = &ctx.accounts.destination;
        let mint_account = &ctx.accounts.mint;

        // Assert that the token account matches the mint account
        assert_eq!(destination_token_account.mint, mint_account.key());

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo{
                    authority: ctx.accounts.mint_authority.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info()                    
                },
                &[&[
                    "mint-authority".as_bytes(),
                    ticker.to_lowercase().as_ref(),
                    &[*ctx.bumps.get("mint_authority").unwrap()]
                ]]),
            amount
        )?;

        msg!("Tokens minted!");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(ticker: String, decimals: u8)]
pub struct CreateMint<'info>{
    #[account(
        init,
        seeds = ["mint".as_bytes(), ticker.to_lowercase().as_ref()],
        bump,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        seeds = [b"mint-authority".as_ref(), ticker.to_lowercase().as_ref()],
        bump,
        space = 8 + 32 + 1 + 1
    )]
    pub mint_authority: Account<'info, MintData>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct AirdropSpl<'info>{
    #[account(
        mut,
        seeds = ["mint".as_bytes(), ticker.to_lowercase().as_ref()],
        bump   
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        seeds = [b"mint-authority".as_ref(), ticker.to_lowercase().as_ref()],
        bump,
    )]
    pub mint_authority: Account<'info, MintData>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}


#[account]
#[derive(Debug)]
pub struct MintData {
    pub mint: Pubkey,
    pub bump: u8,
    pub is_initialized: bool
}
