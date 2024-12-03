use anchor_lang::prelude::*;
//use sp1_solana;

declare_id!("2myQEiqZzJVSMHC6g42FXVpjz5NTdbXHt3ZSRKegDck4");

const DISCRIMINATOR: usize = 8;
const MAX_QUEUE_SIZE: usize = 100;
const VKEY_HASH: &str = "asdasd";

#[program]
pub mod twine_chain {
    use super::*;

    // Initialize the Deposit Message PDA
    pub fn initialize_deposit_message_pda(ctx: Context<InitializeDepositMessagePDA>) -> Result<()> {
        let deposit_message_pda = &mut ctx.accounts.deposit_message_pda;
        deposit_message_pda.deposits = Vec::new();
        Ok(())
    }

    // Initialize the Commitment Data PDA
    pub fn initialize_commitment_data_pda(ctx: Context<InitializeCommitmentDataPDA>) -> Result<()> {
        let commitment_data_pda = &mut ctx.accounts.commitment_data_pda;
        commitment_data_pda.mapping = Vec::new();
        Ok(())
    }
    
    // Append deposit message
    pub fn append_deposit_message(ctx: Context<AppendDepositMessage>,  deposit_info: DepositMessageInfo) -> Result<()> {
        let deposit_message_pda = &mut ctx.accounts.deposit_message_pda;

        // Append the deposit message to the queue
        deposit_message_pda.deposits.push(deposit_info.clone());

        emit!(DepositSuccessful {
            from: deposit_info.from,
            to: deposit_info.to,
            amount: deposit_info.amount,
        });

        Ok(())
    }

    // Commit batch (insert into CommitmentData)
    pub fn commit_batch(ctx: Context<CommitBatch>, commit_info: CommitBatchInfo) -> Result<()> {
        let commitment_data_pda = &mut ctx.accounts.commitment_data_pda;
        let public_input = Vec::new();
        // Insert the commit info into the BTreeMap
        let batch_to_commit = StoredBatchInfo{
            batch_number: commit_info.batch_number,
            batch_hash: commit_info.batch_hash,
            previous_state_root: commit_info.previous_state_root,
            state_root: commit_info.state_root,
            public_input: public_input,
        };
        commitment_data_pda.last_committed_batch_number = batch_to_commit.batch_number;
        commitment_data_pda.mapping.push((batch_to_commit.batch_number, batch_to_commit));

        Ok(())
    }

    // Finalize batch (remove from CommitmentData)
    pub fn finalize_batch(ctx: Context<FinalizeBatch>, batch_number: u64, proof_bytes: Vec<u8>, number_of_deposits: u64) -> Result<()> {
        let commitment_data_pda = &mut ctx.accounts.commitment_data_pda;
        let deposit_message_pda = &mut ctx.accounts.deposit_message_pda;

        let public_input = if let Some((_, stored_batch_info)) = commitment_data_pda.mapping.iter().find(|(num, _)| *num == batch_number) {
            stored_batch_info.public_input.clone()
        } else {
            return Err(ErrorCode::BatchNotCommitted)?;
        };
    // Get the SP1 Groth16 verification key from the `sp1-solana` crate
    //     let vk = sp1_solana::Groth16_VK_2_0_0_BYTES;
    //     // Verify the Proof
    //     sp1_solana::verify_proof(
    //         &proof_bytes,
    //         &public_input,
    //         &VKEY_HASH,
    //         vk,
    //     ).ok_or(ErrorCode::InvalidProof)?;

        let number_of_deposits_usize = number_of_deposits as usize;
        deposit_message_pda.deposits.drain(0..number_of_deposits_usize);
        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializeDepositMessagePDA<'info> {
    #[account(
        init,
        payer = user,
        space = DISCRIMINATOR + 4 + (MAX_QUEUE_SIZE * DepositMessageInfo::LEN),
        seeds = [b"deposit_message_pda"],
        bump
    )]
    deposit_message_pda: Account<'info, DepositMessagePDA>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeCommitmentDataPDA<'info> {
    #[account(
        init,
        payer = user,
        space = DISCRIMINATOR + 4 + (MAX_QUEUE_SIZE * StoredBatchInfo::LEN),
        seeds = [b"commitment_data_pda"],
        bump,
    )]
    pub commitment_data_pda: Account<'info, CommitmentData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CommitBatch<'info> {
    #[account(
        mut,
        seeds = [b"commitment_data_pda"],
        bump,
    )]
    pub commitment_data_pda: Account<'info, CommitmentData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeBatch<'info> {
    #[account(
        mut,
        seeds = [b"commitment_data_pda"],
        bump,
    )]
    pub commitment_data_pda: Account<'info, CommitmentData>,
    #[account(
        mut,
        seeds = [b"deposit_message_pda"] ,
        bump,
     )]
     pub deposit_message_pda: Account<'info, DepositMessagePDA>,
     #[account(mut)]
     pub user: Signer<'info>,
     pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AppendDepositMessage<'info> {
    #[account(
       mut,
       seeds = [b"deposit_message_pda"] ,
       bump,
    )]
    pub deposit_message_pda: Account<'info, DepositMessagePDA>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct DepositMessagePDA {
    pub deposits: Vec<DepositMessageInfo>,
}

#[account]
pub struct CommitmentData {
    pub mapping: Vec<(u64, StoredBatchInfo)>, // Batch number => StoredBatchInfo
    pub last_committed_batch_number: u64,
    pub last_finalized_batch_number: u64,
}

// The DepositInfo struct for individual deposit events
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DepositMessageInfo {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

impl DepositMessageInfo {
    const LEN: usize = 32 + 32 + 8; // Pubkey(32) + Pubkey(32) + u64(8)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CommitBatchInfo {
    pub batch_number: u64,
    pub batch_hash: [u8; 32],
    pub previous_state_root: [u8; 32],
    pub state_root: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StoredBatchInfo {
    pub batch_number: u64,
    pub batch_hash: [u8; 32],
    pub previous_state_root: [u8; 32],
    pub state_root: [u8; 32],
    pub public_input: Vec<u8>,
}

impl StoredBatchInfo {
    const LEN: usize = 8 + 32 + 32 + 32 + 100; // Adjust size for u64 + three 32-byte vectors
}

// Event for deposits
#[event]
pub struct DepositSuccessful {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

// Custom error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid input to the proof")]
    InvalidProof,
    #[msg("Batch needs to be committed before finalization")]
    BatchNotCommitted,
}