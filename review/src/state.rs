use std::collections::HashMap;

use linera_sdk::{
    base::{ArithmeticError, Owner},
    views::{MapView, RegisterView, ViewStorageContext},
};
use linera_views::views::{GraphQLView, RootView, View};
use review::{Asset, Content, InitialState, Reviewer};
use thiserror::Error;

#[derive(RootView, GraphQLView)]
#[view(context = "ViewStorageContext")]
pub struct Review {
    pub reviewers: MapView<Owner, Reviewer>,
    pub reviewer_number: RegisterView<u16>,
    pub reviewer_applications: MapView<Owner, Reviewer>,
    pub content_applications: MapView<String, Content>,
    pub asset_applications: MapView<u64, Asset>,
    pub content_approved_threshold: RegisterView<u16>,
    pub content_rejected_threshold: RegisterView<u16>,
    pub asset_approved_threshold: RegisterView<u16>,
    pub asset_rejected_threshold: RegisterView<u16>,
    pub reviewer_approved_threshold: RegisterView<u16>,
    pub reviewer_rejected_threshold: RegisterView<u16>,
}

#[allow(dead_code)]
impl Review {
    pub(crate) async fn initialize(
        &mut self,
        creator: Owner,
        state: InitialState,
    ) -> Result<(), StateError> {
        self.content_approved_threshold
            .set(state.content_approved_threshold);
        self.content_rejected_threshold
            .set(state.content_rejected_threshold);
        self.asset_approved_threshold
            .set(state.asset_approved_threshold);
        self.asset_rejected_threshold
            .set(state.asset_rejected_threshold);
        self.reviewer_rejected_threshold
            .set(state.reviewer_rejected_threshold);
        self.reviewer_rejected_threshold
            .set(state.reviewer_rejected_threshold);
        self.reviewers.insert(
            &creator,
            Reviewer {
                reviewer: creator,
                resume: None,
                reviewers: HashMap::default(),
                approved: 1,
                rejected: 0,
            },
        )?;
        self.reviewer_number.set(1);
        Ok(())
    }

    pub(crate) async fn is_reviewer(&self, owner: Owner) -> Result<bool, StateError> {
        match self.reviewers.get(&owner).await? {
            Some(_) => Ok(true),
            _ => Ok(false),
        }
    }

    pub(crate) async fn apply_reviewer(
        &mut self,
        owner: Owner,
        resume: String,
    ) -> Result<(), StateError> {
        if self.is_reviewer(owner).await? {
            return Err(StateError::InvalidReviewer);
        }
        match self.reviewer_applications.get(&owner).await? {
            Some(_) => return Err(StateError::InvalidReviewer),
            _ => {}
        }
        self.reviewer_applications.insert(
            &owner,
            Reviewer {
                reviewer: owner,
                resume: Some(resume),
                reviewers: HashMap::default(),
                approved: 0,
                rejected: 0,
            },
        )?;
        Ok(())
    }

    pub(crate) async fn update_reviewer_resume(
        &mut self,
        owner: Owner,
        resume: String,
    ) -> Result<(), StateError> {
        match self.reviewers.get(&owner).await? {
            Some(mut reviewer) => {
                reviewer.resume = Some(resume);
                self.reviewers.insert(&owner, reviewer)?;
                return Ok(());
            }
            _ => {}
        }
        match self.reviewer_applications.get(&owner).await? {
            Some(mut reviewer) => {
                reviewer.resume = Some(resume);
                self.reviewer_applications.insert(&owner, reviewer)?;
                return Ok(());
            }
            _ => Err(StateError::InvalidReviewer),
        }
    }

    pub(crate) async fn validate_reviewer_review(
        &self,
        reviewer: Owner,
        candidate: Owner,
    ) -> Result<(), StateError> {
        if !self.is_reviewer(reviewer).await? {
            return Err(StateError::InvalidReviewer);
        }
        match self.reviewer_applications.get(&candidate).await? {
            Some(_reviewer) => match _reviewer.reviewers.get(&reviewer) {
                Some(_) => Err(StateError::AlreadyReviewed),
                _ => Ok(()),
            },
            None => Err(StateError::InvalidReviewer),
        }
    }

    pub(crate) async fn approve_reviewer(
        &mut self,
        owner: Owner,
        candidate: Owner,
    ) -> Result<bool, StateError> {
        self.validate_reviewer_review(owner, candidate.clone())
            .await?;
        match self.reviewer_applications.get(&candidate).await? {
            Some(mut reviewer) => {
                reviewer.approved += 1;
                self.reviewer_applications.insert(&candidate, reviewer)?;
            }
            _ => return Err(StateError::InvalidReviewer),
        }
        match self.reviewer_applications.get(&candidate).await? {
            Some(reviewer) => {
                let approved_threshold = *self.reviewer_approved_threshold.get();
                let reviewer_number = *self.reviewer_number.get();
                if reviewer.approved >= approved_threshold || reviewer.approved >= reviewer_number {
                    self.reviewers.insert(&candidate, reviewer.clone())?;
                    self.reviewer_applications.remove(&candidate)?;
                    self.reviewer_number.set(reviewer_number + 1);
                    return Ok(true);
                }
            }
            _ => todo!(),
        }
        Ok(false)
    }

