#![no_std]
#![forbid(unsafe_code)]

//! Persistent user policy is data, never a kernel capability. This codec does
//! not authenticate bytes; callers must trust the storage source before use.

pub const MAX_RULES: usize = 64;
const MAGIC: [u8; 8] = *b"NOSPOL01";
const VERSION: u16 = 1;
const HEADER_SIZE: usize = 16;
const RECORD_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppIdentity(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PurposeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivityId(pub u32);

/// Exact policy target. Conditions belong to the stored rule, not this identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub app: AppIdentity,
    pub resource: ResourceId,
    pub purpose: PurposeId,
    pub context: ContextId,
    pub activity: ActivityId,
}

impl Scope {
    fn validate(self) -> Result<(), Error> {
        if self.app.0 == 0
            || self.resource.0 == 0
            || self.purpose.0 == 0
            || self.context.0 == 0
            || self.activity.0 == 0
        {
            return Err(Error::InvalidScope);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredDecision {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    pub scope: Scope,
    /// Stored policy condition, never a request-selected matching key.
    pub foreground_only: bool,
    pub decision: StoredDecision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub scope: Scope,
    /// Supplied by a trusted activity/session broker, not by the requesting app.
    pub activity_active: bool,
    /// Supplied by the trusted session broker, not by the requesting app.
    pub foreground: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyResult {
    NoRule,
    Denied,
    Inactive,
    /// A broker may consider issuing a capability; no runtime authority is issued.
    PermitsActivation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidScope,
    NoSpace,
    BufferTooSmall,
    InvalidEncoding,
    DuplicateRule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserPolicy {
    rules: [Option<Rule>; MAX_RULES],
    len: usize,
}

impl UserPolicy {
    pub const fn new() -> Self {
        Self {
            rules: [None; MAX_RULES],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Replaces the same target, including its decision and foreground condition.
    /// No wildcard matching exists.
    pub fn set(&mut self, rule: Rule) -> Result<(), Error> {
        rule.scope.validate()?;
        for existing in self.rules.iter_mut().flatten() {
            if existing.scope == rule.scope {
                *existing = rule;
                return Ok(());
            }
        }
        let slot = self
            .rules
            .iter_mut()
            .find(|entry| entry.is_none())
            .ok_or(Error::NoSpace)?;
        *slot = Some(rule);
        self.len += 1;
        Ok(())
    }

    /// Evaluates policy only. The result cannot be converted into a runtime
    /// capability; a separate broker must issue and later revoke one.
    pub fn evaluate(&self, request: Request) -> PolicyResult {
        if request.scope.validate().is_err() {
            return PolicyResult::NoRule;
        }
        let Some(rule) = self
            .rules
            .iter()
            .flatten()
            .find(|rule| rule.scope == request.scope)
        else {
            return PolicyResult::NoRule;
        };
        match rule.decision {
            StoredDecision::Deny => PolicyResult::Denied,
            StoredDecision::Allow
                if request.activity_active && (!rule.foreground_only || request.foreground) =>
            {
                PolicyResult::PermitsActivation
            }
            StoredDecision::Allow => PolicyResult::Inactive,
        }
    }

    pub fn encode(&self, output: &mut [u8]) -> Result<usize, Error> {
        let length = HEADER_SIZE
            .checked_add(self.len.checked_mul(RECORD_SIZE).ok_or(Error::NoSpace)?)
            .ok_or(Error::NoSpace)?;
        if output.len() < length {
            return Err(Error::BufferTooSmall);
        }
        output[..length].fill(0);
        output[..8].copy_from_slice(&MAGIC);
        output[8..10].copy_from_slice(&VERSION.to_le_bytes());
        output[10..12].copy_from_slice(&(self.len as u16).to_le_bytes());

        for (index, rule) in self.rules.iter().flatten().enumerate() {
            let start = HEADER_SIZE + index * RECORD_SIZE;
            let record = &mut output[start..start + RECORD_SIZE];
            record[..8].copy_from_slice(&rule.scope.app.0.to_le_bytes());
            record[8..12].copy_from_slice(&rule.scope.resource.0.to_le_bytes());
            record[12..16].copy_from_slice(&rule.scope.purpose.0.to_le_bytes());
            record[16..20].copy_from_slice(&rule.scope.context.0.to_le_bytes());
            record[20..24].copy_from_slice(&rule.scope.activity.0.to_le_bytes());
            record[24..28].copy_from_slice(&u32::from(rule.foreground_only).to_le_bytes());
            record[28] = match rule.decision {
                StoredDecision::Allow => 1,
                StoredDecision::Deny => 2,
            };
        }
        Ok(length)
    }

    pub fn decode(input: &[u8]) -> Result<Self, Error> {
        if input.len() < HEADER_SIZE
            || input[..8] != MAGIC
            || u16::from_le_bytes([input[8], input[9]]) != VERSION
            || input[12..16] != [0; 4]
        {
            return Err(Error::InvalidEncoding);
        }
        let count = usize::from(u16::from_le_bytes([input[10], input[11]]));
        if count > MAX_RULES
            || HEADER_SIZE.checked_add(
                count
                    .checked_mul(RECORD_SIZE)
                    .ok_or(Error::InvalidEncoding)?,
            ) != Some(input.len())
        {
            return Err(Error::InvalidEncoding);
        }

        let mut policy = Self::new();
        for index in 0..count {
            let start = HEADER_SIZE + index * RECORD_SIZE;
            let record = &input[start..start + RECORD_SIZE];
            if record[29..32] != [0; 3] {
                return Err(Error::InvalidEncoding);
            }
            let flags = u32::from_le_bytes(record[24..28].try_into().unwrap());
            if flags & !1 != 0 {
                return Err(Error::InvalidEncoding);
            }
            let scope = Scope {
                app: AppIdentity(u64::from_le_bytes(record[..8].try_into().unwrap())),
                resource: ResourceId(u32::from_le_bytes(record[8..12].try_into().unwrap())),
                purpose: PurposeId(u32::from_le_bytes(record[12..16].try_into().unwrap())),
                context: ContextId(u32::from_le_bytes(record[16..20].try_into().unwrap())),
                activity: ActivityId(u32::from_le_bytes(record[20..24].try_into().unwrap())),
            };
            scope.validate()?;
            if policy
                .rules
                .iter()
                .flatten()
                .any(|existing| existing.scope == scope)
            {
                return Err(Error::DuplicateRule);
            }
            let decision = match record[28] {
                1 => StoredDecision::Allow,
                2 => StoredDecision::Deny,
                _ => return Err(Error::InvalidEncoding),
            };
            policy.set(Rule {
                scope,
                foreground_only: flags == 1,
                decision,
            })?;
        }
        Ok(policy)
    }
}

impl Default for UserPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> Scope {
        Scope {
            app: AppIdentity(7),
            resource: ResourceId(2),
            purpose: PurposeId(3),
            context: ContextId(4),
            activity: ActivityId(5),
        }
    }

    fn request(scope: Scope) -> Request {
        Request {
            scope,
            activity_active: true,
            foreground: true,
        }
    }

    #[test]
    fn policy_match_is_exact_and_only_permits_active_foreground_work() {
        let mut policy = UserPolicy::new();
        policy
            .set(Rule {
                scope: scope(),
                foreground_only: true,
                decision: StoredDecision::Allow,
            })
            .unwrap();
        assert_eq!(
            policy.evaluate(request(scope())),
            PolicyResult::PermitsActivation
        );

        let mut inactive = request(scope());
        inactive.activity_active = false;
        assert_eq!(policy.evaluate(inactive), PolicyResult::Inactive);

        let mut background = request(scope());
        background.foreground = false;
        assert_eq!(policy.evaluate(background), PolicyResult::Inactive);

        let mut other_app = scope();
        other_app.app = AppIdentity(8);
        assert_eq!(policy.evaluate(request(other_app)), PolicyResult::NoRule);
    }

    #[test]
    fn deny_rule_and_missing_policy_never_permit_activation() {
        let mut policy = UserPolicy::new();
        assert_eq!(policy.evaluate(request(scope())), PolicyResult::NoRule);
        policy
            .set(Rule {
                scope: scope(),
                foreground_only: true,
                decision: StoredDecision::Deny,
            })
            .unwrap();
        assert_eq!(policy.evaluate(request(scope())), PolicyResult::Denied);
    }

    #[test]
    fn exact_scope_policy_survives_canonical_encode_decode() {
        let mut policy = UserPolicy::new();
        policy
            .set(Rule {
                scope: scope(),
                foreground_only: true,
                decision: StoredDecision::Allow,
            })
            .unwrap();
        let mut encoded = [0; HEADER_SIZE + RECORD_SIZE];
        let length = policy.encode(&mut encoded).unwrap();
        let decoded = UserPolicy::decode(&encoded[..length]).unwrap();
        assert_eq!(decoded, policy);
        assert_eq!(
            decoded.evaluate(request(scope())),
            PolicyResult::PermitsActivation
        );
    }

    #[test]
    fn malformed_or_ambiguous_persisted_policy_is_rejected() {
        let mut policy = UserPolicy::new();
        policy
            .set(Rule {
                scope: scope(),
                foreground_only: true,
                decision: StoredDecision::Allow,
            })
            .unwrap();
        let mut encoded = [0; HEADER_SIZE + RECORD_SIZE];
        policy.encode(&mut encoded).unwrap();

        let mut invalid_flags = encoded;
        invalid_flags[HEADER_SIZE + 24] = 2;
        assert_eq!(
            UserPolicy::decode(&invalid_flags),
            Err(Error::InvalidEncoding)
        );

        let mut invalid_reserved = encoded;
        invalid_reserved[HEADER_SIZE + 31] = 1;
        assert_eq!(
            UserPolicy::decode(&invalid_reserved),
            Err(Error::InvalidEncoding)
        );

        let mut duplicate = [0; HEADER_SIZE + RECORD_SIZE * 2];
        let length = policy.encode(&mut duplicate).unwrap();
        duplicate[10..12].copy_from_slice(&2u16.to_le_bytes());
        duplicate[length..].copy_from_slice(&encoded[HEADER_SIZE..]);
        assert_eq!(UserPolicy::decode(&duplicate), Err(Error::DuplicateRule));
        assert_eq!(
            policy.encode(&mut [0; HEADER_SIZE - 1]),
            Err(Error::BufferTooSmall)
        );
    }

    #[test]
    fn invalid_scope_is_never_stored_or_permitted() {
        let mut policy = UserPolicy::new();
        let mut invalid = scope();
        invalid.resource = ResourceId(0);
        assert_eq!(
            policy.set(Rule {
                scope: invalid,
                foreground_only: true,
                decision: StoredDecision::Allow,
            }),
            Err(Error::InvalidScope)
        );
        assert_eq!(policy.evaluate(request(invalid)), PolicyResult::NoRule);
    }

    fn regression_rule(foreground_only: bool, decision: StoredDecision) -> Rule {
        Rule {
            scope: scope(),
            foreground_only,
            decision,
        }
    }

    fn regression_request(foreground: bool, activity_active: bool) -> Request {
        Request {
            scope: scope(),
            foreground,
            activity_active,
        }
    }

    #[test]
    fn regression_deny_replaces_foreground_allow_for_same_target() {
        let mut policy = UserPolicy::new();
        policy
            .set(regression_rule(true, StoredDecision::Allow))
            .unwrap();
        policy
            .set(regression_rule(false, StoredDecision::Deny))
            .unwrap();
        for foreground in [true, false] {
            for activity_active in [true, false] {
                assert_eq!(
                    policy.evaluate(Request {
                        scope: scope(),
                        foreground,
                        activity_active,
                    }),
                    PolicyResult::Denied
                );
            }
        }
        assert_eq!(policy.len(), 1);
    }

    #[test]
    fn regression_foreground_allow_replaces_deny_for_same_target() {
        let mut policy = UserPolicy::new();
        policy
            .set(regression_rule(false, StoredDecision::Deny))
            .unwrap();
        policy
            .set(regression_rule(true, StoredDecision::Allow))
            .unwrap();
        assert_eq!(
            policy.evaluate(regression_request(false, true)),
            PolicyResult::Inactive
        );
        assert_eq!(
            policy.evaluate(regression_request(true, true)),
            PolicyResult::PermitsActivation
        );
        assert_eq!(policy.len(), 1);
    }

    #[test]
    fn regression_stored_foreground_condition_and_activity_govern_activation() {
        for foreground_only in [false, true] {
            let mut policy = UserPolicy::new();
            policy
                .set(regression_rule(foreground_only, StoredDecision::Allow))
                .unwrap();
            for foreground in [false, true] {
                for activity_active in [false, true] {
                    let expected = if activity_active && (!foreground_only || foreground) {
                        PolicyResult::PermitsActivation
                    } else {
                        PolicyResult::Inactive
                    };
                    assert_eq!(
                        policy.evaluate(regression_request(foreground, activity_active)),
                        expected
                    );
                }
            }
        }
    }

    #[test]
    fn regression_codec_preserves_last_target_decision_and_condition() {
        for (first, last) in [
            (
                regression_rule(true, StoredDecision::Allow),
                regression_rule(false, StoredDecision::Deny),
            ),
            (
                regression_rule(false, StoredDecision::Deny),
                regression_rule(true, StoredDecision::Allow),
            ),
        ] {
            let mut policy = UserPolicy::new();
            policy.set(first).unwrap();
            policy.set(last).unwrap();
            let mut encoded = [0; HEADER_SIZE + RECORD_SIZE * 2];
            let length = policy.encode(&mut encoded).unwrap();
            let decoded = UserPolicy::decode(&encoded[..length]).unwrap();
            for foreground in [false, true] {
                for activity_active in [false, true] {
                    let expected = match last.decision {
                        StoredDecision::Deny => PolicyResult::Denied,
                        StoredDecision::Allow if foreground && activity_active => {
                            PolicyResult::PermitsActivation
                        }
                        StoredDecision::Allow => PolicyResult::Inactive,
                    };
                    assert_eq!(
                        decoded.evaluate(regression_request(foreground, activity_active)),
                        expected
                    );
                }
            }
            assert_eq!(decoded, policy);
            assert_eq!(decoded.len(), 1);
            assert_eq!(length, HEADER_SIZE + RECORD_SIZE);
            let mut canonical = [0; HEADER_SIZE + RECORD_SIZE * 2];
            assert_eq!(decoded.encode(&mut canonical), Ok(length));
            assert_eq!(canonical, encoded);
        }
    }

    #[test]
    fn regression_codec_rejects_duplicate_targets_with_conflicting_conditions() {
        for first_decision in [StoredDecision::Allow, StoredDecision::Deny] {
            for second_decision in [StoredDecision::Allow, StoredDecision::Deny] {
                for first_foreground_only in [false, true] {
                    let mut policy = UserPolicy::new();
                    policy
                        .set(regression_rule(first_foreground_only, first_decision))
                        .unwrap();
                    let mut encoded = [0; HEADER_SIZE + RECORD_SIZE * 2];
                    let length = policy.encode(&mut encoded).unwrap();
                    encoded[10..12].copy_from_slice(&2u16.to_le_bytes());
                    encoded.copy_within(HEADER_SIZE..length, length);
                    encoded[length + 24] = u8::from(!first_foreground_only);
                    encoded[length + 28] = match second_decision {
                        StoredDecision::Allow => 1,
                        StoredDecision::Deny => 2,
                    };
                    assert_eq!(UserPolicy::decode(&encoded), Err(Error::DuplicateRule));
                }
            }
        }
    }

    #[test]
    fn regression_each_target_field_is_part_of_identity() {
        let target = scope();
        let alternatives = [
            Scope {
                app: AppIdentity(8),
                ..target
            },
            Scope {
                resource: ResourceId(8),
                ..target
            },
            Scope {
                purpose: PurposeId(8),
                ..target
            },
            Scope {
                context: ContextId(8),
                ..target
            },
            Scope {
                activity: ActivityId(8),
                ..target
            },
        ];
        let mut policy = UserPolicy::new();
        policy
            .set(regression_rule(true, StoredDecision::Allow))
            .unwrap();
        for alternative in alternatives {
            assert_eq!(policy.evaluate(request(alternative)), PolicyResult::NoRule);
            policy
                .set(Rule {
                    scope: alternative,
                    foreground_only: false,
                    decision: StoredDecision::Deny,
                })
                .unwrap();
        }
        assert_eq!(policy.len(), alternatives.len() + 1);
        assert_eq!(
            policy.evaluate(request(target)),
            PolicyResult::PermitsActivation
        );
        for alternative in alternatives {
            assert_eq!(policy.evaluate(request(alternative)), PolicyResult::Denied);
        }
    }

    #[test]
    fn regression_same_target_replaces_even_when_policy_is_full() {
        let mut policy = UserPolicy::new();
        for app in 1..=MAX_RULES as u64 {
            policy
                .set(Rule {
                    scope: Scope {
                        app: AppIdentity(app),
                        ..scope()
                    },
                    foreground_only: true,
                    decision: StoredDecision::Allow,
                })
                .unwrap();
        }
        policy
            .set(regression_rule(false, StoredDecision::Deny))
            .unwrap();
        assert_eq!(policy.len(), MAX_RULES);
        assert_eq!(
            policy.evaluate(regression_request(true, true)),
            PolicyResult::Denied
        );
        assert_eq!(
            policy.set(Rule {
                scope: Scope {
                    app: AppIdentity(MAX_RULES as u64 + 1),
                    ..scope()
                },
                foreground_only: false,
                decision: StoredDecision::Allow,
            }),
            Err(Error::NoSpace)
        );
    }

    #[test]
    fn regression_version_one_wire_layout_is_unchanged() {
        for foreground_only in [false, true] {
            for decision in [StoredDecision::Allow, StoredDecision::Deny] {
                let mut expected = [0; 48];
                expected[..8].copy_from_slice(b"NOSPOL01");
                expected[8] = 1;
                expected[10] = 1;
                expected[16] = 7;
                expected[24] = 2;
                expected[28] = 3;
                expected[32] = 4;
                expected[36] = 5;
                expected[40] = u8::from(foreground_only);
                expected[44] = match decision {
                    StoredDecision::Allow => 1,
                    StoredDecision::Deny => 2,
                };
                let mut policy = UserPolicy::new();
                policy
                    .set(regression_rule(foreground_only, decision))
                    .unwrap();
                let mut encoded = [0; 48];
                assert_eq!(policy.encode(&mut encoded), Ok(expected.len()));
                assert_eq!(encoded, expected);
                assert_eq!(UserPolicy::decode(&expected), Ok(policy));
            }
        }
    }

    #[test]
    fn regression_meeting_microphone_policy_is_not_runtime_authority() {
        // These IDs stand for a meeting app, microphone, call purpose, room and call.
        // No driver, broker or capability engine is linked by this policy test.
        let mut policy = UserPolicy::new();
        policy
            .set(regression_rule(true, StoredDecision::Allow))
            .unwrap();
        let remembered_policy = policy;
        assert_eq!(
            policy.evaluate(regression_request(true, true)),
            PolicyResult::PermitsActivation
        );
        assert_eq!(
            policy.evaluate(regression_request(false, true)),
            PolicyResult::Inactive
        );
        assert_eq!(
            policy.evaluate(regression_request(true, false)),
            PolicyResult::Inactive
        );
        assert_eq!(policy, remembered_policy);
        policy
            .set(regression_rule(false, StoredDecision::Deny))
            .unwrap();
        assert_eq!(policy.evaluate(request(scope())), PolicyResult::Denied);
        // Denial affects future policy decisions; runtime revocation is the broker's job.
    }
}
