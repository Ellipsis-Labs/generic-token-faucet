import * as anchor from "@project-serum/anchor";
import { AnchorError, Program } from "@project-serum/anchor";
import { getOrCreateAssociatedTokenAccount, getAccount } from "@solana/spl-token";
import { assert, expect } from "chai";
const { SystemProgram, Transaction } = anchor.web3;
import { DevnetTokenFaucet } from "../target/types/devnet_token_faucet";
const { TOKEN_PROGRAM_ID, Token, ASSOCIATED_TOKEN_PROGRAM_ID } = require("@solana/spl-token");

describe("devnet-token-faucet", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const payer = anchor.web3.Keypair.generate();
  console.log("Payer Pubkey: ", payer.publicKey.toBase58())

  const program = anchor.workspace.DevnetTokenFaucet as Program<DevnetTokenFaucet>;

  let ticker = "SOL"
  let ticker_lowercase = ticker.toLowerCase()
  let decimals = 9
  let amount_to_mint = 10 * 10^(decimals) 

  it("Create mint", async () => {
    const signature = await provider.connection.requestAirdrop(payer.publicKey, 1_000_000_000)
    await provider.connection.confirmTransaction(signature, 'confirmed')

    const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint"),Buffer.from(ticker_lowercase)],
      program.programId
    )

    const [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint-authority"), Buffer.from(ticker_lowercase)],
      program.programId
    )

    const tx = await program.methods
      .createMint(ticker,decimals)
      .accounts({
        mint,
        mintAuthority,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
    console.log("Your transaction signature", tx);

    const mint_pda = await program.account.mintData.fetch(mintAuthority)
    expect(mint_pda.isInitialized)
    expect(mint_pda.mint == mint)
  });

  it("Mint tokens", async () => {
    const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint"),Buffer.from(ticker_lowercase)],
      program.programId
    )

    const [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint-authority"), Buffer.from(ticker_lowercase)],
      program.programId
    )

    const destination = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      TOKEN_PROGRAM_ID,
    )
    
    const ix = await program.methods
      .airdropSpl(ticker, new anchor.BN(amount_to_mint))
      .accounts({
        mint,
        mintAuthority,
        destination: destination.address,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .rpc({skipPreflight: true});

    const destionationTokenAccount = await getAccount(provider.connection, destination.address)
    expect(Number(destionationTokenAccount.amount)).to.equal(amount_to_mint)
  });

  it("Creating mint for an existing ticker should fail", async () => {
    const signature = await provider.connection.requestAirdrop(payer.publicKey, 1_000_000_000)
    await provider.connection.confirmTransaction(signature, 'confirmed')

    const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint"),Buffer.from(ticker_lowercase)],
      program.programId
    )

    const [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint-authority"), Buffer.from(ticker_lowercase)],
      program.programId
    )

    const tx = await program.methods
      .createMint(ticker,decimals)
      .accounts({
        mint,
        mintAuthority,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
    console.log("Your transaction signature", tx);

    const mint_pda = await program.account.mintData.fetch(mintAuthority)
    expect(mint_pda.isInitialized)
    expect(mint_pda.mint == mint)

    try {
      const secondTx = await program.methods
        .createMint(ticker,decimals)
        .accounts({
          mint,
          mintAuthority,
          payer: payer.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([payer])
        .rpc
      assert.ok(false)
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
          const err: AnchorError = _err;
          console.log("Error on re-initialization: ", err)
    }
  });
});
