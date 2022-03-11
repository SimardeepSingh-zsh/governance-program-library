use crate::{
    error::NftVoterError,
    state::{Registrar, VoterWeightRecord},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount};
use mpl_token_metadata::state::Metadata;
use spl_governance_addin_api::voter_weight::VoterWeightAction;

#[derive(Accounts)]
#[instruction(voter_weight_action:VoterWeightAction)]
pub struct UpdateVoterWeightRecord<'info> {
    pub registrar: Account<'info, Registrar>,

    #[account(
        mut,
        constraint = voter_weight_record.realm == registrar.realm 
        @ NftVoterError::InvalidVoterWeightRecordRealm,

        constraint = voter_weight_record.governing_token_mint == registrar.governing_token_mint
        @ NftVoterError::InvalidVoterWeightRecordMint,
    )]
    pub voter_weight_record: Account<'info, VoterWeightRecord>,

    pub nft_token: Account<'info, TokenAccount>,

    pub nft_metadata: UncheckedAccount<'info>,
}

pub fn update_voter_weight_record(
    ctx: Context<UpdateVoterWeightRecord>,
    voter_weight_action: VoterWeightAction,
) -> Result<()> {

    // CastVote can't be evaluated using this instruction 
    require!(
        voter_weight_action != VoterWeightAction::CastVote,
        NftVoterError::CastVoteIsNotAllowed
    );

    // TODO: nft_token account owner/initialized 

    // voter_weight_record.governing_token_owner must be the owner of the NFT
    require!(
        ctx.accounts.nft_token.owner == ctx.accounts.voter_weight_record.governing_token_owner,
        NftVoterError::VoterDoesNotOwnNft
    );

    // TODO: change to get for_token
    let nft_metadata = Metadata::from_account_info(&ctx.accounts.nft_metadata)?;
    // TODO: Verify metadata account owner/initialized 

    // The metadata mint must be the same as the token mint
    require!(
        nft_metadata.mint == ctx.accounts.nft_token.mint,
        NftVoterError::TokenMetadataDoesNotMatch
    );

    let collection = nft_metadata.collection.unwrap();

    // It must have a collection and the collection must be verified 
    require!(
        collection.verified,
        NftVoterError::CollectionMustBeVerified
    );


    let registrar = &mut ctx.accounts.registrar;


    let collection_config = registrar                                                   
        .collection_configs
        .iter()
        .find(|cc| cc.collection == collection.key)
        .ok_or(NftVoterError::CollectionNotFound)?;


    let voter_weight_record = &mut ctx.accounts.voter_weight_record;

    voter_weight_record.voter_weight = collection_config.weight as u64;

    // Record is only valid as of the current slot
    voter_weight_record.voter_weight_expiry = Some(Clock::get()?.slot);

    // Set the action to make it specific and prevent being used for voting
    voter_weight_record.weight_action = Some(voter_weight_action);
    voter_weight_record.weight_action_target = None;

    Ok(())
}
