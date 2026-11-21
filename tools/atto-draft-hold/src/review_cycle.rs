//! Read-only four-party ready-cycle policy evaluation.
//! This module never authorizes a merge and never accepts PR text as evidence.
//! All API data must be collected by an authenticated, trusted deployment.
//! Missing provider bindings, incomplete pagination or event provenance = HOLD.
use std::collections::{HashMap, HashSet};

const DAY: i64 = 86_400;
const HUMAN_ORESOFTWARE: u64 = 11_139_560;
const HUMAN_THE1MILLS: u64 = 2_148_448;
const GH_ACTIONS_APP: u64 = 15_368;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionKind {
    Ready,
    Draft,
}

#[derive(Clone, Debug)]
pub struct Transition {
    pub id: u64,
    pub at: String,
    pub kind: TransitionKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewState {
    Approved,
    Dismissed,
    ChangesRequested,
    Commented,
}

#[derive(Clone, Debug)]
pub struct HumanReview {
    pub id: u64,
    pub login: String,
    pub github_id: u64,
    pub state: ReviewState,
    pub submitted_at: String,
    pub head_sha: String,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Provider {
    OpenAi,
    Anthropic,
}

#[derive(Clone, Debug)]
pub struct AiCheck {
    pub run_id: u64,
    pub provider: Provider,
    pub app_id: u64,
    pub name: String,
    pub head_sha: String,
    pub completed_at: String,
    pub successful: bool,
}

#[derive(Clone, Debug)]
pub struct TrustedAiApps {
    pub openai_app_id: u64,
    pub anthropic_app_id: u64,
    // Must mean independent verification of app ownership and exact
    // review-attestation semantics by the trusted deployment operator,
    // never a boolean parsed from a PR-controlled JSON or workflow input.
    pub provider_bindings_verified: bool,
}

#[derive(Clone, Debug)]
pub struct Evidence {
    pub head_sha: String,
    pub open: bool,
    pub draft: bool,
    pub created_at: String,
    pub created_as_draft: bool,
    pub lifecycle_verified: bool,
    pub transitions_complete: bool,
    pub transitions: Vec<Transition>,
    pub checked_at: String,
    pub last_push_at: String,
    pub push_verified: bool,
    pub push_head_sha: String,
    pub author_github_id: u64,
    pub labels_verified: bool,
    pub labels: Vec<String>,
    pub reviews_complete: bool,
    pub human_reviews: Vec<HumanReview>,
    pub ai_checks_complete: bool,
    pub ai_checks: Vec<AiCheck>,
    pub ai_bindings: Option<TrustedAiApps>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    InitialFour,
    FastHumanAndAi,
    RenewedFour,
    Draft,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub advisory_only: bool,
    pub merge_authorized: bool,
    pub lane: Lane,
    pub status: &'static str,
    pub ready_cycle_started_at: Option<i64>,
    pub failures: Vec<&'static str>,
}

fn valid_sha(value: &str) -> bool {
    value.len() == 40 &&
        value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// Strict GitHub UTC timestamps, no locale or arbitrary offsets.
// No external date parser or dependency is needed.
fn digits(s: &str) -> Option<i64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) { return None; }
    s.parse::<i64>().ok()
}
pub fn utc_seconds(s: &str) -> Option<i64> {
    if s.len() != 20 || &s[4..5] != "-" || &s[7..8] != "-" ||
       &s[10..11] != "T" || &s[13..14] != ":" || &s[16..17] != ":" ||
       &s[19..20] != "Z"
    { return None; }
    let y = digits(&s[0..4])?;
    let m = digits(&s[5..7])?;
    let d = digits(&s[8..10])?;
    let hh = digits(&s[11..13])?;
    let mm = digits(&s[14..16])?;
    let ss = digits(&s[17..19])?;
    if !(1970..=9999).contains(&y) || !(1..=12).contains(&m) ||
       hh > 23 || mm > 59 || ss > 59 { return None; }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let dim = [31, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if d < 1 || d > dim[(m - 1) as usize] { return None; }

    // Days since 1970-01-01 (Hinnant's Gregorian conversion).
    let year = y - if m <= 2 {1} else {0};
    let era = year / 400;
    let yoe = year - era * 400;
    let mp = m + if m > 2 {-3} else {9};
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146_097 + doe - 719_468) * DAY + hh * 3600 + mm * 60 + ss)
}

