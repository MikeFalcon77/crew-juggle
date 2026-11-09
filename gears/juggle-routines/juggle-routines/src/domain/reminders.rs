use super::{self as domain, Delivery, Result, Service};
use crate::domain::PushTransport;
use chrono::Utc;
use juggle_crews_sdk::worker_context;
use juggle_routines_sdk::{DeliveryOutcome, Notification, NotificationKind};
use std::{collections::BTreeMap, time::Duration};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// A member hears about their own tasks and about tasks of people who cannot
/// sign in (somebody has to feed the cat).
fn concerns(state: &domain::State, uid: &str, person_id: &str) -> bool {
    state
        .people
        .iter()
        .find(|p| p.id == person_id)
        .is_some_and(|p| p.account.is_none() || p.account.as_deref() == Some(uid))
}
fn pending_count(state: &domain::State, delivery: &Delivery) -> usize {
    state.days.get(&delivery.day).map_or(0, |items| {
        items
            .iter()
            .filter(|i| {
                i.status == "pending"
                    && i.routine.person_id == delivery.person_id
                    && i.routine.time == delivery.time
            })
            .count()
    })
}
/// Materializes due reminder groups without duplicating existing deliveries.
pub(crate) fn prepare_reminders(state: &mut domain::State, now: chrono::DateTime<Utc>) {
    let date = state.today(now);
    let current_time = now.with_timezone(&state.zone).format("%H:%M").to_string();
    let mut groups = BTreeMap::new();
    for item in state
        .days
        .get(&date)
        .into_iter()
        .flatten()
        .filter(|i| i.status == "pending" && i.routine.time <= current_time)
    {
        groups.insert(
            (item.routine.person_id.clone(), item.routine.time.clone()),
            item.routine.assignee.clone(),
        );
    }
    let mut fresh = vec![];
    for sub in &state.subscriptions {
        for ((person_id, time), assignee) in &groups {
            if !concerns(state, &sub.uid, person_id) {
                continue;
            }
            let key = format!("{date}/{person_id}/{time}/{}", sub.id);
            if !state.deliveries.iter().any(|d| d.key == key) {
                fresh.push(Delivery {
                    key,
                    subscription_id: sub.id.clone(),
                    day: date,
                    assignee: assignee.clone(),
                    person_id: person_id.clone(),
                    time: time.clone(),
                    attempts: 0,
                    status: "pending".into(),
                    retry_at: now,
                    tag: format!("routines/{date}"),
                });
            }
        }
    }
    state.deliveries.extend(fresh);
}
/// Leases eligible groups for one device and fences retries until their deadline.
pub(crate) fn claim_batch(
    state: &mut domain::State,
    subscription_id: &str,
    now: chrono::DateTime<Utc>,
) -> Vec<Delivery> {
    let date = state.today(now);
    for index in 0..state.deliveries.len() {
        let d = &state.deliveries[index];
        if d.subscription_id != subscription_id
            || !["pending", "sending"].contains(&d.status.as_str())
        {
            continue;
        }
        if d.day != date || pending_count(state, d) == 0 {
            state.deliveries[index].status = "cancelled".into();
        } else if d.attempts >= 3 && d.retry_at <= now {
            state.deliveries[index].status = "failed".into();
        }
    }
    // A live send lease prevents simultaneous sends for the same device.
    if state
        .deliveries
        .iter()
        .any(|d| d.subscription_id == subscription_id && d.status == "sending" && d.retry_at > now)
    {
        return vec![];
    }
    let ready = state.deliveries.iter().any(|d| {
        d.subscription_id == subscription_id
            && ["pending", "sending"].contains(&d.status.as_str())
            && d.retry_at <= now
            && d.attempts < 3
    });
    if !ready {
        return vec![];
    }
    state
        .deliveries
        .iter_mut()
        .filter(|d| {
            d.subscription_id == subscription_id
                && ["pending", "sending"].contains(&d.status.as_str())
                && d.attempts < 3
        })
        .map(|d| {
            d.attempts += 1;
            d.status = "sending".into();
            d.retry_at = now + chrono::Duration::minutes(2);
            d.tag = format!("routines/{date}");
            d.clone()
        })
        .collect()
}
/// One-line notification text for the tasks due in one slot.
pub(crate) fn summary(
    state: &domain::State,
    now: chrono::DateTime<Utc>,
    uid: &str,
    viewer_zone: &str,
) -> Result<Option<String>> {
    let date = state.today(now);
    let current_time = now.with_timezone(&state.zone).format("%H:%M").to_string();
    let mut groups: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for item in state.days.get(&date).into_iter().flatten().filter(|i| {
        i.status == "pending"
            && i.routine.time <= current_time
            && concerns(state, uid, &i.routine.person_id)
    }) {
        let name = state
            .people
            .iter()
            .find(|p| p.id == item.routine.person_id)
            .map(|p| &p.name)
            .unwrap_or(&item.routine.assignee);
        groups
            .entry((name.clone(), item.routine.time.clone()))
            .or_default()
            .push(
                item.routine
                    .name
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            );
    }
    if groups.is_empty() {
        return Ok(None);
    }
    let mut lines = vec![];
    for ((name, time), items) in groups {
        lines.push(format!(
            "{name} · {}: {}",
            domain::time_label(date, &time, state.zone, viewer_zone)?,
            items.join(", ")
        ));
    }
    let body = lines.join("\n");
    if body.chars().count() > 700 {
        Ok(Some(format!(
            "{}…\nOpen the app for the full list.",
            body.chars().take(650).collect::<String>()
        )))
    } else {
        Ok(Some(body))
    }
}

