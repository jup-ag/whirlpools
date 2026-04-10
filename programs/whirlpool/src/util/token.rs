use crate::state::{PositionBundle, Whirlpool};
use anchor_lang::prelude::*;
use anchor_spl::metadata::{self, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3};
use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, Burn, CloseAccount, Mint, MintTo, SetAuthority,
    Token, TokenAccount, Transfer,
};

use crate::constants::nft::{
    WPB_METADATA_NAME_PREFIX, WPB_METADATA_SYMBOL, WPB_METADATA_URI, WP_METADATA_NAME,
    WP_METADATA_SYMBOL, WP_METADATA_URI,
};

pub fn transfer_from_owner_to_vault<'info>(
    position_authority: &Signer<'info>,
    token_owner_account: &Account<'info, TokenAccount>,
    token_vault: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    token::transfer(
        CpiContext::new(
            token_program.key(),
            Transfer {
                from: token_owner_account.to_account_info(),
                to: token_vault.to_account_info(),
                authority: position_authority.to_account_info(),
            },
        ),
        amount,
    )
}

pub fn transfer_from_vault_to_owner<'info>(
    whirlpool: &Account<'info, Whirlpool>,
    token_vault: &Account<'info, TokenAccount>,
    token_owner_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    token::transfer(
        CpiContext::new_with_signer(
            token_program.key(),
            Transfer {
                from: token_vault.to_account_info(),
                to: token_owner_account.to_account_info(),
                authority: whirlpool.to_account_info(),
            },
            &[&whirlpool.seeds()],
        ),
        amount,
    )
}

pub fn burn_and_close_user_position_token<'info>(
    token_authority: &Signer<'info>,
    receiver: &UncheckedAccount<'info>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    token::burn(
        CpiContext::new(
            token_program.key(),
            Burn {
                mint: position_mint.to_account_info(),
                from: position_token_account.to_account_info(),
                authority: token_authority.to_account_info(),
            },
        ),
        1,
    )?;

    token::close_account(CpiContext::new(
        token_program.key(),
        CloseAccount {
            account: position_token_account.to_account_info(),
            destination: receiver.to_account_info(),
            authority: token_authority.to_account_info(),
        },
    ))?;

    Ok(())
}

pub fn mint_position_token_and_remove_authority<'info>(
    whirlpool: &Account<'info, Whirlpool>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    mint_position_token(
        whirlpool,
        position_mint,
        position_token_account,
        token_program,
    )?;
    remove_position_token_mint_authority(whirlpool, position_mint, token_program)
}

#[allow(clippy::too_many_arguments)]
pub fn mint_position_token_with_metadata_and_remove_authority<'info>(
    whirlpool: &Account<'info, Whirlpool>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    position_metadata_account: &UncheckedAccount<'info>,
    metadata_update_auth: &UncheckedAccount<'info>,
    funder: &Signer<'info>,
    metadata_program: &Program<'info, metadata::Metadata>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    mint_position_token(
        whirlpool,
        position_mint,
        position_token_account,
        token_program,
    )?;

    let metadata_mint_auth_account = whirlpool;
    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            metadata_program.key(),
            CreateMetadataAccountsV3 {
                metadata: position_metadata_account.to_account_info(),
                mint: position_mint.to_account_info(),
                mint_authority: metadata_mint_auth_account.to_account_info(),
                update_authority: metadata_update_auth.to_account_info(),
                payer: funder.to_account_info(),
                rent: rent.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&metadata_mint_auth_account.seeds()],
        ),
        DataV2 {
            name: WP_METADATA_NAME.to_string(),
            symbol: WP_METADATA_SYMBOL.to_string(),
            uri: WP_METADATA_URI.to_string(),
            creators: None,
            seller_fee_basis_points: 0,
            collection: None,
            uses: None,
        },
        true,
        false,
        None,
    )?;

    remove_position_token_mint_authority(whirlpool, position_mint, token_program)
}

fn mint_position_token<'info>(
    whirlpool: &Account<'info, Whirlpool>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    token::mint_to(
        CpiContext::new_with_signer(
            token_program.key(),
            MintTo {
                mint: position_mint.to_account_info(),
                to: position_token_account.to_account_info(),
                authority: whirlpool.to_account_info(),
            },
            &[&whirlpool.seeds()],
        ),
        1,
    )
}

