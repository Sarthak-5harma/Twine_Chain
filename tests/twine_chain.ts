import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TwineChain } from "../target/types/twine_chain";

import { TokenGateway } from "../../token_gateway/target/types/token_gateway";
import * as assert from "assert";

describe("Deposit Flow", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const tokenGatewayProgram = anchor.workspace.TokenGateway as Program<TokenGateway>;
  const twineChainProgram = anchor.workspace.TwineChain as Program<TwineChain>;

  const user = provider.wallet;

  // PDAs
  let nativePDA: anchor.web3.PublicKey;
  let depositMessagePDA: anchor.web3.PublicKey;

  // Bumps
  let nativePDABump: number;
  let depositMessagePDABump: number;

  // Constants
  const depositAmount = 1_000_000_000; // 1 SOL
  const depositTo = anchor.web3.Keypair.generate().publicKey;

  before(async () => {
    // Derive Native PDA
    [nativePDA, nativePDABump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("native_pda")],
      tokenGatewayProgram.programId
    );

    // Derive DepositMessage PDA
    [depositMessagePDA, depositMessagePDABump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("deposit_message_pda")],
      twineChainProgram.programId
    );

    // Initialize Native PDA
    await tokenGatewayProgram.methods
      .initializeNativePda()
      .accounts({
        nativePda: nativePDA,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Initialize DepositMessage PDA
    await twineChainProgram.methods
      .initializeDepositMessagePda()
      .accounts({
        depositMessagePda: depositMessagePDA,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("Handles a deposit and appends to DepositMessagePDA", async () => {
    // Perform deposit
    await tokenGatewayProgram.methods
      .depositSol(depositTo, new anchor.BN(depositAmount))
      .accounts({
        nativePda: nativePDA,
        user: user.publicKey,
        depositMessagePda: depositMessagePDA,
        systemProgram: anchor.web3.SystemProgram.programId,
        twineChainProgram: twineChainProgram.programId,
      })
      .rpc();

    // Verify NativePDA has locked the amount
    const nativePDAState = await tokenGatewayProgram.account.nativePda.fetch(nativePDA);
    assert.strictEqual(nativePDAState.totalDeposits.toNumber(), depositAmount);

    // Verify DepositMessagePDA contains the deposit message
    const depositMessagePDAState = await twineChainProgram.account.depositMessagePda.fetch(depositMessagePDA);
    assert.strictEqual(depositMessagePDAState.deposits.length, 1);

    const depositMessage = depositMessagePDAState.deposits[0];
    assert.strictEqual(depositMessage.from.toString(), user.publicKey.toString());
    assert.strictEqual(depositMessage.to.toString(), depositTo.toString());
    assert.strictEqual(depositMessage.amount.toNumber(), depositAmount);
  });
});
