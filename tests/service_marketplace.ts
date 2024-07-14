import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ServiceMarketplace } from "../target/types/service_marketplace";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";
import { expect } from "chai";

describe("service-marketplace", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ServiceMarketplace as Program<ServiceMarketplace>;

  let marketplace: anchor.web3.Keypair;
  let mint: anchor.web3.PublicKey;
  let vendorTokenAccount: anchor.web3.PublicKey;
  let buyerTokenAccount: anchor.web3.PublicKey;
  let marketplaceTokenAccount: anchor.web3.PublicKey;

  const vendor = anchor.web3.Keypair.generate();
  const buyer = anchor.web3.Keypair.generate();

  before(async () => {
    // Airdrop SOL to vendor and buyer
    await provider.connection.requestAirdrop(vendor.publicKey, 1000000000);
    await provider.connection.requestAirdrop(buyer.publicKey, 1000000000);

    await new Promise(resolve => setTimeout(resolve, 500));

    // Create token mint
    mint = await createMint(
      provider.connection,
      vendor,
      vendor.publicKey,
      null,
      6
    );

    // Create token accounts
    vendorTokenAccount = await createAccount(
      provider.connection,
      vendor,
      mint,
      vendor.publicKey
    );

    buyerTokenAccount = await createAccount(
      provider.connection,
      buyer,
      mint,
      buyer.publicKey
    );

    marketplaceTokenAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      provider.wallet.publicKey
    );

    // Mint tokens to buyer
    await mintTo(
      provider.connection,
      buyer,
      mint,
      buyerTokenAccount,
      vendor,
      1000000000
    );
  });

  it("Initializes the marketplace", async () => {
    marketplace = anchor.web3.Keypair.generate();
    const royaltyPercentage = 5;

    await program.methods
      .initializeMarketplace(royaltyPercentage)
      .accounts({
        marketplace: marketplace.publicKey,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([marketplace])
      .rpc();

    const marketplaceAccount = await program.account.marketplace.fetch(marketplace.publicKey);
    expect(marketplaceAccount.authority.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(marketplaceAccount.serviceCount.toNumber()).to.equal(0);
    expect(marketplaceAccount.royaltyPercentage).to.equal(royaltyPercentage);
  });

  it("Lists a service", async () => {
    const service = anchor.web3.Keypair.generate();
    const serviceMint = await createMint(
      provider.connection,
      vendor,
      vendor.publicKey,
      null,
      0
    );

    const metadata = {
      name: "Test Service",
      description: "A test service",
      price: new anchor.BN(100000000),
      isSoulbound: false,
    };

    await program.methods
      .listService(metadata)
      .accounts({
        marketplace: marketplace.publicKey,
        service: service.publicKey,
        vendor: vendor.publicKey,
        mint: serviceMint,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vendor, service])
      .rpc();

    const serviceAccount = await program.account.service.fetch(service.publicKey);
    expect(serviceAccount.vendor.toString()).to.equal(vendor.publicKey.toString());
    expect(serviceAccount.metadata.name).to.equal(metadata.name);
    expect(serviceAccount.metadata.price.toNumber()).to.equal(metadata.price.toNumber());
    expect(serviceAccount.isAvailable).to.be.true;
  });

  it("Purchases a service", async () => {
    const service = anchor.web3.Keypair.generate();
    const serviceMint = await createMint(
      provider.connection,
      vendor,
      vendor.publicKey,
      null,
      0
    );

    const metadata = {
      name: "Test Service",
      description: "A test service",
      price: new anchor.BN(100000000),
      isSoulbound: false,
    };

    await program.methods
      .listService(metadata)
      .accounts({
        marketplace: marketplace.publicKey,
        service: service.publicKey,
        vendor: vendor.publicKey,
        mint: serviceMint,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vendor, service])
      .rpc();

    const buyerNftAccount = await createAccount(
      provider.connection,
      buyer,
      serviceMint,
      buyer.publicKey
    );

    await program.methods
      .purchaseService()
      .accounts({
        marketplace: marketplace.publicKey,
        service: service.publicKey,
        buyer: buyer.publicKey,
        vendor: vendor.publicKey,
        mint: serviceMint,
        buyerTokenAccount: buyerTokenAccount,
        vendorTokenAccount: vendorTokenAccount,
        buyerNftAccount: buyerNftAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([buyer])
      .rpc();

    const serviceAccount = await program.account.service.fetch(service.publicKey);
    expect(serviceAccount.isAvailable).to.be.false;

    const buyerNftBalance = await provider.connection.getTokenAccountBalance(buyerNftAccount);
    expect(buyerNftBalance.value.uiAmount).to.equal(1);
  });

  it("Resells a service", async () => {
    const service = anchor.web3.Keypair.generate();
    const serviceMint = await createMint(
      provider.connection,
      vendor,
      vendor.publicKey,
      null,
      0
    );

    const metadata = {
      name: "Test Service",
      description: "A test service",
      price: new anchor.BN(100000000),
      isSoulbound: false,
    };

    await program.methods
      .listService(metadata)
      .accounts({
        marketplace: marketplace.publicKey,
        service: service.publicKey,
        vendor: vendor.publicKey,
        mint: serviceMint,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vendor, service])
      .rpc();

    const sellerNftAccount = await createAccount(
      provider.connection,
      vendor,
      serviceMint,
      vendor.publicKey
    );

    await mintTo(
      provider.connection,
      vendor,
      serviceMint,
      sellerNftAccount,
      vendor,
      1
    );

    const buyerNftAccount = await createAccount(
      provider.connection,
      buyer,
      serviceMint,
      buyer.publicKey
    );

    const newPrice = new anchor.BN(150000000);

    await program.methods
      .resellService(newPrice)
      .accounts({
        marketplace: marketplace.publicKey,
        service: service.publicKey,
        seller: vendor.publicKey,
        buyer: buyer.publicKey,
        mint: serviceMint,
        sellerNftAccount: sellerNftAccount,
        buyerNftAccount: buyerNftAccount,
        sellerTokenAccount: vendorTokenAccount,
        buyerTokenAccount: buyerTokenAccount,
        marketplaceTokenAccount: marketplaceTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([vendor, buyer])
      .rpc();

    const serviceAccount = await program.account.service.fetch(service.publicKey);
    expect(serviceAccount.metadata.price.toNumber()).to.equal(newPrice.toNumber());

    const buyerNftBalance = await provider.connection.getTokenAccountBalance(buyerNftAccount);
    expect(buyerNftBalance.value.uiAmount).to.equal(1);

    const sellerNftBalance = await provider.connection.getTokenAccountBalance(sellerNftAccount);
    expect(sellerNftBalance.value.uiAmount).to.equal(0);
  });
});