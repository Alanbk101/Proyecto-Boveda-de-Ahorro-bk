use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("CsxAVQQUeQ6BiJh77DrNaNPxkCezM4QhgkVgPKPLn5Pq"); 

#[program]
pub mod boveda_ahorro {
    use super::*;

    
    pub fn crear_boveda(ctx: Context<CrearBoveda>) -> Result<()> {
        let boveda = &mut ctx.accounts.boveda;
        boveda.owner = ctx.accounts.owner.key();
        boveda.saldo = 0;
        boveda.total_depositado = 0;
        boveda.total_retirado = 0;
        boveda.bump = ctx.bumps.boveda;

        msg!("✅ Bóveda creada para: {}", boveda.owner);
        Ok(())
    }

    
    pub fn depositar(ctx: Context<Depositar>, cantidad: u64) -> Result<()> {
        require!(cantidad > 0, BovedaError::CantidadInvalida);

        
        let cpi_accounts = Transfer {
            from: ctx.accounts.token_usuario.to_account_info(),
            to: ctx.accounts.token_boveda.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, cantidad)?;

        
        let boveda = &mut ctx.accounts.boveda;
        boveda.saldo = boveda.saldo.checked_add(cantidad).unwrap();
        boveda.total_depositado = boveda.total_depositado.checked_add(cantidad).unwrap();

        msg!("💰 Depositado: {} lamports de USDC", cantidad);
        msg!("📊 Saldo actual: {}", boveda.saldo);
        Ok(())
    }

    
    pub fn retirar(ctx: Context<Retirar>, cantidad: u64) -> Result<()> {
        require!(cantidad > 0, BovedaError::CantidadInvalida);
        require!(
            ctx.accounts.boveda.saldo >= cantidad,
            BovedaError::SaldoInsuficiente
        );

        let owner_key = ctx.accounts.owner.key();
        let seeds = &[
            b"boveda",
            owner_key.as_ref(),
            &[ctx.accounts.boveda.bump],
        ];
        let signer = &[&seeds[..]];

        
        let cpi_accounts = Transfer {
            from: ctx.accounts.token_boveda.to_account_info(),
            to: ctx.accounts.token_usuario.to_account_info(),
            authority: ctx.accounts.boveda.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, cantidad)?;

        
        let boveda = &mut ctx.accounts.boveda;
        boveda.saldo = boveda.saldo.checked_sub(cantidad).unwrap();
        boveda.total_retirado = boveda.total_retirado.checked_add(cantidad).unwrap();

        msg!("🏧 Retirado: {} lamports de USDC", cantidad);
        msg!("📊 Saldo restante: {}", boveda.saldo);
        Ok(())
    }

    
    pub fn ver_saldo(ctx: Context<VerSaldo>) -> Result<()> {
        let boveda = &ctx.accounts.boveda;
        msg!("👤 Owner: {}", boveda.owner);
        msg!("💵 Saldo actual: {}", boveda.saldo);
        msg!("📈 Total depositado: {}", boveda.total_depositado);
        msg!("📉 Total retirado: {}", boveda.total_retirado);
        Ok(())
    }
}



#[derive(Accounts)]
pub struct CrearBoveda<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Boveda::INIT_SPACE + 8,
        seeds = [b"boveda", owner.key().as_ref()],
        bump
    )]
    pub boveda: Account<'info, Boveda>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Depositar<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"boveda", owner.key().as_ref()],
        bump = boveda.bump,
        has_one = owner
    )]
    pub boveda: Account<'info, Boveda>,

    
    #[account(mut)]
    pub token_usuario: Account<'info, TokenAccount>,

    
    #[account(
        mut,
        seeds = [b"token_boveda", owner.key().as_ref()],
        bump
    )]
    pub token_boveda: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Retirar<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"boveda", owner.key().as_ref()],
        bump = boveda.bump,
        has_one = owner
    )]
    pub boveda: Account<'info, Boveda>,

    
    #[account(mut)]
    pub token_usuario: Account<'info, TokenAccount>,

    
    #[account(
        mut,
        seeds = [b"token_boveda", owner.key().as_ref()],
        bump
    )]
    pub token_boveda: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct VerSaldo<'info> {
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"boveda", owner.key().as_ref()],
        bump = boveda.bump,
        has_one = owner
    )]
    pub boveda: Account<'info, Boveda>,
}



#[account]
#[derive(InitSpace)]
pub struct Boveda {
    pub owner: Pubkey,       
    pub saldo: u64,          
    pub total_depositado: u64,
    pub total_retirado: u64,
    pub bump: u8,            
}


#[error_code]
pub enum BovedaError {
    #[msg("La cantidad debe ser mayor a 0")]
    CantidadInvalida,
    #[msg("Saldo insuficiente en la bóveda")]
    SaldoInsuficiente,
}