/// One reminder pass over every crew. A failing crew does not stop the others.
pub async fn tick(
    service: &Service,
    push: &dyn PushTransport,
    cancel: &CancellationToken,
) -> Result<()> {
    let crews = service
        .crews
        .crew_ids(&worker_context(Uuid::nil()))
        .await
        .map_err(|e| domain::Error::Storage(anyhow::anyhow!("{e}")))?;
    for crew in crews {
        if cancel.is_cancelled() {
            break;
        }
        if let Err(error) = tick_crew(service, push, cancel, crew).await {
            tracing::warn!(%crew, %error, "reminders failed for one crew");
        }
    }
    Ok(())
}

async fn tick_crew(
    service: &Service,
    push: &dyn PushTransport,
    cancel: &CancellationToken,
    crew: Uuid,
) -> Result<()> {
    let ctx = worker_context(crew);
    let now = Utc::now();
    let day = service.day(&ctx, None, now).await?;
    let date = day.date;
    let subscriptions = service
        .mutate(&ctx, |state| {
            prepare_reminders(state, now);
            Ok(state.subscriptions.clone())
        })
        .await?;
    // At most one combined send per device per pass.
    for subscription in subscriptions {
        if cancel.is_cancelled() {
            break;
        }
        let deliveries = service
            .mutate(&ctx, |state| {
                Ok(claim_batch(state, &subscription.id, Utc::now()))
            })
            .await?;
        if deliveries.is_empty() {
            continue;
        }
        let (_, fresh) = service.read(&ctx).await?;
        let body = if fresh.subscriptions.iter().any(|s| s.id == subscription.id)
            && deliveries.iter().any(|d| pending_count(&fresh, d) > 0)
            && fresh.today(Utc::now()) == date
        {
            summary(
                &fresh,
                Utc::now(),
                &subscription.uid,
                &fresh.preferences_for(&subscription.uid).timezone,
            )?
        } else {
            None
        };
        let Some(body) = body else {
            service
                .mutate(&ctx, |state| {
                    for claimed in &deliveries {
                        if let Some(d) = state.deliveries.iter_mut().find(|d| {
                            d.key == claimed.key
                                && d.attempts == claimed.attempts
                                && d.status == "sending"
                        }) {
                            d.status = "cancelled".into();
                        }
                    }
                    Ok(())
                })
                .await?;
            continue;
        };
        let message = Notification {
            title: "crew-juggle · due now".into(),
            body,
            tag: format!("routines/{date}"),
            url: format!("/today?date={date}"),
            kind: NotificationKind::Routines,
            ttl_seconds: 300,
        };
        let send = push.send(&subscription, &message);
        let result = tokio::select! {_=cancel.cancelled()=>break,result=tokio::time::timeout(Duration::from_secs(25),send)=>result};
        let sent = matches!(result, Ok(Ok(DeliveryOutcome::Sent)));
        let gone = matches!(&result, Ok(Ok(DeliveryOutcome::SubscriptionGone)));
        service
            .mutate(&ctx, |state| {
                if gone {
                    state.subscriptions.retain(|s| s.id != subscription.id);
                }
                for claimed in &deliveries {
                    if let Some(d) = state.deliveries.iter_mut().find(|d| {
                        d.key == claimed.key
                            && d.attempts == claimed.attempts
                            && d.status == "sending"
                    }) {
                        d.status = if sent {
                            "sent"
                        } else if gone || d.attempts >= 3 {
                            "failed"
                        } else {
                            "pending"
                        }
                        .into();
                        d.retry_at =
                            Utc::now() + chrono::Duration::seconds(60 * i64::from(d.attempts));
                    }
                }
                Ok(())
            })
            .await?;
        if !sent {
            tracing::warn!(expired = gone, "combined push send did not succeed");
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/reminders_tests.rs"]
mod regression_tests;
