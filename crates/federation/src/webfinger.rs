use crate::FederationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebfingerLink {
    pub rel: String,
    #[serde(rename = "type", default)]
    pub link_type: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebfingerResponse {
    pub subject: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub links: Vec<WebfingerLink>,
}

impl WebfingerResponse {
    pub fn new(subject: String) -> Self {
        Self {
            subject,
            aliases: Vec::new(),
            links: Vec::new(),
        }
    }

    pub fn add_self_link(&mut self, href: &str) {
        self.links.push(WebfingerLink {
            rel: "self".to_string(),
            link_type: Some("application/activity+json".to_string()),
            href: Some(href.to_string()),
            template: None,
        });
    }

    pub fn add_profile_link(&mut self, href: &str) {
        self.links.push(WebfingerLink {
            rel: "http://webfinger.net/rel/profile-page".to_string(),
            link_type: Some("text/html".to_string()),
            href: Some(href.to_string()),
            template: None,
        });
    }

    pub fn find_link_by_rel(&self, rel: &str) -> Option<&WebfingerLink> {
        self.links.iter().find(|l| l.rel == rel)
    }

    pub fn activitypub_actor_url(&self) -> Option<&str> {
        self.find_link_by_rel("self")
            .and_then(|l| l.href.as_deref())
    }
}

pub fn parse_webfinger_query(query: &str) -> Result<(String, String), FederationError> {
    let query = query.trim_start_matches("acct:");
    let parts: Vec<&str> = query.splitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(FederationError::WebfingerError(format!(
            "invalid webfinger query: {query}"
        )));
    }
    let resource = parts[0].to_string();
    let host = parts[1].to_string();
    if resource.is_empty() || host.is_empty() {
        return Err(FederationError::WebfingerError(
            "invalid webfinger query: empty resource or host".into(),
        ));
    }
    Ok((resource, host))
}

pub async fn resolve_webfinger(
    resource: &str,
    host: &str,
) -> Result<WebfingerResponse, FederationError> {
    let encoded =
        url::form_urlencoded::byte_serialize(format!("acct:{resource}@{host}").as_bytes())
            .collect::<String>();
    let url_str = format!("https://{host}/.well-known/webfinger?resource={encoded}");

    let response = reqwest::get(&url_str)
        .await
        .map_err(|e| FederationError::WebfingerError(format!("HTTP request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(FederationError::WebfingerError(format!(
            "webfinger lookup failed with status: {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| FederationError::WebfingerError(format!("failed to read response: {e}")))?;

    let webfinger: WebfingerResponse = serde_json::from_str(&body)
        .map_err(|e| FederationError::WebfingerError(format!("invalid JSON response: {e}")))?;

    Ok(webfinger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_webfinger_query_valid() {
        let (resource, host) = parse_webfinger_query("user@example.com").unwrap();
        assert_eq!(resource, "user");
        assert_eq!(host, "example.com");
    }

    #[test]
    fn test_parse_webfinger_query_with_acct_prefix() {
        let (resource, host) = parse_webfinger_query("acct:user@example.com").unwrap();
        assert_eq!(resource, "user");
        assert_eq!(host, "example.com");
    }

    #[test]
    fn test_parse_webfinger_query_invalid() {
        assert!(parse_webfinger_query("invalid").is_err());
        assert!(parse_webfinger_query("@host").is_err());
    }

    #[test]
    fn test_webfinger_response_creation() {
        let mut response = WebfingerResponse::new("acct:user@example.com".into());
        response.add_self_link("https://example.com/actor");
        response.add_profile_link("https://example.com/profile");
        assert_eq!(response.subject, "acct:user@example.com");
        assert_eq!(response.links.len(), 2);
    }

    #[test]
    fn test_webfinger_activitypub_actor_url() {
        let mut response = WebfingerResponse::new("acct:user@node.example".into());
        response.add_self_link("https://node.example/users/user");
        assert_eq!(
            response.activitypub_actor_url(),
            Some("https://node.example/users/user")
        );
    }

    #[test]
    fn test_webfinger_serialization_roundtrip() {
        let mut response = WebfingerResponse::new("acct:test@example.com".into());
        response.add_self_link("https://example.com/actor");
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: WebfingerResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.subject, response.subject);
    }
}
