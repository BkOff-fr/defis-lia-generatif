//! Dispatch des notifications d'alerte (webhook / email / log_only).
//!
//! Brief C29.4 §"Notifications" :
//!
//! - **webhook**  : `reqwest::Client::post(url).json(payload).timeout(5s)`.
//! - **email**    : SMTP via `lettre 0.11`. Configuration lue depuis la
//!   table `config` (clés `smtp_url` + `smtp_from`). Si absente → fallback
//!   automatique `log_only` (graceful degrade exigé par le brief).
//! - **log_only** : `tracing::warn!` structuré.
//!
//! Cette fonction est **async** et tourne en dehors du lock SQLite. L'appelant
//! met à jour `alert_triggers.notified_at` / `notify_error` après retour.

use std::time::Duration;

use chrono::Utc;
use lettre::{
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, AsyncSmtpTransport},
    AsyncTransport, Message, Tokio1Executor,
};
use serde::Serialize;
use tracing::warn;

use crate::alerts::checker::AlertEvent;
use crate::alerts::store::NotifyKind;

const WEBHOOK_TIMEOUT_SECS: u64 = 5;

/// Payload POSTé aux webhooks. Format documenté dans le brief.
#[derive(Debug, Serialize)]
pub struct WebhookPayload<'a> {
    pub threshold_id: &'a str,
    pub scope: &'a str,
    pub target_id: Option<&'a str>,
    pub period: &'a str,
    pub gco2eq_max: f64,
    pub observed_gco2eq: f64,
    pub period_start: String,
    pub period_end: String,
    pub triggered_at: String,
}

/// Résultat de l'envoi de notification.
#[derive(Debug)]
pub struct NotifyOutcome {
    /// `true` si la notification a réussi (peut être un fallback log_only).
    pub ok: bool,
    /// Message d'erreur si `ok = false`, ou trace ("fallback log_only", etc.).
    pub error: Option<String>,
}

/// Notifie un événement d'alerte. Ne panique jamais — toute erreur réseau /
/// SMTP est capturée dans `NotifyOutcome`.
pub async fn notify(event: &AlertEvent, smtp_config: Option<SmtpConfig>) -> NotifyOutcome {
    let kind = event.threshold.notify_kind;
    let target = event.threshold.notify_target.as_deref();

    let payload = WebhookPayload {
        threshold_id: &event.threshold.id,
        scope: event.threshold.scope.as_str(),
        target_id: event.threshold.target_id.as_deref(),
        period: event.threshold.period.as_str(),
        gco2eq_max: event.threshold.gco2eq_max,
        observed_gco2eq: event.observed_gco2eq,
        period_start: event.period_start.to_rfc3339(),
        period_end: event.period_end.to_rfc3339(),
        triggered_at: Utc::now().to_rfc3339(),
    };

    match kind {
        NotifyKind::Webhook => {
            let Some(url) = target else {
                return log_fallback(event, "webhook sans URL — fallback log_only");
            };
            send_webhook(url, &payload).await
        },
        NotifyKind::Email => {
            let Some(to) = target else {
                return log_fallback(event, "email sans destinataire — fallback log_only");
            };
            let Some(smtp) = smtp_config else {
                return log_fallback(
                    event,
                    "SMTP non configuré (smtp_url manquant) — fallback log_only",
                );
            };
            send_email(&smtp, to, event, &payload).await
        },
        NotifyKind::LogOnly => log_fallback(event, "log_only"),
    }
}

async fn send_webhook<P: Serialize + ?Sized>(url: &str, payload: &P) -> NotifyOutcome {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(WEBHOOK_TIMEOUT_SECS))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return NotifyOutcome {
                ok: false,
                error: Some(format!("client reqwest: {e}")),
            };
        },
    };
    match client.post(url).json(payload).send().await {
        Ok(resp) if resp.status().is_success() => NotifyOutcome {
            ok: true,
            error: None,
        },
        Ok(resp) => NotifyOutcome {
            ok: false,
            error: Some(format!("HTTP {}", resp.status())),
        },
        Err(e) => NotifyOutcome {
            ok: false,
            error: Some(format!("transport: {e}")),
        },
    }
}

async fn send_email(
    smtp: &SmtpConfig,
    to: &str,
    event: &AlertEvent,
    payload: &WebhookPayload<'_>,
) -> NotifyOutcome {
    let body = format!(
        "Seuil d'alerte Sobr.ia dépassé.\n\n\
         Threshold : {threshold_id}\n\
         Scope     : {scope} {target}\n\
         Période   : {period} [{p_start} → {p_end}]\n\
         Observé   : {observed:.2} gCO2eq (limite {max:.2})\n\
         Déclenché : {triggered}\n\n\
         Voir le dashboard admin : alertes → triggers.\n",
        threshold_id = payload.threshold_id,
        scope = payload.scope,
        target = payload.target_id.unwrap_or("-"),
        period = payload.period,
        p_start = payload.period_start,
        p_end = payload.period_end,
        observed = payload.observed_gco2eq,
        max = payload.gco2eq_max,
        triggered = payload.triggered_at,
    );

    let from = match smtp.from.parse() {
        Ok(addr) => addr,
        Err(e) => {
            return NotifyOutcome {
                ok: false,
                error: Some(format!("smtp_from invalide: {e}")),
            };
        },
    };
    let to_addr = match to.parse() {
        Ok(addr) => addr,
        Err(e) => {
            return NotifyOutcome {
                ok: false,
                error: Some(format!("destinataire email invalide: {e}")),
            };
        },
    };
    let msg = match Message::builder()
        .from(from)
        .to(to_addr)
        .subject(format!(
            "[Sobr.ia] Alerte seuil dépassé ({}/{}/{:.2}g)",
            payload.scope, payload.period, payload.observed_gco2eq
        ))
        .header(ContentType::TEXT_PLAIN)
        .body(body)
    {
        Ok(m) => m,
        Err(e) => {
            return NotifyOutcome {
                ok: false,
                error: Some(format!("compose: {e}")),
            };
        },
    };

    let transport = match build_smtp_transport(smtp) {
        Ok(t) => t,
        Err(e) => {
            return NotifyOutcome {
                ok: false,
                error: Some(e),
            };
        },
    };
    match transport.send(msg).await {
        Ok(_) => {
            let _ = event; // event est suffixé pour l'API mais pas relogué côté succès
            NotifyOutcome {
                ok: true,
                error: None,
            }
        },
        Err(e) => NotifyOutcome {
            ok: false,
            error: Some(format!("SMTP: {e}")),
        },
    }
}

