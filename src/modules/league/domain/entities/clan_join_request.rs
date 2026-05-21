use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
pub enum RequestStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestStatus::Pending => write!(f, "Pending"),
            RequestStatus::Approved => write!(f, "Approved"),
            RequestStatus::Rejected => write!(f, "Rejected"),
        }
    }
}

impl RequestStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Approved" => RequestStatus::Approved,
            "Rejected" => RequestStatus::Rejected,
            _ => RequestStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClanJoinRequest {
    id: Uuid,
    clan_id: Uuid,
    user_id: Uuid,
    status: RequestStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ClanJoinRequest {
    pub fn new(clan_id: Uuid, user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            clan_id,
            user_id,
            status: RequestStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(dead_code)]
    pub fn with_id(
        id: Uuid,
        clan_id: Uuid,
        user_id: Uuid,
        status: RequestStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            clan_id,
            user_id,
            status,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn clan_id(&self) -> Uuid {
        self.clan_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn status(&self) -> &RequestStatus {
        &self.status
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn approve(&mut self) {
        self.status = RequestStatus::Approved;
        self.updated_at = Utc::now();
    }

    pub fn reject(&mut self) {
        self.status = RequestStatus::Rejected;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_request_defaults_to_pending() {
        let req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(req.status(), &RequestStatus::Pending);
    }

    #[test]
    fn test_approve_changes_status() {
        let mut req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        req.approve();
        assert_eq!(req.status(), &RequestStatus::Approved);
    }

    #[test]
    fn test_reject_changes_status() {
        let mut req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        req.reject();
        assert_eq!(req.status(), &RequestStatus::Rejected);
    }

    #[test]
    fn test_request_status_display() {
        assert_eq!(RequestStatus::Pending.to_string(), "Pending");
        assert_eq!(RequestStatus::Approved.to_string(), "Approved");
        assert_eq!(RequestStatus::Rejected.to_string(), "Rejected");
    }

    #[test]
    fn test_request_status_from_str() {
        assert_eq!(RequestStatus::from_str("Pending"), RequestStatus::Pending);
        assert_eq!(RequestStatus::from_str("Approved"), RequestStatus::Approved);
        assert_eq!(RequestStatus::from_str("Rejected"), RequestStatus::Rejected);
        assert_eq!(RequestStatus::from_str("Unknown"), RequestStatus::Pending);
    }

    #[test]
    fn test_approve_updates_timestamp() {
        let before = Utc::now();
        let mut req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        let original_updated = req.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));
        req.approve();
        assert!(req.updated_at() > original_updated);
        assert!(req.updated_at() >= before);
    }

    // Approving an already-approved request is idempotent on the entity level
    #[test]
    fn test_double_approve() {
        let mut req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        req.approve();
        assert_eq!(req.status(), &RequestStatus::Approved);
        req.approve();
        assert_eq!(req.status(), &RequestStatus::Approved);
    }

    // Rejecting an already-rejected request is idempotent
    #[test]
    fn test_double_reject() {
        let mut req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        req.reject();
        assert_eq!(req.status(), &RequestStatus::Rejected);
        req.reject();
        assert_eq!(req.status(), &RequestStatus::Rejected);
    }

    // Approve after reject: entity doesn't prevent this (use case does)
    #[test]
    fn test_approve_after_reject() {
        let mut req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        req.reject();
        req.approve();
        assert_eq!(req.status(), &RequestStatus::Approved);
    }

    // with_id round-trip: all fields must match
    #[test]
    fn test_with_id_roundtrip() {
        let id = Uuid::new_v4();
        let clan_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let created_at = Utc::now();
        let updated_at = Utc::now();

        let req = ClanJoinRequest::with_id(
            id, clan_id, user_id, RequestStatus::Approved, created_at, updated_at,
        );
        assert_eq!(req.id(), id);
        assert_eq!(req.clan_id(), clan_id);
        assert_eq!(req.user_id(), user_id);
        assert_eq!(req.status(), &RequestStatus::Approved);
        assert_eq!(req.created_at(), created_at);
        assert_eq!(req.updated_at(), updated_at);
    }

    // new() generates a unique ID each time
    #[test]
    fn test_new_generates_unique_id() {
        let req1 = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        let req2 = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        assert_ne!(req1.id(), req2.id());
    }

    // Default status is Pending
    #[test]
    fn test_default_status_is_pending() {
        let req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(req.status(), &RequestStatus::Pending);
    }

    // From_str with empty string falls back to Pending
    #[test]
    fn test_from_str_empty_falls_back_to_pending() {
        assert_eq!(RequestStatus::from_str(""), RequestStatus::Pending);
    }

    // reject updates timestamp too
    #[test]
    fn test_reject_updates_timestamp() {
        let mut req = ClanJoinRequest::new(Uuid::new_v4(), Uuid::new_v4());
        let original = req.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));
        req.reject();
        assert!(req.updated_at() > original);
    }
}