fn lifecycle(e: &Evidence, now: i64, failures: &mut Vec<&'static str>) -> Option<i64> {
    if !e.lifecycle_verified || !e.transitions_complete {
        failures.push("unverified or paginated ready-cycle event history");
        return None;
    }
    let created = match utc_seconds(&e.created_at) {
        Some(v) if v <= now => v,
        _ => { failures.push("invalid PR creation timestamp"); return None; }
    };
    let mut draft = e.created_as_draft;
    let mut start = if draft {None} else {Some(created)};
    let mut prior = created;
    let mut ids = HashSet::new();
    for event in &e.transitions {
        let at = match utc_seconds(&event.at) {
            Some(v) if v >= prior && v <= now => v,
            _ => { failures.push("lifecycle event timestamp missing, reordered, or future"); return None; }
        };
        if event.id == 0 || !ids.insert(event.id) {
            failures.push("missing or duplicate lifecycle event identity");
            return None;
        }
        match event.kind {
            TransitionKind::Ready if draft => { draft = false; start = Some(at); }
            TransitionKind::Draft if !draft => { draft = true; start = None; }
            _ => { failures.push("non-alternating Draft/Ready lifecycle events"); return None; }
        }
        prior = at;
    }
    if draft != e.draft {
        failures.push("live Draft flag disagrees with authenticated lifecycle");
        return None;
    }
    start
}

fn human_role(r: &HumanReview) -> Option<u64> {
    match r.login.to_ascii_lowercase().as_str() {
        "oresoftware" if r.github_id == HUMAN_ORESOFTWARE => Some(HUMAN_ORESOFTWARE),
        "the1mills" if r.github_id == HUMAN_THE1MILLS => Some(HUMAN_THE1MILLS),
        _ => None,
    }
}
fn app_id_for(bind: &TrustedAiApps, p: Provider) -> u64 {
    match p { Provider::OpenAi => bind.openai_app_id, Provider::Anthropic => bind.anthropic_app_id }
}
fn context_for(p: Provider) -> &'static str {
    match p {
        Provider::OpenAi => "atto-ai/openai/review-attested",
        Provider::Anthropic => "atto-ai/anthropic/review-attested",
    }
}