/// Configuration SMTP lue depuis la table `config`.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// URL au format `smtps://user:pass@host:port` ou `smtp://host:port`.
    pub url: String,
    /// Adresse `From` (ex. `sobria@example.org`).
    pub from: String,
}

/// Construit la configuration SMTP depuis le KV `config` (smtp_url + smtp_from).
/// Renvoie `None` si l'une des deux clés est absente — signal de fallback log_only.
pub fn read_smtp_config(storage: &crate::storage::Storage) -> Option<SmtpConfig> {
    let url = storage.get_config("smtp_url").ok().flatten()?;
    let from = storage.get_config("smtp_from").ok().flatten()?;
    if url.trim().is_empty() || from.trim().is_empty() {
        return None;
    }
    Some(SmtpConfig { url, from })
}

fn build_smtp_transport(smtp: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    // On parse l'URL manuellement pour permettre `smtp://host:port` (sans creds).
    let (scheme, rest) = smtp
        .url
        .split_once("://")
        .ok_or_else(|| "smtp_url doit commencer par smtp:// ou smtps://".to_string())?;
    let (creds, host_port) = match rest.split_once('@') {
        Some((c, hp)) => (Some(c), hp),
        None => (None, rest),
    };
    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => (
            h.to_string(),
            p.parse::<u16>()
                .map_err(|e| format!("port invalide: {e}"))?,
        ),
        None => match scheme {
            "smtps" => (host_port.to_string(), 465),
            _ => (host_port.to_string(), 587),
        },
    };
    let mut builder = match scheme {
        "smtps" => AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .map_err(|e| format!("relay smtps: {e}"))?
            .port(port),
        "smtp" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host).port(port),
        other => return Err(format!("schéma SMTP inconnu: {other}")),
    };
    if let Some(c) = creds {
        if let Some((user, pass)) = c.split_once(':') {
            builder = builder.credentials(Credentials::new(user.to_string(), pass.to_string()));
        }
    }
    Ok(builder.build())
}

fn log_fallback(event: &AlertEvent, reason: &str) -> NotifyOutcome {
    warn!(
        threshold_id = %event.threshold.id,
        scope = %event.threshold.scope.as_str(),
        period = %event.threshold.period.as_str(),
        observed = %event.observed_gco2eq,
        max = %event.threshold.gco2eq_max,
        reason = reason,
        "alerte de seuil déclenchée"
    );
    NotifyOutcome {
        ok: true,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::periods::AlertPeriod;
    use crate::alerts::store::{AlertScope, NotifyKind, Threshold};
    use chrono::TimeZone;

    fn fake_event(kind: NotifyKind, target: Option<&str>) -> AlertEvent {
        let now = Utc.with_ymd_and_hms(2026, 5, 16, 12, 0, 0).unwrap();
        AlertEvent {
            trigger_id: "tr-x".into(),
            threshold: Threshold {
                id: "t-x".into(),
                scope: AlertScope::Team,
                target_id: None,
                period: AlertPeriod::Daily,
                gco2eq_max: 5.0,
                notify_kind: kind,
                notify_target: target.map(String::from),
                created_by_admin_id: "a-1".into(),
                created_at: now,
                disabled_at: None,
            },
            period_start: now,
            period_end: now,
            observed_gco2eq: 12.0,
        }
    }

    #[tokio::test]
    async fn log_only_always_succeeds() {
        let ev = fake_event(NotifyKind::LogOnly, None);
        let out = notify(&ev, None).await;
        assert!(out.ok);
        assert!(out.error.is_none());
    }

    #[tokio::test]
    async fn email_without_smtp_falls_back_to_log_only() {
        let ev = fake_event(NotifyKind::Email, Some("admin@example.org"));
        let out = notify(&ev, None).await;
        assert!(out.ok, "fallback log_only doit retourner ok=true");
    }

    #[tokio::test]
    async fn webhook_without_target_falls_back_to_log_only() {
        let ev = fake_event(NotifyKind::Webhook, None);
        let out = notify(&ev, None).await;
        assert!(out.ok, "fallback log_only doit retourner ok=true");
    }

    #[tokio::test]
    async fn webhook_to_unreachable_returns_error() {
        let ev = fake_event(NotifyKind::Webhook, Some("http://127.0.0.1:1/dead"));
        let out = notify(&ev, None).await;
        assert!(!out.ok);
        assert!(out.error.is_some());
    }
}
