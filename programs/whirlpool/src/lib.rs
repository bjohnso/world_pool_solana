#![allow(dead_code)]
#![allow(unused_variables)]

mod validation;

extern crate core;

use std::{convert::TryFrom};
use anchor_spl::{associated_token::AssociatedToken, token::{CloseAccount, Mint, Token, TokenAccount, Transfer}};
use anchor_lang::{prelude::*, context::CpiContext};
use error::*;
use validation::*;

use solana_program::{
    system_program,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
};

declare_id!("AJjyLsVoEfhz7ds1ZM9RU44Zkf6bNakFC86PxXM4B7kT");

#[program]
pub mod whirlpool {
    use super::*;

    pub fn create_pool(ctx: Context<CreatePool>, pool_bump: u8, pool_token_bump: u8, name: String, description: String) -> ProgramResult {
        let pool_account = &mut ctx.accounts.pool_account;
        let token_account = &ctx.accounts.token_account;
        let mint = &ctx.accounts.mint;
        let admin_account = &ctx.accounts.admin;

        pool_account.name = name;
        pool_account.description = description;
        pool_account.admin = admin_account.key.clone();
        pool_account.mint = mint.key().clone();
        pool_account.token_account = token_account.key().clone();
        pool_account.token_account_bump = pool_token_bump;

        let pda = <[u8; 32]>::try_from(pool_account.to_account_info().key.as_ref()).unwrap();
        let token_account = <[u8; 32]>::try_from(token_account.key().as_ref()).unwrap();
        let mint = <[u8; 32]>::try_from(mint.key().as_ref()).unwrap();
        let admin = <[u8; 32]>::try_from(admin_account.key.as_ref()).unwrap();

        msg!("pool created with PDA {}", hex::encode(pda));
        msg!("pool created with name {}", pool_account.name);
        msg!("pool created with description {}", pool_account.description);
        msg!("pool created by admin {}", hex::encode(admin));
        msg!("pool created with mint {}", hex::encode(mint));
        msg!("pool created with token account {}", hex::encode(token_account));
        msg!("pool created with bump {}", pool_bump);
        msg!("pool created with token account bump {}", pool_account.token_account_bump);

        Ok(())
    }

    pub fn update_pool(ctx: Context<UpdatePool>, bump: u8, name: String, description: String) -> ProgramResult {
        let admin_account = &ctx.accounts.admin;
        let pool_account = &mut ctx.accounts.pool_account;

        if name != pool_account.name {
            pool_account.name = name;
        }

        if description != pool_account.description {
            pool_account.description = description;
        }

        let admin = <[u8; 32]>::try_from(admin_account.to_account_info().key.as_ref()).unwrap();
        let pda = <[u8; 32]>::try_from(pool_account.key().as_ref()).unwrap();

        msg!("pool updated with PDA {}", hex::encode(pda));
        msg!("pool updated with name {}", pool_account.name);
        msg!("pool updated with description {}", pool_account.description);
        msg!("pool updated by admin {}", hex::encode(admin));
        msg!("pool updated with bump {}", bump);

        Ok(())
    }

    pub fn read_pool(ctx: Context<ReadPool>, bump: u8) -> ProgramResult {
        let admin_account = &ctx.accounts.admin;
        let pool_account = &mut ctx.accounts.pool_account;

        let admin = <[u8; 32]>::try_from(admin_account.to_account_info().key.as_ref()).unwrap();
        let pda = <[u8; 32]>::try_from(pool_account.key().as_ref()).unwrap();

        msg!("pool read with PDA {}", hex::encode(pda));
        msg!("pool read with name {}", pool_account.name);
        msg!("pool read with description {}", pool_account.description);
        msg!("pool read by admin {}", hex::encode(admin));
        msg!("pool read with bump {}", bump);

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, state_bump: u8, escrow_bump: u8, pool_bump: u8, token_amount: u64) -> ProgramResult {
        let state_account = &mut ctx.accounts.state_account;
        let pool_account = &mut ctx.accounts.pool_account;

        state_account.user = ctx.accounts.user.key().clone();
        state_account.escrow = ctx.accounts.escrow_account.key().clone();
        state_account.token_amount = token_amount;

        pool_account.state_account = state_account.key().clone();
        pool_account.state_account_bump = state_bump;

        let state_pda = <[u8; 32]>::try_from(state_account.key().as_ref()).unwrap();
        let state_user = <[u8; 32]>::try_from(state_account.user.as_ref()).unwrap();
        let state_escrow = <[u8; 32]>::try_from(state_account.escrow.as_ref()).unwrap();

        msg!("escrow state created with PDA {}", hex::encode(state_pda));
        msg!("escrow state created with user {}", hex::encode(state_user));
        msg!("escrow state created with escrow {}", hex::encode(state_escrow));
        msg!("escrow state created with token amount {}", state_account.token_amount);

        let user_key = ctx.accounts.user.key;
        let state_bump_bytes = state_bump.to_le_bytes();

        let vector = vec![
            b"state-account".as_ref(),
            user_key.as_ref(),
            state_bump_bytes.as_ref()
        ];

        let signer_seeds = vec![vector.as_slice()];

        let transfer_instruction = Transfer {
            from: ctx.accounts.donor_account.to_account_info(),
            to: ctx.accounts.escrow_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info()
        };

        let cpi_context= CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer_seeds.as_slice()
        );

        anchor_spl::token::transfer(cpi_context, state_account.token_amount)?;

        state_account.stage = EscrowStage::Deposited.to_u8();

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, pool_bump: u8) -> ProgramResult {
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum EscrowStage {
    Deposited = 0,
    Complete = 1,
    Reversed = 2
}

impl EscrowStage {
    fn to_u8(&self) -> u8 {
       *self as u8
    }

    fn to_escrow_stage(stage: u8) -> Option<EscrowStage> {
        match stage {
            0 => Some(EscrowStage::Deposited),
            1 => Some(EscrowStage::Complete),
            2 => Some(EscrowStage::Reversed),
            _ => None
        }
    }
}
