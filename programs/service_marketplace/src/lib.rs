use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("BxRHkqPFi17qnGMM9EHiMjGoJFC36mBtGrPhZ6Tt6x1m");

#[program]
pub mod service_marketplace {
    use super::*;

    pub fn initialize_marketplace(
        ctx: Context<InitializeMarketplace>,
        royalty_percentage: u8,
    ) -> Result<()> {
        require!(royalty_percentage <= 100, MarketplaceError::InvalidRoyaltyPercentage);

        let marketplace = &mut ctx.accounts.marketplace;
        marketplace.authority = ctx.accounts.authority.key();
        marketplace.service_count = 0;
        marketplace.royalty_percentage = royalty_percentage;

        Ok(())
    }

    pub fn list_service(
        ctx: Context<ListService>,
        metadata: ServiceMetadata,
    ) -> Result<()> {
        let marketplace = &mut ctx.accounts.marketplace;
        let service = &mut ctx.accounts.service;

        service.vendor = ctx.accounts.vendor.key();
        service.metadata = metadata;
        service.mint = ctx.accounts.mint.key();
        service.is_available = true;

        marketplace.service_count += 1;

        Ok(())
    }

    pub fn purchase_service(ctx: Context<PurchaseService>) -> Result<()> {
        let service = &mut ctx.accounts.service;
        require!(service.is_available, MarketplaceError::ServiceNotAvailable);

        let price = service.metadata.price;

        // Transfer tokens from buyer to vendor
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.buyer_token_account.to_account_info(),
                    to: ctx.accounts.vendor_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            price,
        )?;

        // Mint NFT to buyer
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.buyer_nft_account.to_account_info(),
                    authority: ctx.accounts.mint.to_account_info(),
                },
            ),
            1,
        )?;

        service.is_available = false;

        Ok(())
    }

    pub fn resell_service(ctx: Context<ResellService>, new_price: u64) -> Result<()> {
        let service = &mut ctx.accounts.service;
        let marketplace = &ctx.accounts.marketplace;

        require!(!service.metadata.is_soulbound, MarketplaceError::SoulboundNFTCannotBeResold);

        // Calculate royalty
        let royalty_amount = (new_price * marketplace.royalty_percentage as u64) / 100;
        let seller_amount = new_price - royalty_amount;

        // Transfer NFT from seller to buyer
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.seller_nft_account.to_account_info(),
                    to: ctx.accounts.buyer_nft_account.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
        )?;

        // Transfer payment from buyer to seller
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.buyer_token_account.to_account_info(),
                    to: ctx.accounts.seller_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            seller_amount,
        )?;

        // Transfer royalty to marketplace authority
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.buyer_token_account.to_account_info(),
                    to: ctx.accounts.marketplace_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            royalty_amount,
        )?;

        service.metadata.price = new_price;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ServiceMetadata {
    pub name: String,
    pub description: String,
    pub price: u64,
    pub is_soulbound: bool,
}

#[account]
pub struct Marketplace {
    pub authority: Pubkey,
    pub service_count: u64,
    pub royalty_percentage: u8,
}

#[account]
pub struct Service {
    pub vendor: Pubkey,
    pub metadata: ServiceMetadata,
    pub mint: Pubkey,
    pub is_available: bool,
}

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 1
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListService<'info> {
    #[account(mut)]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        init,
        payer = vendor,
        space = 8 + 32 + 200 + 32 + 1
    )]
    pub service: Account<'info, Service>,
    #[account(mut)]
    pub vendor: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchaseService<'info> {
    #[account(mut)]
    pub marketplace: Account<'info, Marketplace>,
    #[account(mut)]
    pub service: Account<'info, Service>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vendor: AccountInfo<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vendor_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_nft_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ResellService<'info> {
    pub marketplace: Account<'info, Marketplace>,
    #[account(mut)]
    pub service: Account<'info, Service>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub seller_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub marketplace_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum MarketplaceError {
    #[msg("Invalid royalty percentage")]
    InvalidRoyaltyPercentage,
    #[msg("Service is not available")]
    ServiceNotAvailable,
    #[msg("Soulbound NFT cannot be resold")]
    SoulboundNFTCannotBeResold,
}