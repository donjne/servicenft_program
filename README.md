# Marketplace Program

This Solana program uses Anchor and the SPL token metadata library to create and manage NFTs for a service marketplace. It provides functionalities for listing services as NFTs, purchasing services, and handling royalties.

## Table of Contents

- [Installation](#installation)
  - [Prerequisites](#prerequisites)
  - [Clone the Repository](#clone-the-repository)
  - [Build the project](#build-the-project)
  - [Deploy the Project](#deploy-the-project)
- [Usage](#usage)
  - [Initialize Marketplace](#initialize-marketplace)
  - [List Service](#list-service)
  - [Purchase Service from Vendor](#purchase-service-from-vendor)
  - [Purchase Service from Buyer](#purchase-service-from-buyer)
- [Accounts](#accounts)
  - [InitializeMarketplace](#initializemarketplace)
  - [MintNFT](#mintnft)
  - [Marketplace](#marketplace)
  - [ServiceNFT](#servicenft)
  - [URI](#example-uri-json)
- [Error Codes](#error-codes)

## Installation

### Prerequisites

Ensure you have the following tools installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solanalabs.com/cli/install)
- [Anchor](https://www.anchor-lang.com/docs/installation)

### Clone the Repository

Clone the repository containing the `marketplace` program:

```shell
git clone https://github.com/donjne/servicenft_program.git
cd marketplace
```

### Build the Project

Build the project using anchor:

```shell
anchor build
```

### Deploy the project

Build the project using anchor:

```shell
anchor deploy
```

## Usage

### Initialize Marketplace

To initialize the marketplace, call the initialize_marketplace instruction. This sets up the marketplace authority and the royalty percentage.

```rust
pub fn initialize_marketplace(ctx: Context<InitializeMarketplace>) -> Result<()>
```

### List Service

To list a service as an NFT, call the list_service instruction with the required metadata for the NFT.

```rust
pub fn list_service(ctx: Context<MintNFT>, metadata: ServiceNFT) -> Result<()>
```

### Purchase Service from Vendor

To purchase a service NFT from a vendor, call the purchase_service_from_vendor instruction with the token and NFT amounts.

```rust
pub fn purchase_service_from_vendor(ctx: Context<MintNFT>, token_amount: u64, nft_amount: u64) -> Result<()>
```

### Purchase Service from Buyer

To purchase a service NFT from a buyer, call the purchase_service_from_buyer instruction with the token and NFT amounts. This handles the royalty distribution.

```rust
pub fn purchase_service_from_buyer(ctx: Context<MintNFT>, token_amount: u64, nft_amount: u64) -> Result<()>
```

## Accounts

### InitializeMarketplace

The InitializeMarketplace account struct is used in the initialize_marketplace instruction to set up the marketplace.

```rust
#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 33, seeds = [b"servicemarketplace"], bump)]
    pub marketplace: Account<'info, Marketplace>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
```

### MintNFT

The MintNFT account struct is used in the list_service, purchase_service_from_vendor, and purchase_service_from_buyer instructions to handle NFT minting, metadata creation, and transfers.

```rust
#[derive(Accounts)]
#[instruction(params: ServiceNFT)]
pub struct MintNFT<'info> {
    #[account(mut, signer)]
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
```

### Marketplace

The Marketplace account struct stores the marketplace authority and royalty percentage.

```rust
#[account]
pub struct Marketplace {
    pub authority: Pubkey,
    pub royalty_percentage: u8,
}
```

### ServiceNFT

The ServiceNFT account struct stores the metadata for a service NFT.

```rust
#[account]
pub struct ServiceNFT {
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub uri: String, // We can store additional offchain data on arweave and generate a uri link, eg. https://v2.akord.com/public/vaults/active/xdmaYXBiPF6GS9g0yf3vg7JumVNwWQ_c0aa0Mdz0SnE/gallery#3a540011-9043-4980-b458-8e7d16904d79
    pub soulbound: bool,
    pub duration: u64,
    pub terms_of_service: String,
    pub price: u64,
}
```

### Example URI JSON

```json
{
    "name": "DeGood",
    "symbol": "DGN",
    "description": "Fav",
    "image": "https://nftlately.com/wp-content/uploads/2022/05/logo.jpg",
    "attributes": [
        {
            "trait_type": "type",
            "value": "value"
        }
    ],
    "properties": {
        "creators": [
            {
                "address": "creators's wallet address",
                "share": "royalty"
            }
        ]
    },
    "collection": {
        "name": "name",
        "family": "family"
    }
}
```

### Error Codes

The program defines custom error codes to handle specific errors.

```rust
#[error_code]
pub enum ErrorCodes {
    #[msg("Inequivalent SOL amount.")]
    Inequivalent,
    #[msg("Invalid Action: Soulbound tokens cannot be purchased.")]
    InvalidAction,
}
```

This error is used in the purchase_service_from_vendor and purchase_service_from_buyer instructions to handle inequivalent SOL amounts and invalid actions for soulbound tokens.
