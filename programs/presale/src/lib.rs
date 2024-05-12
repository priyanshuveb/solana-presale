use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Token, TokenAccount, Transfer as SplTransfer}; // for spl tokem transfer
use solana_program::system_instruction;

declare_id!("4G1qdqCMYXMnSAyup9A7qV5tZyoSHgjCjpfMDa2AbuP5");

#[program]
pub mod presale {

    use super::*;

    pub fn initialize(
        ctx: Context<Init>,
        start_time: u128,
        end_time: u128,
        token_price: u128,
        tokens_to_sell: u128,
    ) -> Result<()> {
        let current_time = Clock::get().unwrap().unix_timestamp.try_into().unwrap();

        require!(
            start_time > current_time && end_time > start_time,
            PresaleErrors::InvalidTime
        );

        require!(
            token_price > 0 && tokens_to_sell > 0,
            PresaleErrors::ZeroParams
        );

        let presale_account = &mut ctx.accounts.presale_account;

        presale_account.start_time = start_time;
        presale_account.end_time = end_time;
        presale_account.tokens_to_sell = tokens_to_sell;
        presale_account.token_price = token_price;
        presale_account.owner = ctx.accounts.owner.key();

        msg!(
            "Presale account created, with owner as {:?}",
            ctx.accounts.owner.key()
        );

        Ok(())
    }

    pub fn transfer_ownership(ctx: Context<UpdatePresale>, new_owner: Pubkey) -> Result<()> {
        let presale_account = &mut ctx.accounts.presale_account;
        msg!(
            "Updating the ownership from {} to {}",
            presale_account.owner,
            ctx.accounts.owner.key()
        );
        presale_account.owner = new_owner;
        Ok(())
    }

    pub fn updateimeline(
        ctx: Context<UpdatePresale>,
        start_time: u128,
        end_time: u128,
    ) -> Result<()> {
        let presale_account = &mut ctx.accounts.presale_account;
        presale_account.start_time = start_time;
        presale_account.end_time = end_time;
        Ok(())
    }

    pub fn update_token_price(ctx: Context<UpdatePresale>, token_price: u128) -> Result<()> {
        let presale_account = &mut ctx.accounts.presale_account;
        presale_account.token_price = token_price;
        Ok(())
    }

    pub fn update_tokens_to_sell(ctx: Context<UpdatePresale>, tokens_to_sell: u128) -> Result<()> {
        let presale_account = &mut ctx.accounts.presale_account;
        require!(
            presale_account.tokens_sold < tokens_to_sell,
            PresaleErrors::InvalidTokenSaleNumbers
        );
        presale_account.tokens_to_sell = tokens_to_sell;
        Ok(())
    }

    pub fn buy(ctx: Context<TransferLamportsAndBuyTokens>,  tokens_to_buy: u128) -> Result<()> {
        let from_account = &mut ctx.accounts.from;
        // let from_account: &ctx.accounts.from;
        let to_account = &mut ctx.accounts.to;
        let presale_account = &mut ctx.accounts.presale_account;
        let token_price = presale_account.token_price;
        let tokens_sold =  presale_account.tokens_sold;
        let tokens_to_sell = presale_account.tokens_to_sell;
        let amount_lamports = token_price * tokens_to_buy;
        // require!(token_price * tokens_to_buy == amount_sol.into(), PresaleErrors::InsufficientLamports);
        require!(tokens_sold + tokens_to_buy <= tokens_to_sell, PresaleErrors::InsufficientTokens);
        require_keys_eq!(to_account.key(), presale_account.owner);
        let user_account = &mut ctx.accounts.user_account;
        user_account.bought_amount += tokens_to_buy;
        // Create the transfer instruction
        let transfer_instruction = system_instruction::transfer(from_account.key, to_account.key, amount_lamports as u64);
        // Invoke the transfer instruction
        // when the pda is transferring, use invoke_signed
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                from_account.to_account_info(), // transformation to the AccountInfo struct
                to_account.clone(), // to_account.to_account_info()
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;


        Ok(())
    }

}

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(init, payer = owner, space = 8 + PresaleAccount::INIT_SPACE, seeds = [b"presale_account"], bump)]
    pub presale_account: Account<'info, PresaleAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePresale<'info> {
    pub owner: Signer<'info>, // why we did not put mutable here since this is a write function and lamports will be spent
    #[account(mut, has_one = owner, seeds = [b"presale_account"], bump)]
    pub presale_account: Account<'info, PresaleAccount>,
}

#[derive(Accounts)]
pub struct TransferLamportsAndBuyTokens<'info> {
    #[account(mut)]
    pub from: Signer<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    #[account(mut, seeds = [b"presale_account"], bump)]
    pub presale_account: Account<'info, PresaleAccount>,
    /*
    Update: using signer's pubkey as extra param for seed so that for every user a unique pda will be 
    created unlike before where we were just updating same account balance for every call since the seed was same
    */
    #[account(init_if_needed, payer = from, space = 8 + UserAccount::INIT_SPACE, seeds = [b"bought_amount", from.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    // pub amount_sol: u128,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct PresaleAccount {
    pub start_time: u128,
    pub end_time: u128,
    pub token_price: u128,
    pub tokens_to_sell: u128,
    pub tokens_sold: u128,
    pub owner: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub bought_amount: u128,
}

#[error_code]
pub enum PresaleErrors {
    #[msg("Invalid start and end time passed")]
    InvalidTime,
    #[msg("Zero value parameters")]
    ZeroParams,
    #[msg("Token to sell less than token sold")]
    InvalidTokenSaleNumbers,
    #[msg("Not enough lamports sent")]
    InsufficientLamports,
    #[msg("Not enough tokens")]
    InsufficientTokens,
    #[msg("To address invalid")]
    ToAddressNotOwner
}