fn remove_position_token_mint_authority<'info>(
    whirlpool: &Account<'info, Whirlpool>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    token::set_authority(
        CpiContext::new_with_signer(
            token_program.key(),
            SetAuthority {
                current_authority: whirlpool.to_account_info(),
                account_or_mint: position_mint.to_account_info(),
            },
            &[&whirlpool.seeds()],
        ),
        AuthorityType::MintTokens,
        None,
    )
}

pub fn mint_position_bundle_token_and_remove_authority<'info>(
    position_bundle: &Account<'info, PositionBundle>,
    position_bundle_mint: &Account<'info, Mint>,
    position_bundle_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    position_bundle_seeds: &[&[u8]],
) -> Result<()> {
    mint_position_bundle_token(
        position_bundle,
        position_bundle_mint,
        position_bundle_token_account,
        token_program,
        position_bundle_seeds,
    )?;
    remove_position_bundle_token_mint_authority(
        position_bundle,
        position_bundle_mint,
        token_program,
        position_bundle_seeds,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn mint_position_bundle_token_with_metadata_and_remove_authority<'info>(
    funder: &Signer<'info>,
    position_bundle: &Account<'info, PositionBundle>,
    position_bundle_mint: &Account<'info, Mint>,
    position_bundle_token_account: &Account<'info, TokenAccount>,
    position_bundle_metadata: &UncheckedAccount<'info>,
    metadata_update_auth: &UncheckedAccount<'info>,
    metadata_program: &Program<'info, metadata::Metadata>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
    position_bundle_seeds: &[&[u8]],
) -> Result<()> {
    mint_position_bundle_token(
        position_bundle,
        position_bundle_mint,
        position_bundle_token_account,
        token_program,
        position_bundle_seeds,
    )?;

    let mint_address = position_bundle_mint.key().to_string();
    let mut nft_name = String::from(WPB_METADATA_NAME_PREFIX);
    nft_name += " ";
    nft_name += &mint_address[0..4];
    nft_name += "...";
    nft_name += &mint_address[mint_address.len() - 4..];

    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            metadata_program.key(),
            CreateMetadataAccountsV3 {
                metadata: position_bundle_metadata.to_account_info(),
                mint: position_bundle_mint.to_account_info(),
                mint_authority: position_bundle.to_account_info(),
                update_authority: metadata_update_auth.to_account_info(),
                payer: funder.to_account_info(),
                rent: rent.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[position_bundle_seeds],
        ),
        DataV2 {
            name: nft_name,
            symbol: WPB_METADATA_SYMBOL.to_string(),
            uri: WPB_METADATA_URI.to_string(),
            creators: None,
            seller_fee_basis_points: 0,
            collection: None,
            uses: None,
        },
        true,
        false,
        None,
    )?;

    remove_position_bundle_token_mint_authority(
        position_bundle,
        position_bundle_mint,
        token_program,
        position_bundle_seeds,
    )
}

fn mint_position_bundle_token<'info>(
    position_bundle: &Account<'info, PositionBundle>,
    position_bundle_mint: &Account<'info, Mint>,
    position_bundle_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    position_bundle_seeds: &[&[u8]],
) -> Result<()> {
    token::mint_to(
        CpiContext::new_with_signer(
            token_program.key(),
            MintTo {
                mint: position_bundle_mint.to_account_info(),
                to: position_bundle_token_account.to_account_info(),
                authority: position_bundle.to_account_info(),
            },
            &[position_bundle_seeds],
        ),
        1,
    )
}

fn remove_position_bundle_token_mint_authority<'info>(
    position_bundle: &Account<'info, PositionBundle>,
    position_bundle_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
    position_bundle_seeds: &[&[u8]],
) -> Result<()> {
    token::set_authority(
        CpiContext::new_with_signer(
            token_program.key(),
            SetAuthority {
                current_authority: position_bundle.to_account_info(),
                account_or_mint: position_bundle_mint.to_account_info(),
            },
            &[position_bundle_seeds],
        ),
        AuthorityType::MintTokens,
        None,
    )
}

pub fn burn_and_close_position_bundle_token<'info>(
    position_bundle_authority: &Signer<'info>,
    receiver: &UncheckedAccount<'info>,
    position_bundle_mint: &Account<'info, Mint>,
    position_bundle_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    burn_and_close_user_position_token(
        position_bundle_authority,
        receiver,
        position_bundle_mint,
        position_bundle_token_account,
        token_program,
    )
}