    pub(crate) async fn reject_reviewer(
        &mut self,
        owner: Owner,
        candidate: Owner,
    ) -> Result<bool, StateError> {
        self.validate_reviewer_review(owner, candidate.clone())
            .await?;
        match self.reviewer_applications.get(&candidate).await? {
            Some(mut reviewer) => {
                reviewer.rejected += 1;
                self.reviewer_applications.insert(&candidate, reviewer)?;
            }
            _ => return Err(StateError::InvalidReviewer),
        }
        match self.reviewer_applications.get(&candidate).await? {
            Some(reviewer) => {
                let rejected_threshold = *self.reviewer_rejected_threshold.get();
                let reviewer_number = *self.reviewer_number.get();
                if reviewer.rejected >= rejected_threshold || reviewer.rejected >= reviewer_number {
                    self.reviewers.insert(&candidate, reviewer.clone())?;
                    self.reviewer_applications.remove(&candidate)?;
                    self.reviewer_number.set(reviewer_number + 1);
                    return Ok(true);
                }
            }
            _ => todo!(),
        }
        Ok(false)
    }

    pub(crate) async fn validate_content_review(
        &self,
        reviewer: Owner,
        content_cid: String,
    ) -> Result<(), StateError> {
        if !self.is_reviewer(reviewer).await? {
            return Err(StateError::InvalidReviewer);
        }
        match self.content_applications.get(&content_cid).await? {
            Some(content) => match content.reviewers.get(&reviewer) {
                Some(_) => Err(StateError::AlreadyReviewed),
                _ => Ok(()),
            },
            None => Err(StateError::InvalidContent),
        }
    }

    pub(crate) async fn submit_content(&mut self, content: Content) -> Result<(), StateError> {
        self.content_applications
            .insert(&content.clone().cid, content)?;
        Ok(())
    }

    pub(crate) async fn approve_content(
        &mut self,
        reviewer: Owner,
        content_cid: String,
    ) -> Result<Option<Content>, StateError> {
        self.validate_content_review(reviewer, content_cid.clone())
            .await?;
        match self.content_applications.get(&content_cid).await? {
            Some(mut content) => {
                content.approved += 1;
                self.content_applications.insert(&content_cid, content)?;
            }
            _ => return Err(StateError::InvalidReviewer),
        }
        match self.content_applications.get(&content_cid).await? {
            Some(content) => {
                let approved_threshold = *self.reviewer_approved_threshold.get();
                let reviewer_number = *self.reviewer_number.get();
                if content.approved >= approved_threshold || content.approved >= reviewer_number {
                    return Ok(Some(content));
                }
            }
            _ => todo!(),
        }
        Ok(None)
    }

    pub(crate) async fn reject_content(
        &mut self,
        reviewer: Owner,
        content_cid: String,
    ) -> Result<bool, StateError> {
        self.validate_content_review(reviewer, content_cid.clone())
            .await?;
        match self.content_applications.get(&content_cid).await? {
            Some(mut content) => {
                content.rejected += 1;
                self.content_applications.insert(&content_cid, content)?;
            }
            _ => return Err(StateError::InvalidReviewer),
        }
        match self.content_applications.get(&content_cid).await? {
            Some(content) => {
                let rejected_threshold = *self.reviewer_rejected_threshold.get();
                let reviewer_number = *self.reviewer_number.get();
                if content.rejected >= rejected_threshold || content.rejected >= reviewer_number {
                    return Ok(true);
                }
            }
            _ => todo!(),
        }
        Ok(false)
    }

    pub(crate) async fn validate_asset_review(
        &self,
        reviewer: Owner,
        collection_id: u64,
    ) -> Result<(), StateError> {
        if !self.is_reviewer(reviewer).await? {
            return Err(StateError::InvalidReviewer);
        }
        match self.asset_applications.get(&collection_id).await? {
            Some(asset) => match asset.reviewers.get(&reviewer) {
                Some(_) => Err(StateError::AlreadyReviewed),
                _ => Ok(()),
            },
            None => Ok(()),
        }
    }

    pub(crate) async fn approve_asset(
        &mut self,
        reviewer: Owner,
        collection_id: u64,
    ) -> Result<bool, StateError> {
        self.validate_asset_review(reviewer, collection_id).await?;
        match self.asset_applications.get(&collection_id).await? {
            Some(mut asset) => {
                asset.approved += 1;
                self.asset_applications.insert(&collection_id, asset)?;
            }
            _ => return Err(StateError::InvalidReviewer),
        }
        match self.asset_applications.get(&collection_id).await? {
            Some(asset) => {
                let approved_threshold = *self.reviewer_approved_threshold.get();
                let reviewer_number = *self.reviewer_number.get();
                if asset.approved >= approved_threshold || asset.approved >= reviewer_number {
                    return Ok(true);
                }
            }
            _ => todo!(),
        }
        Ok(false)
    }

    pub(crate) async fn reject_asset(
        &mut self,
        reviewer: Owner,
        collection_id: u64,
    ) -> Result<bool, StateError> {
        self.validate_asset_review(reviewer, collection_id).await?;
        match self.asset_applications.get(&collection_id).await? {
            Some(mut asset) => {
                asset.rejected += 1;
                self.asset_applications.insert(&collection_id, asset)?;
            }
            _ => return Err(StateError::InvalidReviewer),
        }
        match self.asset_applications.get(&collection_id).await? {
            Some(asset) => {
                let rejected_threshold = *self.reviewer_rejected_threshold.get();
                let reviewer_number = *self.reviewer_number.get();
                if asset.rejected >= rejected_threshold || asset.rejected >= reviewer_number {
                    return Ok(true);
                }
            }
            _ => todo!(),
        }
        Ok(false)
    }
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("View error")]
    ViewError(#[from] linera_views::views::ViewError),

    #[error("Arithmetic error")]
    ArithmeticError(#[from] ArithmeticError),

    #[error("Invalid reviewer")]
    InvalidReviewer,

    #[error("Already reviewer")]
    AlreadyReviewed,

    #[error("Invalid content")]
    InvalidContent,
}
