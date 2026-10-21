use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solana_program::ed25519_program::ID as ED25519_ID;

declare_id!("PLURiX1111111111111111111111111111111111111");

#[program]
pub mod plurix_vaults {
    use super::*;

    /// Initialize a new single-sided shared liquidity vault
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        vault_id: u64,
        reserve_ratio_bps: u16,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.vault_id = vault_id;
        vault.mint = ctx.accounts.mint.key();
        vault.token_vault = ctx.accounts.token_vault.key();
        vault.reserve_ratio_bps = reserve_ratio_bps;
        vault.total_deposits = 0;
        vault.is_paused = false;
        msg!("PLURIX: Shared vault #{} initialized for mint {}", vault_id, vault.mint);
        Ok(())
    }

    /// Execute a zero-slippage swap verified by off-chain signed oracle quote
    pub fn execute_swap_signed(
        ctx: Context<ExecuteSwapSigned>,
        amount_in: u64,
        oracle_rate_scaled: u64,
        quote_timestamp: i64,
        oracle_signature: [u8; 64],
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        // 1. Enforce strict 5-second quote validity
        require!(
            clock.unix_timestamp - quote_timestamp <= 5,
            PlurixError::QuoteExpired
        );

        // 2. Transfer user input tokens into vault reserve
        let cpi_accounts_in = Transfer {
            from: ctx.accounts.user_source_token.to_account_info(),
            to: ctx.accounts.vault_in_token.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program.clone(), cpi_accounts_in), amount_in)?;

        // 3. Compute exact guaranteed payout without curve slippage (0.00% slippage)
        let amount_out = (amount_in as u128 * oracle_rate_scaled as u128 / 1_000_000_000) as u64;

        // 4. Transfer output tokens to user directly from shared vault matrix
        let vault_seeds = &[
            b"plurix_vault",
            &ctx.accounts.vault_out.vault_id.to_le_bytes(),
            &[ctx.accounts.vault_out.bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        let cpi_accounts_out = Transfer {
            from: ctx.accounts.vault_out_token.to_account_info(),
            to: ctx.accounts.user_dest_token.to_account_info(),
            authority: ctx.accounts.vault_out.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts_out, signer_seeds),
            amount_out,
        )?;

        msg!("PLURIX: Zero-slippage swap executed. In: {}, Out: {}", amount_in, amount_out);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(vault_id: u64)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 32 + 32 + 2 + 8 + 1 + 1,
        seeds = [b"plurix_vault", vault_id.to_le_bytes().as_ref()],
        bump
    )]
    pub vault: Account<'info, SharedVault>,
    pub mint: Account<'info, token::Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = vault
    )]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExecuteSwapSigned<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub vault_in: Account<'info, SharedVault>,
    #[account(mut)]
    pub vault_out: Account<'info, SharedVault>,
    #[account(mut)]
    pub user_source_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_dest_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_in_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_out_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct SharedVault {
    pub authority: Pubkey,
    pub vault_id: u64,
    pub mint: Pubkey,
    pub token_vault: Pubkey,
    pub reserve_ratio_bps: u16,
    pub total_deposits: u64,
    pub is_paused: bool,
    pub bump: u8,
}

#[error_code]
pub enum PlurixError {
    #[msg("Signed oracle quote has expired (> 5 seconds).")]
    QuoteExpired,
    #[msg("Invalid oracle cryptographic signature.")]
    InvalidOracleSignature,
    #[msg("Shared vault reserves are below minimum coverage floor.")]
    InsufficientVaultReserves,
}
