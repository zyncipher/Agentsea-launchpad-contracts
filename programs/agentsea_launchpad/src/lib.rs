use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod agentsea_launchpad {
    use super::*;

    /// Initialize the launchpad global state
    pub fn initialize_launchpad(
        ctx: Context<InitializeLaunchpad>,
        min_stake_amount: u64,
    ) -> Result<()> {
        let launchpad = &mut ctx.accounts.launchpad;
        launchpad.authority = ctx.accounts.authority.key();
        launchpad.agent_count = 0;
        launchpad.min_stake_amount = min_stake_amount;
        launchpad.agents_token_mint = ctx.accounts.agents_token_mint.key();
        Ok(())
    }

    /// Register a new agent (like minting an NFT identity)
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        name: String,
        metadata_uri: String,
        description: String,
    ) -> Result<()> {
        require!(name.len() <= 50, ErrorCode::NameTooLong);
        require!(metadata_uri.len() <= 200, ErrorCode::UriTooLong);
        require!(description.len() <= 500, ErrorCode::DescriptionTooLong);

        let launchpad = &mut ctx.accounts.launchpad;
        let agent = &mut ctx.accounts.agent;

        agent.agent_id = launchpad.agent_count;
        agent.owner = ctx.accounts.owner.key();
        agent.name = name;
        agent.metadata_uri = metadata_uri;
        agent.description = description;
        agent.total_staked = 0;
        agent.reputation_score = 0;
        agent.feedback_count = 0;
        agent.is_active = true;
        agent.created_at = Clock::get()?.unix_timestamp;

        launchpad.agent_count += 1;

        emit!(AgentRegistered {
            agent_id: agent.agent_id,
            owner: agent.owner,
            name: agent.name.clone(),
            timestamp: agent.created_at,
        });

        Ok(())
    }

    /// Stake $AGENTS tokens to an agent
    pub fn stake_to_agent(
        ctx: Context<StakeToAgent>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
        require!(
            amount >= ctx.accounts.launchpad.min_stake_amount,
            ErrorCode::StakeTooLow
        );

        let agent = &mut ctx.accounts.agent;
        let stake_account = &mut ctx.accounts.stake_account;

        // Transfer tokens from staker to stake vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.staker_token_account.to_account_info(),
            to: ctx.accounts.stake_vault.to_account_info(),
            authority: ctx.accounts.staker.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update stake account
        stake_account.agent = agent.key();
        stake_account.staker = ctx.accounts.staker.key();
        stake_account.amount = stake_account.amount.checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        stake_account.staked_at = Clock::get()?.unix_timestamp;

        // Update agent's total staked
        agent.total_staked = agent.total_staked.checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        emit!(TokensStaked {
            agent_id: agent.agent_id,
            staker: ctx.accounts.staker.key(),
            amount,
            timestamp: stake_account.staked_at,
        });

        Ok(())
    }

    /// Unstake $AGENTS tokens from an agent
    pub fn unstake_from_agent(
        ctx: Context<UnstakeFromAgent>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let stake_account = &mut ctx.accounts.stake_account;
        let agent = &mut ctx.accounts.agent;

        require!(
            stake_account.amount >= amount,
            ErrorCode::InsufficientStake
        );

        // Transfer tokens from stake vault back to staker
        let launchpad = &ctx.accounts.launchpad;
        let (_, bump) = Pubkey::find_program_address(&[b"launchpad"], ctx.program_id);
        let seeds = &[
            b"launchpad".as_ref(),
            &[bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.staker_token_account.to_account_info(),
            authority: launchpad.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        // Update stake account
        stake_account.amount = stake_account.amount.checked_sub(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        // Update agent's total staked
        agent.total_staked = agent.total_staked.checked_sub(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        emit!(TokensUnstaked {
            agent_id: agent.agent_id,
            staker: ctx.accounts.staker.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Give feedback to an agent 
    pub fn give_feedback(
        ctx: Context<GiveFeedback>,
        score: u8,
        comment_uri: String,
    ) -> Result<()> {
        require!(score <= 100, ErrorCode::InvalidScore);
        require!(comment_uri.len() <= 200, ErrorCode::UriTooLong);

        let agent = &mut ctx.accounts.agent;
        let feedback = &mut ctx.accounts.feedback;

        feedback.agent = agent.key();
        feedback.reviewer = ctx.accounts.reviewer.key();
        feedback.score = score;
        feedback.comment_uri = comment_uri;
        feedback.timestamp = Clock::get()?.unix_timestamp;

        // Update agent reputation (simple average)
        let total_score = (agent.reputation_score as u64)
            .checked_mul(agent.feedback_count as u64)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_add(score as u64)
            .ok_or(ErrorCode::MathOverflow)?;

        agent.feedback_count += 1;
        agent.reputation_score = (total_score / agent.feedback_count as u64) as u8;

        emit!(FeedbackGiven {
            agent_id: agent.agent_id,
            reviewer: ctx.accounts.reviewer.key(),
            score,
            timestamp: feedback.timestamp,
        });

        Ok(())
    }

    /// Update agent metadata (only owner)
    pub fn update_agent_metadata(
        ctx: Context<UpdateAgentMetadata>,
        new_metadata_uri: String,
        new_description: String,
    ) -> Result<()> {
        require!(new_metadata_uri.len() <= 200, ErrorCode::UriTooLong);
        require!(new_description.len() <= 500, ErrorCode::DescriptionTooLong);

        let agent = &mut ctx.accounts.agent;
        agent.metadata_uri = new_metadata_uri;
        agent.description = new_description;

        Ok(())
    }
}


#[account]
pub struct Launchpad {
    pub authority: Pubkey,
    pub agent_count: u64,
    pub min_stake_amount: u64,
    pub agents_token_mint: Pubkey,
}

#[account]
pub struct Agent {
    pub agent_id: u64,
    pub owner: Pubkey,
    pub name: String,
    pub metadata_uri: String,
    pub description: String,
    pub total_staked: u64,
    pub reputation_score: u8,
    pub feedback_count: u32,
    pub is_active: bool,
    pub created_at: i64,
}

#[account]
pub struct StakeAccount {
    pub agent: Pubkey,
    pub staker: Pubkey,
    pub amount: u64,
    pub staked_at: i64,
}

#[account]
pub struct Feedback {
    pub agent: Pubkey,
    pub reviewer: Pubkey,
    pub score: u8,
    pub comment_uri: String,
    pub timestamp: i64,
}


#[derive(Accounts)]
pub struct InitializeLaunchpad<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8 + 32,
        seeds = [b"launchpad"],
        bump
    )]
    pub launchpad: Account<'info, Launchpad>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub agents_token_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 8 + 32 + (4 + 50) + (4 + 200) + (4 + 500) + 8 + 1 + 4 + 1 + 8,
        seeds = [b"agent", launchpad.agent_count.to_le_bytes().as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,

    #[account(mut, seeds = [b"launchpad"], bump)]
    pub launchpad: Account<'info, Launchpad>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeToAgent<'info> {
    #[account(mut)]
    pub agent: Account<'info, Agent>,

    #[account(
        init_if_needed,
        payer = staker,
        space = 8 + 32 + 32 + 8 + 8,
        seeds = [b"stake", agent.key().as_ref(), staker.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(seeds = [b"launchpad"], bump)]
    pub launchpad: Account<'info, Launchpad>,

    #[account(
        init_if_needed,
        payer = staker,
        token::mint = agents_token_mint,
        token::authority = launchpad,
        seeds = [b"stake_vault", agent.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staker: Signer<'info>,

    #[account(
        mut,
        constraint = staker_token_account.owner == staker.key(),
        constraint = staker_token_account.mint == agents_token_mint.key()
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub agents_token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeFromAgent<'info> {
    #[account(mut)]
    pub agent: Account<'info, Agent>,

    #[account(
        mut,
        seeds = [b"stake", agent.key().as_ref(), staker.key().as_ref()],
        bump,
        constraint = stake_account.staker == staker.key()
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(seeds = [b"launchpad"], bump)]
    pub launchpad: Account<'info, Launchpad>,

    #[account(
        mut,
        seeds = [b"stake_vault", agent.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staker: Signer<'info>,

    #[account(
        mut,
        constraint = staker_token_account.owner == staker.key(),
        constraint = staker_token_account.mint == launchpad.agents_token_mint
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct GiveFeedback<'info> {
    #[account(mut)]
    pub agent: Account<'info, Agent>,

    #[account(
        init,
        payer = reviewer,
        space = 8 + 32 + 32 + 1 + (4 + 200) + 8,
        seeds = [b"feedback", agent.key().as_ref(), reviewer.key().as_ref()],
        bump
    )]
    pub feedback: Account<'info, Feedback>,

    #[account(mut)]
    pub reviewer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAgentMetadata<'info> {
    #[account(
        mut,
        constraint = agent.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub agent: Account<'info, Agent>,

    pub owner: Signer<'info>,
}


#[event]
pub struct AgentRegistered {
    pub agent_id: u64,
    pub owner: Pubkey,
    pub name: String,
    pub timestamp: i64,
}

#[event]
pub struct TokensStaked {
    pub agent_id: u64,
    pub staker: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensUnstaked {
    pub agent_id: u64,
    pub staker: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct FeedbackGiven {
    pub agent_id: u64,
    pub reviewer: Pubkey,
    pub score: u8,
    pub timestamp: i64,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Name is too long (max 50 characters)")]
    NameTooLong,

    #[msg("URI is too long (max 200 characters)")]
    UriTooLong,

    #[msg("Description is too long (max 500 characters)")]
    DescriptionTooLong,

    #[msg("Invalid amount (must be > 0)")]
    InvalidAmount,

    #[msg("Stake amount is below minimum required")]
    StakeTooLow,

    #[msg("Insufficient stake in account")]
    InsufficientStake,

    #[msg("Invalid score (must be 0-100)")]
    InvalidScore,

    #[msg("Unauthorized operation")]
    Unauthorized,

    #[msg("Math overflow")]
    MathOverflow,
}
