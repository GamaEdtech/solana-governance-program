import * as anchor from "@coral-xyz/anchor";
import {
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountIdempotentInstruction,
  TOKEN_2022_PROGRAM_ID
} from "@solana/spl-token";

(async () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  
  const program = anchor.workspace.GamaedtechProgram;
  const mint = new anchor.web3.PublicKey("GeutGuhcTYRf4rkbZmWDMEgjt5jHyJN4nHko38GJjQhv");

  // ---- PDA vault authority ----
  const [vaultAuthority, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault-authority")],
    program.programId
  );

  // ---- Create ATA for vault ----
  const vaultAta = getAssociatedTokenAddressSync(
    mint,
    vaultAuthority,
    true,                  // allow owner off-curve
    TOKEN_2022_PROGRAM_ID  // Token-2022
  );

  // Instruction to create ATA (idempotent = safe if already exists)
  const ix = createAssociatedTokenAccountIdempotentInstruction(
    program.provider.publicKey, // payer
    vaultAta,                   // ATA
    vaultAuthority,             // owner of ATA
    mint,                       // token
    TOKEN_2022_PROGRAM_ID       // Token-2022 program
  );

  // Send transaction
  const tx = new anchor.web3.Transaction().add(ix);

  const sig = await program.provider.sendAndConfirm(tx);
  
  console.log("Vault ATA created:", vaultAta.toString());
  console.log("Tx:", sig);
})();
