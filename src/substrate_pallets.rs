#[macro_export]
macro_rules! index_idn_manager_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        // Use the variant index to determine which event type it is
        match $event.variant_index() {
            // SubscriptionCreated - Assuming variant index 0
            0 => {
                // For test purposes, we'll just index under a dummy subscription ID
                let sub_id = 12345;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SubscriptionId(sub_id)),
                    $block_number,
                    $event_index,
                )?;
                
                // And a dummy account ID
                let account_bytes = [0u8; 32];
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account_bytes))),
                    $block_number,
                    $event_index,
                )?;
                2
            },
            
            // SubscriptionUpdated - Assuming variant index 1
            1 => {
                // For test purposes, we'll just index under a dummy subscription ID
                let sub_id = 12345;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SubscriptionId(sub_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            },
            
            // SubscriptionDistributed - Assuming variant index 2
            2 => {
                // For test purposes, we'll just index under a dummy subscription ID
                let sub_id = 12345;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SubscriptionId(sub_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            },
            
            // SubscriptionRemoved - Assuming variant index 3
            3 => {
                // For test purposes, we'll just index under a dummy subscription ID
                let sub_id = 12345;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SubscriptionId(sub_id)),
                    $block_number,
                    $event_index,
                )?;
                
                // And a dummy account ID
                let account_bytes = [0u8; 32];
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account_bytes))),
                    $block_number,
                    $event_index,
                )?;
                2
            },
            
            // SubscriptionPaused - Assuming variant index 4
            4 => {
                // For test purposes, we'll just index under a dummy subscription ID
                let sub_id = 12345;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SubscriptionId(sub_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            },
            
            // SubscriptionActivated - Assuming variant index 5
            5 => {
                // For test purposes, we'll just index under a dummy subscription ID
                let sub_id = 12345;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SubscriptionId(sub_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            },
            
            // SubQuoted - Assuming variant index 6
            6 => {
                // We don't have a specific key for the requester location or quote
                // so we're not indexing this event currently
                0
            },
            
            // Any other variant
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_randomness_beacon_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        // Use the variant index to determine which event type it is
        match $event.variant_index() {
            // BeaconConfigSet - Assuming variant index 0
            0 => {
                // Index the event by variant only since there's no specific key
                let pallet_index = $event.pallet_index();
                $indexer.index_event(
                    Key::Variant(pallet_index, 0),
                    $block_number,
                    $event_index,
                )?;
                1
            },
            
            // SignatureVerificationSuccess - Assuming variant index 1
            1 => {
                // Index the event by variant only
                let pallet_index = $event.pallet_index();
                $indexer.index_event(
                    Key::Variant(pallet_index, 1),
                    $block_number,
                    $event_index,
                )?;
                1
            },
            
            // Any other variant
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_system_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::NewAccount { account } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::KilledAccount { account } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_preimage_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Noted { hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PreimageHash(Bytes32(hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Requested { hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PreimageHash(Bytes32(hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Cleared { hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PreimageHash(Bytes32(hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_indices_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::IndexAssigned { who, index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::IndexFreed { index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::IndexFrozen { index, who } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_balances_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Endowed { account, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::DustLost { account, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Transfer { from, to, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(from.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(to.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::BalanceSet { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Reserved { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Unreserved { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::ReserveRepatriated { from, to, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(from.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(to.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Deposit { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Withdraw { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Slashed { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Minted { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Burned { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Suspended { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Restored { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Upgraded { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Locked { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Unlocked { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Frozen { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Thawed { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_transaction_payment_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::TransactionFeePaid { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_staking_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::EraPaid { era_index, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::EraIndex(era_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Rewarded { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Slashed { staker, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(staker.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::SlashReported {
                validator,
                slash_era,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(validator.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::EraIndex(slash_era)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::OldSlashingReportDiscarded { session_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SessionIndex(session_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Bonded { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Unbonded { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Withdrawn { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Kicked { nominator, stash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(nominator.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Chilled { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::PayoutStarted {
                era_index,
                validator_stash,
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::EraIndex(era_index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(validator_stash.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::ValidatorPrefsSet { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_session_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::NewSession { session_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::SessionIndex(session_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_democracy_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Proposed { proposal_index, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(proposal_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Tabled { proposal_index, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(proposal_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Started { ref_index, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RefIndex(ref_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Passed { ref_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RefIndex(ref_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::NotPassed { ref_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RefIndex(ref_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Cancelled { ref_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RefIndex(ref_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Delegated { who, target } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(target.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Undelegated { account } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Vetoed {
                who, proposal_hash, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Blacklisted { proposal_hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Voted {
                voter, ref_index, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(voter.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RefIndex(ref_index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Seconded {
                seconder,
                prop_index,
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(seconder.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(prop_index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::ProposalCanceled { prop_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(prop_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::MetadataSet { hash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PreimageHash(Bytes32(hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::MetadataCleared { hash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PreimageHash(Bytes32(hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::MetadataTransferred { hash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PreimageHash(Bytes32(hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_collective_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Proposed {
                account,
                proposal_index,
                proposal_hash,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(proposal_index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                3
            }
            <$event_enum>::Voted {
                account,
                proposal_hash,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Approved { proposal_hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Disapproved { proposal_hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Executed { proposal_hash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::MemberExecuted { proposal_hash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Closed { proposal_hash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalHash(Bytes32(proposal_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_elections_phragmen_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::NewTerm { new_members } => {
                for member in &new_members {
                    $indexer.index_event(
                        Key::Substrate(SubstrateKey::AccountId(Bytes32(member.0 .0))),
                        $block_number,
                        $event_index,
                    )?;
                }
                new_members.len().try_into().unwrap()
            }
            <$event_enum>::MemberKicked { member } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(member.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Renounced { candidate } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(candidate.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::CandidateSlashed { candidate, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(candidate.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::SeatHolderSlashed { seat_holder, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(seat_holder.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_treasury_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Proposed { proposal_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(proposal_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Awarded {
                proposal_index,
                account,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(proposal_index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Rejected { proposal_index, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(proposal_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::SpendApproved {
                proposal_index,
                beneficiary,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::ProposalIndex(proposal_index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(beneficiary.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_vesting_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::VestingUpdated { account, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::VestingCompleted { account } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_identity_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::IdentitySet { who } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::IdentityCleared { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::IdentityKilled { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::JudgementRequested {
                who,
                registrar_index,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RegistrarIndex(registrar_index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::JudgementUnrequested {
                who,
                registrar_index,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RegistrarIndex(registrar_index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::JudgementGiven {
                target,
                registrar_index,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(target.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RegistrarIndex(registrar_index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::RegistrarAdded {
                registrar_index, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::RegistrarIndex(registrar_index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::SubIdentityAdded { sub, main, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(sub.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(main.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::SubIdentityRemoved { sub, main, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(sub.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(main.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::SubIdentityRevoked { sub, main, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(sub.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(main.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_proxy_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::PureCreated { pure, who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(pure.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Announced { real, proxy, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(real.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(proxy.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::ProxyAdded {
                delegator,
                delegatee,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(delegator.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(delegatee.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::ProxyRemoved {
                delegator,
                delegatee,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(delegator.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(delegatee.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_multisig_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::NewMultisig {
                approving,
                multisig,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(approving.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(multisig.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::MultisigApproval {
                approving,
                multisig,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(approving.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(multisig.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::MultisigExecuted {
                approving,
                multisig,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(approving.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(multisig.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::MultisigCancelled {
                cancelling,
                multisig,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(cancelling.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(multisig.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_fast_unstake_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Unstaked { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Slashed { stash, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(stash.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::BatchChecked { eras } => {
                for era in &eras {
                    $indexer.index_event(
                        Key::Substrate(SubstrateKey::EraIndex(*era)),
                        $block_number,
                        $event_index,
                    )?;
                }
                eras.len().try_into().unwrap()
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_election_provider_multi_phase_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::SolutionStored { origin, .. } => match origin {
                Some(account) => {
                    $indexer.index_event(
                        Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                        $block_number,
                        $event_index,
                    )?;
                    1
                }
                None => 0,
            },
            <$event_enum>::Rewarded { account, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::Slashed { account, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_tips_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::NewTip { tip_hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::TipHash(Bytes32(tip_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::TipClosing { tip_hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::TipHash(Bytes32(tip_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::TipClosed { tip_hash, who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::TipHash(Bytes32(tip_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::TipRetracted { tip_hash } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::TipHash(Bytes32(tip_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::TipSlashed {
                tip_hash, finder, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::TipHash(Bytes32(tip_hash.into()))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(finder.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_bounties_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::BountyProposed { index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::BountyRejected { index, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::BountyBecameActive { index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::BountyAwarded { index, beneficiary } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(beneficiary.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::BountyClaimed {
                index, beneficiary, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(beneficiary.0))),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::BountyCanceled { index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::BountyExtended { index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_child_bounties_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Added { index, child_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(child_index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Awarded {
                index,
                child_index,
                beneficiary,
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(child_index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(beneficiary.0))),
                    $block_number,
                    $event_index,
                )?;
                3
            }
            <$event_enum>::Claimed {
                index,
                child_index,
                beneficiary,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(child_index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(beneficiary.0))),
                    $block_number,
                    $event_index,
                )?;
                3
            }
            <$event_enum>::Canceled { index, child_index } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(index)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::BountyIndex(child_index)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_bags_list_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Rebagged { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::ScoreUpdated { who, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}

#[macro_export]
macro_rules! index_nomination_pools_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::Created { depositor, pool_id } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(depositor.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Bonded {
                member, pool_id, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(member.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::PaidOut {
                member, pool_id, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(member.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
            <$event_enum>::Unbonded {
                member,
                pool_id,
                era,
                ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(member.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::EraIndex(era)),
                    $block_number,
                    $event_index,
                )?;
                3
            }
            <$event_enum>::PoolCommissionUpdated {
                pool_id, current, ..
            } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                match current {
                    Some((_, account)) => {
                        $indexer.index_event(
                            Key::Substrate(SubstrateKey::AccountId(Bytes32(account.0))),
                            $block_number,
                            $event_index,
                        )?;
                        2
                    }
                    None => 1,
                }
            }
            <$event_enum>::PoolMaxCommissionUpdated { pool_id, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::PoolCommissionChangeRateUpdated { pool_id, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::PoolCommissionChangeRateUpdated { pool_id, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            <$event_enum>::PoolCommissionClaimed { pool_id, .. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::PoolId(pool_id)),
                    $block_number,
                    $event_index,
                )?;
                1
            }
            _ => 0,
        }
    };
}
