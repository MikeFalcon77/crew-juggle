use super::{AvatarEdit, AvatarMeta, Command, CommandResponse, Error, Operation, Result, State};

#[toolkit_macros::domain_model]
/// Accepted avatar metadata and whether the operation was replayed.
pub struct AvatarDecision {
    /// Current photo revision and presence returned to the caller.
    pub metadata: AvatarMeta,
    /// True when the stored result of this operation was returned unchanged.
    pub replayed: bool,
}

impl State {
    /// The transaction adapter supplies the current image revision and a content digest.
    /// Membership, replay identity and revision rules stay in the domain.
    pub fn apply_avatar_change(
        &mut self,
        id: &str,
        actor: &str,
        request: &AvatarEdit,
        digest: &str,
        current_revision: u64,
        has_photo: bool,
    ) -> Result<AvatarDecision> {
        let fingerprint = format!("avatar/{id}/{}/{digest}", request.expected_revision);
        let command = Command::Avatar(fingerprint);
        if let Some(op) = self.operations.get(&request.operation_id) {
            if op.actor != actor || op.command != command {
                return Err(Error::Conflict);
            }
            let CommandResponse::Avatar(metadata) = &op.response else {
                return Err(Error::Conflict);
            };
            let metadata = metadata.clone();
            return Ok(AvatarDecision {
                metadata,
                replayed: true,
            });
        }
        if !self.people.iter().any(|p| p.id == id) {
            return Err(Error::NotFound);
        }
        if current_revision != request.expected_revision {
            return Err(Error::Conflict);
        }
        let revision = request
            .expected_revision
            .checked_add(1)
            .filter(|r| i64::try_from(*r).is_ok())
            .ok_or(Error::Conflict)?;
        let metadata = AvatarMeta {
            revision,
            has_photo,
        };
        self.operations.insert(
            request.operation_id,
            Operation {
                actor: actor.into(),
                command,
                response: CommandResponse::Avatar(metadata.clone()),
            },
        );
        Ok(AvatarDecision {
            metadata,
            replayed: false,
        })
    }
}
