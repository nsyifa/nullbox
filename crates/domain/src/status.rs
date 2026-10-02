use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Lifecycle of a ticket. Funny names live in the frontend copy files only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TicketStatus {
    Open,
    InProgress,
    WaitingOnCustomer,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot move a ticket from {from:?} to {to:?}")]
pub struct InvalidTransition {
    pub from: TicketStatus,
    pub to: TicketStatus,
}

impl TicketStatus {
    pub const ALL: [TicketStatus; 5] =
        [Self::Open, Self::InProgress, Self::WaitingOnCustomer, Self::Resolved, Self::Closed];

    /// Statuses that count against Inbox Zero (and feed the Backlog Beast).
    pub const ACTIVE: [TicketStatus; 3] = [Self::Open, Self::InProgress, Self::WaitingOnCustomer];

    pub fn is_active(self) -> bool {
        Self::ACTIVE.contains(&self)
    }

    /// `closed` is the archive: nothing leaves the Shadow Realm.
    pub fn can_transition_to(self, to: TicketStatus) -> bool {
        use TicketStatus::*;
        match (self, to) {
            (Closed, _) => false,
            (from, to) if from == to => false,
            (Resolved, Open | Closed) => true,
            (Resolved, _) => false,
            _ => true,
        }
    }

    pub fn transition_to(self, to: TicketStatus) -> Result<TicketStatus, InvalidTransition> {
        if self.can_transition_to(to) { Ok(to) } else { Err(InvalidTransition { from: self, to }) }
    }

    /// Status after a customer replies. Replying to a resolved ticket reopens it ("IT'S ALIVE!").
    pub fn after_customer_reply(self) -> TicketStatus {
        match self {
            Self::Resolved | Self::WaitingOnCustomer => Self::Open,
            other => other,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::WaitingOnCustomer => "waiting_on_customer",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }
}

impl std::str::FromStr for TicketStatus {
    type Err = UnknownVariant;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|status| status.as_str() == s).ok_or_else(|| UnknownVariant(s.to_owned()))
    }
}

/// A stored string that doesn't match any enum variant.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown variant: {0}")]
pub struct UnknownVariant(pub String);

#[cfg(test)]
mod tests {
    use super::TicketStatus::*;
    use super::*;

    #[test]
    fn closed_tickets_stay_in_the_shadow_realm() {
        for to in TicketStatus::ALL {
            assert!(!Closed.can_transition_to(to), "closed -> {to:?} must be rejected");
        }
    }

    #[test]
    fn resolved_tickets_can_only_reopen_or_close() {
        assert!(Resolved.can_transition_to(Open));
        assert!(Resolved.can_transition_to(Closed));
        assert_eq!(Resolved.transition_to(InProgress), Err(InvalidTransition { from: Resolved, to: InProgress }));
    }

    #[test]
    fn customer_reply_resurrects_resolved_tickets() {
        assert_eq!(Resolved.after_customer_reply(), Open);
        assert_eq!(WaitingOnCustomer.after_customer_reply(), Open);
        assert_eq!(InProgress.after_customer_reply(), InProgress);
    }

    #[test]
    fn status_strings_round_trip() {
        for status in TicketStatus::ALL {
            assert_eq!(status.as_str().parse::<TicketStatus>(), Ok(status));
        }
    }
}
