use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount, Transfer, transfer},
};
use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata as Meta}, 
    types::DataV2,
    };

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("F8YCVd2YFF6WLkqpdBZEpf9rfVhXXUAWUhPPaW3GNG2u");

#[program]
mod marketplace {
    use super::*;

    /// Initializes the marketplace
    pub fn initialize_marketplace(ctx: Context<InitializeMarketplace>) -> Result<()> {
        let marketplace = &mut ctx.accounts.marketplace;
        marketplace.authority = ctx.accounts.user.key();
        marketplace.royalty_percentage = 0;

        Ok(())
    }

    /// Lists a service as an NFT
    pub fn list_service(ctx: Context<MintNFT>, metadata: ServiceNFT) -> Result<()> {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.associated_token_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        );
        mint_to(cpi_context, 1)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        // Create metadata for the NFT
        let token_data = DataV2 {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(cpi_context, token_data, false, true, None)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        create_master_edition_v3(cpi_context, None)?;

        msg!("Token mint created successfully.");

        Ok(())
    }

    /// Purchases a service NFT
    pub fn purchase_service_from_vendor(ctx: Context<MintNFT>, token_amount: u64, nft_amount: u64) -> Result<()> {
        let service_nft = &mut ctx.accounts.service_nft;
        if token_amount != service_nft.price {
            return Err(ErrorCodes::Inequivalent.into())
        }

        if service_nft.soulbound {
            return  Err(ErrorCodes::InvalidAction.into());
        }

        // Transfer tokens from buyer to vendor
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.associated_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program_sol = ctx.accounts.token_program.to_account_info();
        let cpi_context_sol = CpiContext::new(cpi_program_sol, cpi_accounts);
        transfer(cpi_context_sol, token_amount)?;

        // Transfer NFT from vendor to buyer to complete swap
        let cpi_accounts = Transfer {
            from: ctx.accounts.associated_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program_sol = ctx.accounts.token_program.to_account_info();
        let cpi_context_sol = CpiContext::new(cpi_program_sol, cpi_accounts);
        transfer(cpi_context_sol, nft_amount)?;

        Ok(())
    }

        pub fn purchase_service_from_buyer(ctx: Context<MintNFT>, token_amount: u64, nft_amount: u64) -> Result<()> {
        let service_nft = &mut ctx.accounts.service_nft;
        let marketplace = &ctx.accounts.marketplace;
        if token_amount != service_nft.price {
            return Err(ErrorCodes::Inequivalent.into())
        }

        if service_nft.soulbound {
            return  Err(ErrorCodes::InvalidAction.into());
        }

        // Calculate Royalty Amount
        let royalty_amount = token_amount * marketplace.royalty_percentage as u64 / 100;
        let amount_after_royalty = token_amount - royalty_amount;

        // Transfer 90% of the token amount to the buyer firstly
        let cpi_accounts = Transfer {
            from: ctx.accounts.third_party_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program_sol = ctx.accounts.token_program.to_account_info();
        let cpi_context_sol = CpiContext::new(cpi_program_sol, cpi_accounts);
        transfer(cpi_context_sol, amount_after_royalty)?;


        // Transfer 10% of the token amount to the escrow account secondly
        let cpi_accounts = Transfer {
            from: ctx.accounts.third_party_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program_sol = ctx.accounts.token_program.to_account_info();
        let cpi_context_sol = CpiContext::new(cpi_program_sol, cpi_accounts);
        transfer(cpi_context_sol, royalty_amount)?;

        let bump = &[ctx.bumps.escrow_token_account];
        let seeds: &[&[u8]] = &[b"vault".as_ref(), bump];
        let signer_seeds = &[&seeds[..]];


        // Tranfer NFT from escrow to vendor thirdly to fulfill the royalties contract
        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.associated_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program_sol = ctx.accounts.system_program.to_account_info();
        let cpi_context_sol = CpiContext::new(cpi_program_sol, cpi_accounts).with_signer(signer_seeds);
        transfer(cpi_context_sol, royalty_amount)?;

        // Transfer NFT from buyer to third party to complete agreement between the buyer and third party
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.third_party_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program_sol = ctx.accounts.token_program.to_account_info();
        let cpi_context_sol = CpiContext::new(cpi_program_sol, cpi_accounts);
        transfer(cpi_context_sol, nft_amount)?;
        
        Ok(())
    }


}

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 33, seeds = [b"servicemarketplace"], bump)]
    pub marketplace: Account<'info, Marketplace>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Marketplace {
    pub authority: Pubkey,
    pub royalty_percentage: u8,
}


#[account]
pub struct ServiceNFT {
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub uri: String,
    pub soulbound: bool,
    pub duration: u64,
    pub terms_of_service: String,
    pub price: u64,
}

#[derive(Accounts)]
#[instruction(
    params: ServiceNFT
)]
pub struct MintNFT<'info> {
    #[account(mut, signer)]
    /// CHECK: ok, we are passing in this account ourselves
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = signer.key(),
        mint::freeze_authority = signer.key(),
        seeds = [b"mint"],
        bump
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: address
    pub metadata_account: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            token_metadata_program.key().as_ref(),
            mint.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: address
    pub master_edition_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub third_party_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault".as_ref()],
        bump
    )]
    pub escrow_token_account: SystemAccount<'info>,

    #[account(mut)]
    pub service_nft: Account<'info, ServiceNFT>,

    #[account(mut)]
    pub marketplace: Account<'info, Marketplace>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum ErrorCodes {
    #[msg("Inequivalent SOL amount.")]
    Inequivalent,
    #[msg("Invalid Action: Soulbound tokens annot be purchased")]
    InvalidAction,
}