pub fn assess(e: &Evidence) -> Report {
    let mut failures = Vec::new();
    let now = utc_seconds(&e.checked_at);
    if now.is_none() { failures.push("invalid trusted evaluation time"); }
    if !valid_sha(&e.head_sha) { failures.push("invalid immutable head SHA"); }
    if !e.open || e.draft { failures.push("PR not open and Ready"); }
    if !e.labels_verified { failures.push("live labels unverified"); }
    if e.labels.iter().any(|x| matches!(x.to_ascii_lowercase().as_str(),
        "keep-as-draft" | "keep-draft" | "not-ready"))
    { failures.push("blocking hold label present"); }
    if !e.push_verified || e.push_head_sha != e.head_sha {
        failures.push("authenticated last-push event missing or bound to another head");
    }
    let pushed = utc_seconds(&e.last_push_at);
    if pushed.is_none() || pushed > now { failures.push("last-push timestamp invalid"); }
    if e.author_github_id == 0 { failures.push("immutable author GitHub identity missing"); }
    if !e.reviews_complete { failures.push("GitHub review history incomplete"); }
    if !e.ai_checks_complete { failures.push("provider attestation check history incomplete"); }
    let binding = e.ai_bindings.as_ref().filter(|b|
        b.provider_bindings_verified && b.openai_app_id > 0 && b.anthropic_app_id > 0 &&
        b.openai_app_id != b.anthropic_app_id &&
        b.openai_app_id != GH_ACTIONS_APP && b.anthropic_app_id != GH_ACTIONS_APP);
    if binding.is_none() { failures.push("two independently audited AI issuer identities absent"); }
    let start = now.and_then(|n| lifecycle(e, n, &mut failures));
    let lane = if e.draft { Lane::Draft } else if let (Some(s),Some(n)) = (start,now) {
        if n < s { failures.push("Ready clock is in the future"); Lane::Unknown }
        else if n-s < 3*DAY { Lane::InitialFour }
        else if n-s < 10*DAY { Lane::FastHumanAndAi }
        else { Lane::RenewedFour }
    } else { Lane::Unknown };
    let fresh_from = if lane == Lane::RenewedFour { start.map(|s| s+10*DAY) }
                     else { start };
    let mut approved_humans = HashSet::new();
    if e.reviews_complete {
        let mut latest: HashMap<u64, (&HumanReview,i64)> = HashMap::new();
        let mut review_ids = HashSet::new();
        for r in &e.human_reviews {
            let at = utc_seconds(&r.submitted_at);
            if r.id == 0 || !review_ids.insert(r.id) || at.is_none() || at > now {
                failures.push("malformed or duplicate authenticated GitHub review"); continue;
            }
            let at = at.unwrap();
            if r.github_id == 0 { failures.push("GitHub reviewer numeric identity missing"); continue; }
            match latest.get(&r.github_id) {
                Some((_, previous_at)) if *previous_at >= at =>
                    failures.push("ambiguous or out-of-order GitHub review event"),
                _ => { latest.insert(r.github_id,(r,at)); }
            }
        }
        for (r,at) in latest.values() {
            if r.state == ReviewState::ChangesRequested {
                failures.push("unresolved GitHub changes requested");
            }
            if r.state != ReviewState::Approved { continue; }
            if r.github_id == e.author_github_id { continue; }
            if let (Some(role),Some(boundary),Some(push)) = (human_role(r),fresh_from,pushed) {
                if *at >= boundary && *at > push && r.head_sha == e.head_sha {
                    approved_humans.insert(role);
                }
            }
        }
    }
    let mut approved_ai = HashSet::new();
    if e.ai_checks_complete {
        let mut run_ids = HashSet::new();
        let mut per_provider: HashMap<Provider,(i64,bool)> = HashMap::new();
        for c in &e.ai_checks {
            let at = utc_seconds(&c.completed_at);
            if c.run_id == 0 || !run_ids.insert(c.run_id) || at.is_none() || at > now {
                failures.push("malformed or duplicate AI check-run evidence"); continue;
            }
            let at = at.unwrap();
            let verified_issuer = binding.is_some_and(|b|
                app_id_for(b,c.provider) == c.app_id && c.name == context_for(c.provider));
            if !verified_issuer { continue; }
            let is_current = c.successful && c.head_sha == e.head_sha
                && fresh_from.is_some_and(|boundary| at >= boundary)
                && pushed.is_some_and(|push| at > push);
            match per_provider.get(&c.provider) {
                Some((previous,_)) if *previous >= at =>
                    failures.push("ambiguous or out-of-order AI attestation"),
                _ => { per_provider.insert(c.provider,(at,is_current)); }
            }
        }
        for (provider,(_,good)) in per_provider {
            if good { approved_ai.insert(provider); }
        }
    }
    let enough = match lane {
        Lane::InitialFour | Lane::RenewedFour =>
            approved_humans.contains(&HUMAN_ORESOFTWARE) &&
            approved_humans.contains(&HUMAN_THE1MILLS) &&
            approved_ai.contains(&Provider::OpenAi) &&
            approved_ai.contains(&Provider::Anthropic),
        Lane::FastHumanAndAi => !approved_humans.is_empty() && !approved_ai.is_empty(),
        _ => false,
    };
    if !enough { failures.push("ready-cycle named human and AI threshold not satisfied"); }
    Report {
        advisory_only: true, merge_authorized: false, lane,
        status: if failures.is_empty() { "REPORTED_SATISFIED_NOT_AUTHORIZED" } else { "HOLD" },
        ready_cycle_started_at: start, failures,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OLD: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const CREATED: &str = "2026-11-01T00:00:00Z";
    fn human(id: u64, login: &str, at: &str) -> HumanReview {
        HumanReview { id, login:login.into(),
            github_id: if login.eq_ignore_ascii_case("oresoftware") {HUMAN_ORESOFTWARE} else {HUMAN_THE1MILLS},
            state:ReviewState::Approved, submitted_at:at.into(), head_sha:SHA.into() }
    }
    fn ai(run_id:u64, provider:Provider, at:&str) -> AiCheck {
        AiCheck { run_id, provider, app_id:if provider==Provider::OpenAi {100_001} else {100_002},
            name:context_for(provider).into(), head_sha:SHA.into(),
            completed_at:at.into(), successful:true }
    }
    fn base(now:&str) -> Evidence {
        let time = "2026-11-01T01:00:00Z";
        Evidence {head_sha:SHA.into(),open:true,draft:false,created_at:CREATED.into(),
            created_as_draft:false,lifecycle_verified:true,transitions_complete:true,
            transitions:vec![], checked_at:now.into(),
            last_push_at:"2026-11-01T00:15:00Z".into(),push_verified:true,push_head_sha:SHA.into(),
            author_github_id:9999,labels_verified:true,labels:vec![],reviews_complete:true,
            human_reviews:vec![human(1,"ORESoftware",time),human(2,"the1mills",time)],
            ai_checks_complete:true,
            ai_checks:vec![ai(3,Provider::OpenAi,time),ai(4,Provider::Anthropic,time)],
            ai_bindings:Some(TrustedAiApps {openai_app_id:100_001,anthropic_app_id:100_002,
                provider_bindings_verified:true}) }
    }
    fn satisfied(e:&Evidence)->bool {assess(e).status=="REPORTED_SATISFIED_NOT_AUTHORIZED"}
    #[test] fn utc_parser_enforces_calendar_and_zone() {
        assert_eq!(utc_seconds("1970-01-01T00:00:00Z"),Some(0));
        assert_eq!(utc_seconds("2026-11-01T00:00:00Z"),Some(1_793_491_200));
        for x in ["2026-02-29T00:00:00Z","2026-00-01T00:00:00Z",
            "2026-11-01T24:00:00Z","2026-11-01T00:00:60Z","2026-11-01T00:00:00+00:00",
            "yesterday","2026-11-01T00:00:00Z\n"] { assert_eq!(utc_seconds(x),None,"{x}"); }
        assert!(utc_seconds("2024-02-29T00:00:00Z").is_some());
    }
    #[test] fn first_72h_requires_both_humans_and_both_ais() {
        let mut e=base("2026-11-02T00:00:00Z");
        assert!(satisfied(&e));
        e.human_reviews.pop();assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.ai_checks.pop();assert!(!satisfied(&e));
    }
    #[test] fn exactly_72h_requires_one_human_and_one_ai() {
        let mut e=base("2026-11-04T00:00:00Z");
        e.human_reviews.truncate(1);e.ai_checks.truncate(1);
        assert_eq!(assess(&e).lane,Lane::FastHumanAndAi);
        assert!(satisfied(&e));
        e.ai_checks.clear();assert!(!satisfied(&e));
        e=base("2026-11-04T00:00:00Z");e.human_reviews.clear();
        assert!(!satisfied(&e));
    }
    #[test] fn at_240h_requires_post_boundary_four_party_approvals() {
        let mut e=base("2026-11-11T00:00:00Z");
        assert_eq!(assess(&e).lane,Lane::RenewedFour);
        assert!(!satisfied(&e));
        let at="2026-11-11T00:05:00Z";
        for r in &mut e.human_reviews {r.submitted_at=at.into();}
        for c in &mut e.ai_checks {c.completed_at=at.into();}
        assert!(satisfied(&e));
    }
    #[test] fn draft_ready_draft_ready_resets_clock_and_evidence() {
        let mut e=base("2026-11-12T00:00:00Z");
        e.created_as_draft=true;
        e.transitions=vec![
            Transition{id:1,at:"2026-11-02T00:00:00Z".into(),kind:TransitionKind::Ready},
            Transition{id:2,at:"2026-11-10T00:00:00Z".into(),kind:TransitionKind::Draft},
            Transition{id:3,at:"2026-11-11T23:00:00Z".into(),kind:TransitionKind::Ready},
        ];
        assert_eq!(assess(&e).lane,Lane::InitialFour);
        assert!(!satisfied(&e));
        for r in &mut e.human_reviews {r.submitted_at="2026-11-11T23:30:00Z".into();}
        for c in &mut e.ai_checks {c.completed_at="2026-11-11T23:30:00Z".into();}
        assert!(satisfied(&e));
    }
    #[test] fn inconsistent_lifecycle_fails_closed() {
        let mut e=base("2026-11-04T00:00:00Z");
        e.transitions.push(Transition{id:1,at:"2026-11-02T00:00:00Z".into(),kind:TransitionKind::Ready});
        assert!(!satisfied(&e));
        e.transitions[0].kind=TransitionKind::Draft;
        assert!(!satisfied(&e));
        e.transitions.push(Transition{id:1,at:"2026-11-03T00:00:00Z".into(),kind:TransitionKind::Ready});
        assert!(!satisfied(&e));
    }
    #[test] fn holds_or_draft_always_override_approvals() {
        for label in ["keep-as-draft","keep-draft","not-ready"] {
            let mut e=base("2026-11-02T00:00:00Z");
            e.labels=vec![label.to_ascii_uppercase()];
            assert!(!satisfied(&e));
        }
        let mut e=base("2026-11-02T00:00:00Z");e.draft=true;
        assert!(!satisfied(&e));
    }
    #[test] fn missing_trusted_source_and_pagination_fail_closed() {
        let mut e=base("2026-11-02T00:00:00Z");
        e.push_verified=false;assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.ai_bindings=None;assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.reviews_complete=false;assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.ai_checks_complete=false;assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.transitions_complete=false;assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.lifecycle_verified=false;assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.labels_verified=false;assert!(!satisfied(&e));
    }
    #[test] fn app_id_provider_spoofing_or_reused_app_fails() {
        let mut e=base("2026-11-02T00:00:00Z");e.ai_checks[0].app_id=GH_ACTIONS_APP;
        assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.ai_bindings.as_mut().unwrap().anthropic_app_id=100_001;
        assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.ai_checks[0].name="atto-ai/anthropic/review-attested".into();
        assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.ai_bindings.as_mut().unwrap().provider_bindings_verified=false;
        assert!(!satisfied(&e));
    }
    #[test] fn forged_human_login_id_and_self_review_fail() {
        let mut e=base("2026-11-02T00:00:00Z");e.human_reviews[0].github_id=HUMAN_THE1MILLS;
        assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.author_github_id=HUMAN_THE1MILLS;
        assert!(!satisfied(&e));
    }
    #[test] fn author_may_use_fast_lane_with_other_human_and_one_ai() {
        let mut e=base("2026-11-04T00:00:00Z");
        e.author_github_id=HUMAN_THE1MILLS;e.ai_checks.truncate(1);
        assert!(satisfied(&e));
    }
    #[test] fn stale_head_or_last_push_invalidates_approvals() {
        let mut e=base("2026-11-02T00:00:00Z");
        e.human_reviews[0].head_sha=OLD.into();assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.last_push_at="2026-11-01T01:30:00Z".into();
        assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");e.push_head_sha=OLD.into();assert!(!satisfied(&e));
    }
    #[test] fn last_review_event_wins_and_change_requests_hold() {
        let mut e=base("2026-11-02T00:00:00Z");
        e.human_reviews.push(HumanReview{id:5,state:ReviewState::ChangesRequested,
            submitted_at:"2026-11-01T02:00:00Z".into(),..e.human_reviews[0].clone()});
        assert!(!satisfied(&e));
        e.human_reviews.push(HumanReview{id:6,state:ReviewState::Approved,
            submitted_at:"2026-11-01T03:00:00Z".into(),..e.human_reviews[0].clone()});
        assert!(satisfied(&e));
    }
    #[test] fn invalid_timeline_event_or_duplicate_review_id_holds() {
        let mut e=base("2026-11-02T00:00:00Z");
        e.human_reviews[1].id=1;assert!(!satisfied(&e));
        e=base("2026-11-02T00:00:00Z");
        e.transitions=vec![Transition{id:0,at:"2026-11-02T00:00:00Z".into(),kind:TransitionKind::Draft}];
        assert!(!satisfied(&e));
    }
    #[test] fn never_yields_merge_authorized_even_when_all_approvals_present() {
        let e=base("2026-11-02T00:00:00Z");
        let r=assess(&e);assert!(!r.merge_authorized);assert!(r.advisory_only);
        assert_eq!(r.status,"REPORTED_SATISFIED_NOT_AUTHORIZED");
    }
}
