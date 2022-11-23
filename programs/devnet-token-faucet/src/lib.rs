use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, sysvar},
    InstructionData,
};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("DLr1ELqXdqAqf1TCuXedFx8YaVq4KQDudnAvprJcJjRt");

pub fn get_mint_address(ticker: &str) -> Pubkey {
    Pubkey::find_program_address(
        &["mint".as_bytes(), ticker.to_lowercase().as_ref()],
        &crate::ID,
    )
    .0
}

pub fn get_mint_authority_address(ticker: &str) -> Pubkey {
    Pubkey::find_program_address(
        &["mint-authority".as_bytes(), ticker.to_lowercase().as_ref()],
        &crate::ID,
    )
    .0
}

#[program]
pub mod devnet_token_faucet {
    use anchor_spl::token::MintTo;

    use super::*;

    pub fn create_mint(ctx: Context<CreateMint>, ticker: String, decimals: u8) -> Result<()> {
        let mint_authority = &mut ctx.accounts.mint_authority;

        // Assert mint_authority is not already initialized
        assert!(!mint_authority.is_initialized);

        //Store the mint information in the PDA
        mint_authority.mint = ctx.accounts.mint.key();
        mint_authority.is_initialized = true;
        mint_authority.bump = *ctx.bumps.get("mint_authority").unwrap();
        mint_authority.decimals = decimals;
        mint_authority.ticker_len = ticker.len() as u8;
        mint_authority.ticker[..ticker.len()].copy_from_slice(&ticker.to_lowercase().as_bytes());

        Ok(())
    }

    pub fn airdrop_spl(ctx: Context<AirdropSpl>, amount: u64) -> Result<()> {
        let AirdropSpl {
            mint,
            mint_authority,
            destination,
            token_program,
        } = ctx.accounts;

        let ticker = mint_authority.ticker[..mint_authority.ticker_len as usize].to_vec();

        // Assert that the token account matches the mint account
        assert_eq!(destination.mint, mint.key());

        token::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    authority: mint_authority.to_account_info(),
                    to: destination.to_account_info(),
                    mint: mint.to_account_info(),
                },
                &[&[
                    "mint-authority".as_bytes(),
                    ticker.as_ref(),
                    &[mint_authority.bump],
                ]],
            ),
            amount,
        )?;

        msg!("Tokens minted!");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(ticker: String, decimals: u8)]
pub struct CreateMint<'info> {
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
        space = 8 + 32 + 1 + 1 + 1 + 1 + 16
    )]
    pub mint_authority: Account<'info, MintData>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AirdropSpl<'info> {
    #[account(
        mut,
        seeds = ["mint".as_bytes(), mint_authority.ticker[..mint_authority.ticker_len as usize].as_ref()],
        bump
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        seeds = ["mint-authority".as_bytes(), mint_authority.ticker[..mint_authority.ticker_len as usize].as_ref()],
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
    pub is_initialized: bool,
    pub decimals: u8,
    pub ticker_len: u8,
    pub ticker: [u8; 16],
}

pub fn create_mint_ix(
    program_id: Pubkey,
    payer: Pubkey,
    ticker: String,
    decimals: u8,
) -> Instruction {
    let (mint, _) = Pubkey::find_program_address(
        &["mint".as_bytes(), ticker.to_lowercase().as_ref()],
        &program_id,
    );

    let (mint_authority, _) = Pubkey::find_program_address(
        &["mint-authority".as_bytes(), ticker.to_lowercase().as_ref()],
        &program_id,
    );

    let accounts = accounts::CreateMint {
        mint,
        mint_authority,
        payer,
        token_program: token::ID,
        system_program: System::id(),
        rent: sysvar::rent::id(),
    };

    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::CreateMint {
            ticker: ticker,
            decimals: decimals,
        }
        .data(),
    }
}

pub fn airdrop_spl_with_ticker_ix(
    program_id: &Pubkey,
    ticker: String,
    payer: &Pubkey,
    amount: u64,
) -> Instruction {
    let mint = get_mint_address(&ticker);
    let mint_authority = get_mint_authority_address(&ticker);

    let destination = spl_associated_token_account::get_associated_token_address(&payer, &mint);

    let accounts = accounts::AirdropSpl {
        mint,
        mint_authority,
        destination,
        token_program: token::ID,
    };

    Instruction {
        program_id: *program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::AirdropSpl { amount }.data(),
    }
}

pub fn airdrop_spl_with_mint_pdas_ix(
    program_id: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    amount: u64,
) -> Instruction {
    let destination = spl_associated_token_account::get_associated_token_address(&payer, &mint);

    let accounts = accounts::AirdropSpl {
        mint: *mint,
        mint_authority: *mint_authority,
        destination,
        token_program: token::ID,
    };

    Instruction {
        program_id: *program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::AirdropSpl { amount }.data(),
    }
}

#[cfg(test)]
mod ix_tests {
    use super::*;

    #[test]
    fn test_creat_mint_ix() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let ticker = "SOL".to_string();
        let decimals: u8 = 9;

        let instruction = create_mint_ix(program_id, payer, ticker.clone(), decimals);
        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 6);
        assert_eq!(
            instruction.data,
            instruction::CreateMint {
                ticker: ticker,
                decimals: decimals
            }
            .data()
        );
    }

    #[test]
    fn test_airdrop_spl_ix() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let ticker = "SOL".to_string();
        let amount: u64 = 10;

        let instruction = airdrop_spl_with_ticker_ix(&program_id, ticker.clone(), &payer, amount);
        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 4);
        assert_eq!(instruction.data, instruction::AirdropSpl { amount }.data())
    }
}
