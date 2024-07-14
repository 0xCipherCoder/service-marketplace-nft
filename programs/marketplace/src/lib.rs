use anchor_lang::prelude::*;

declare_id!("7VEP39wxE8vD2biZw6fzBemgiE9oNyPHBrMMEMsvcAEj");

#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
