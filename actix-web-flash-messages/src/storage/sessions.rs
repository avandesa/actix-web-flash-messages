use crate::storage::{FlashMessageStore, LoadError, StoreError};
use crate::FlashMessage;
use actix_session::UserSession;
use actix_web::dev::ResponseHead;
use actix_web::HttpRequest;

/// A session-based implementation of flash messages.
///
/// [`SessionMessageStore`] uses the session machinery provided by `actix-session`
/// to store and retrieve [`FlashMessage`]s.  
///
/// Use either [`SessionMessageStore::default`] or [`SessionMessageStore::default`]
/// to build an instance of [`SessionMessageStore`].
///
/// # Disclaimer
///
/// Be careful: you need to wrap your application in an additional middleware,
/// in addition to [`FlashMessagesFramework`], that provides persistence for the
/// session data.  
/// `actix-session` provides a cookie-based implementation of sessions via
/// [`actix_session::CookieSession`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.CookieSession.html).  
/// Alternatively, you can use [`RedisSession`](https://docs.rs/actix-redis/0.10.0-beta.2/actix_redis/struct.RedisSession.html)
/// from `actix-redis`.
///
/// Flash messages will not work if you fail to mount a storage backend for your sessions.
///
/// You can find examples of application using [`SessionMessageStore`] on GitHub:
/// using both
/// [cookie-based sessions](https://github.com/LukeMathWalker/actix-web-flash-messages/tree/main/examples/session-cookie) and
/// [Redis-based sessions](https://github.com/LukeMathWalker/actix-web-flash-messages/tree/main/examples/session-redis).
///
/// [`FlashMessagesFramework`]: crate::FlashMessagesFramework
#[derive(Clone)]
pub struct SessionMessageStore {
    key: String,
}

impl SessionMessageStore {
    /// Build a new [`SessionMessageStore`] and specify which key should be used
    /// to store outgoing flash messages in the session map.
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl Default for SessionMessageStore {
    fn default() -> Self {
        Self {
            key: "_flash".into(),
        }
    }
}

impl FlashMessageStore for SessionMessageStore {
    fn load(&self, request: &HttpRequest) -> Result<Vec<FlashMessage>, LoadError> {
        let session = request.get_session();
        let messages = session
            .get(&self.key)
            .map_err(|e| {
                // This sucks - we are losing all context.
                let e = anyhow::anyhow!("{}", e)
                    .context("Failed to retrieve flash messages from session storage.");
                LoadError::GenericError(e)
            })?
            .unwrap_or_default();
        Ok(messages)
    }

    fn store(
        &self,
        messages: &[FlashMessage],
        request: HttpRequest,
        _response: &mut ResponseHead,
    ) -> Result<(), StoreError> {
        let session = request.get_session();
        if messages.is_empty() {
            // Make sure to clear up previous flash messages!
            // No need to do this on the other if-branch because we are overwriting
            // any pre-existing flash message with a new value.
            session.remove(&self.key);
        } else {
            session.insert(&self.key, messages).map_err(|e| {
                // This sucks - we are losing all context.
                let e = anyhow::anyhow!("{}", e)
                    .context("Failed to retrieve flash messages from session storage.");
                StoreError::GenericError(e)
            })?;
        }
        Ok(())
    }
}
